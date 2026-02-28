//! Fuuka Bot Internals for interested.
//!
//! **WARNING: External crate links are broken in the build documentation on GitHub Pages, sorry.**
//!
//! ## User Agent
//!
//! The bot consistently uses the following user agent template:
//!
//! ```text
//! fuuka-bot/<version> (https://github.com/ShadowRZ/fuuka-bot)
//! ```
//!
//! Where `<version>` is the running version of the bot.
mod config;
mod env;
pub mod format;
pub mod matrix;
pub mod media_proxy;
pub mod message;
mod middleware;
pub mod services;
mod traits;
pub mod utils;

pub use crate::config::Config;
use crate::config::FeaturesConfig;
use crate::config::GitHubConfig;
use crate::config::MediaProxyConfig;
use crate::config::PixivConfig;
pub use crate::media_proxy::MediaProxy;
use crate::services::github::pr_tracker::streams::CronStream;
pub use crate::traits::*;

use clap::Parser;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::config::RequestConfig;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::presence::PresenceState;
use pixiv_ajax_api::PixivClient;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::task::JoinHandle;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

static APP_PRESENCE_TEXT: &str = concat!(
    "Fuuka Bot (v",
    env!("CARGO_PKG_VERSION"),
    ") | ",
    env!("CARGO_PKG_REPOSITORY")
);

#[derive(Clone)]
pub struct Context {
    pub prefix: String,
    pub admin_user: Option<OwnedUserId>,
    pub http: reqwest::Client,
    pub hitokoto: hitokoto_api::HitokotoClient,
    pub crates: crates_api::CratesClient,
    pub media_proxy: Option<MediaProxy>,
    pub pixiv: Option<(Arc<PixivClient>, Arc<crate::services::pixiv::Context>)>,
    pub features: FeaturesConfig,
    pub github: Option<crate::services::github::Context>,
}

#[derive(Debug, clap::Parser)]
#[command(disable_help_subcommand = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Create and upload a new cross signing identity.
    BootstrapCrossSigning {
        /// Only perform this action if that has not been done yet.
        #[arg(long)]
        if_needed: bool,
    },
    /// Reset the cross-signing keys.
    ResetCrossSigning,
    /// Recover all the secrets from the homeserver.
    RecoverCrossSigning,
    /// Create a new secret store.
    CreateSecretStore,
    /// Create a new backup version, encrypted with a new backup recovery key.
    NewBackup,
}

/// Builder for the bot.
#[derive(Default)]
pub struct Builder {
    with_key_backups: bool,
    with_optional_media_proxy: bool,
}

pub fn builder() -> Builder {
    Default::default()
}

impl Builder {
    pub fn with_key_backups(mut self) -> Self {
        self.with_key_backups = true;

        self
    }

    pub fn with_optional_media_proxy(mut self) -> Self {
        self.with_optional_media_proxy = true;

        self
    }

    pub fn build(self) -> anyhow::Result<crate::Client> {
        let args = Args::parse();
        use anyhow::Context;

        let session = crate::env::credentials().context("Getting credentials failed!")?;

        let config: Config = crate::env::config().context("Getting config failed!")?;

        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;

        let store_path = crate::env::store()?;
        let builder = matrix_sdk::Client::builder()
            .http_client(http.clone())
            .request_config(RequestConfig::new().timeout(config.matrix.timeout))
            .homeserver_url(&config.matrix.homeserver)
            .sqlite_store(store_path, None);

        Ok(crate::Client {
            args,
            config,
            session,
            http,
            builder,
            with_key_backups: self.with_key_backups,
            with_optional_media_proxy: self.with_optional_media_proxy,
        })
    }
}

pub struct Client {
    args: Args,
    config: Config,
    session: MatrixSession,
    http: reqwest::Client,
    builder: matrix_sdk::ClientBuilder,
    with_key_backups: bool,
    with_optional_media_proxy: bool,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        let Self {
            args,
            config,
            session,
            http,
            builder,
            with_key_backups,
            with_optional_media_proxy,
        } = self;
        let client = builder.build().await?;
        client.restore_session(session).await?;
        client.send_queue().set_enabled(true).await;

        if let Some(command) = args.command {
            match command {
                Subcommand::BootstrapCrossSigning { if_needed } => {
                    if if_needed {
                        crate::matrix::bootstrap_cross_signing_if_needed(&client).await?;
                    } else {
                        crate::matrix::bootstrap_cross_signing(&client).await?;
                    }
                }
                Subcommand::ResetCrossSigning => {
                    crate::matrix::reset_cross_signing(&client).await?;
                }
                Subcommand::RecoverCrossSigning => {
                    crate::matrix::recover_cross_signing(&client).await?;
                }
                Subcommand::CreateSecretStore => {
                    crate::matrix::create_secret_store(&client).await?;
                }
                Subcommand::NewBackup => {
                    crate::matrix::new_backup(&client).await?;
                }
            }

            return Ok(());
        }

        if with_key_backups {
            crate::matrix::enable_key_backups(&client).await?;
        }

        let media_proxy = media_proxy(&client, &config)?;
        if with_optional_media_proxy && let Some(ref media_proxy) = media_proxy {
            media_proxy.start().await?;
        }

        let pixiv = match config.pixiv {
            PixivConfig::Disabled => None,
            PixivConfig::Enabled {
                token,
                r18,
                tag_triggers,
            } => {
                use http_body_util::BodyExt;
                use tower::BoxError;
                use tower_http::ServiceBuilderExt;

                let service = tower::ServiceBuilder::new()
                    .concurrency_limit(1)
                    .rate_limit(1, Duration::from_secs(1))
                    .map_response_body(|resp: reqwest::Body| {
                        resp.map_err(|e| Into::into(Box::new(e) as BoxError))
                            .boxed()
                    })
                    .map_err(|e| Box::new(e) as BoxError)
                    .layer(crate::middleware::reqwest::ReqwestLayer)
                    .service(http.clone());
                let client = Arc::new(PixivClient::new(service, token));
                let context = crate::services::pixiv::Context { r18, tag_triggers };
                Some((client, Arc::new(context)))
            }
        };

        let prefix = config.command.prefix;

        let GitHubConfig {
            base_url,
            pr_tracker,
            token,
        } = config.services.github;

        let github = match pr_tracker {
            config::PrTrackerConfig::Enabled { cron, targets } => {
                use crate::services::github::pr_tracker::PrTrackerContext;
                use std::str::FromStr;
                let base_url = http::Uri::from_str(base_url.as_str())?;
                let octocrab = crate::services::github::octocrab(&http, base_url, token);
                let cron = cron.map(CronStream::new).map(Arc::new);

                Some(crate::services::github::Context {
                    octocrab,
                    cron,
                    pr_tracker: Arc::new(PrTrackerContext::new(targets)?),
                })
            }
            config::PrTrackerConfig::Disabled => None,
        };

        let hitokoto = {
            use http_body_util::BodyExt;
            use tower::BoxError;
            use tower_http::ServiceBuilderExt;

            let base_url = http::Uri::from_str(config.services.hitokoto.base_url.as_str())?;
            let service = tower::ServiceBuilder::new()
                .concurrency_limit(1)
                .rate_limit(1, Duration::from_secs(1))
                .map_response_body(|resp: reqwest::Body| {
                    resp.map_err(|e| Into::into(Box::new(e) as BoxError))
                        .boxed()
                })
                .layer(crate::middleware::reqwest::ReqwestLayer)
                .service(http.clone());
            hitokoto_api::HitokotoClient::new(service, base_url)
        };
        let crates = {
            use http_body_util::BodyExt;
            use tower::BoxError;
            use tower_http::ServiceBuilderExt;

            let base_url = http::Uri::from_static("https://crates.io");
            let service = tower::ServiceBuilder::new()
                .concurrency_limit(1)
                .rate_limit(1, Duration::from_secs(1))
                .map_response_body(|resp: reqwest::Body| {
                    resp.map_err(|e| Into::into(Box::new(e) as BoxError))
                        .boxed()
                })
                .layer(crate::middleware::reqwest::ReqwestLayer)
                .service(http.clone());
            crates_api::CratesClient::new(service, base_url)
        };

        let context = Context {
            prefix,
            http,
            pixiv,
            media_proxy,
            github,
            features: config.features,
            hitokoto,
            crates,
            admin_user: config.admin_user,
        };

        client.add_event_handler_context(context);
        crate::matrix::log_encryption_info(&client).await?;
        let task: JoinHandle<()> = tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    while let Err(e) = sync(&client).await {
                        use tokio::time::{sleep, Duration};
                        tracing::error!("Unexpected error happened, retrying in 10s: {e:#}");
                        sleep(Duration::from_secs(10)).await;
                    }
                } => {},
                _ = graceful_shutdown_future() => {
                    tracing::info!("Bye!");
                },
            }
        });

        Ok(task.await?)
    }
}

fn media_proxy(client: &matrix_sdk::Client, config: &Config) -> anyhow::Result<Option<MediaProxy>> {
    match &config.media_proxy {
        MediaProxyConfig::Enabled {
            listen,
            public_url,
            ttl_seconds,
        } => {
            use anyhow::Context;

            let jwk = crate::env::jwk_token().context("Locate JWK file failed")?;
            let media_proxy = MediaProxy::new(
                jwk,
                client,
                listen.clone(),
                public_url.clone(),
                *ttl_seconds,
            )?;
            Ok(Some(media_proxy))
        }
        MediaProxyConfig::Disabled => Ok(None),
    }
}

/// A sharable graceful shutdown signal.
pub async fn graceful_shutdown_future() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn sync(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    crate::matrix::initial_sync(client).await?;
    if let Err(e) = crate::matrix::ensure_self_device_verified(client).await {
        tracing::warn!("Failed to ensure this device is verified: {e:#}");
    }
    let settings = SyncSettings::default().set_presence(PresenceState::Online);
    use matrix_sdk::ruma::api::client::presence::set_presence::v3::Request;
    if let Some(user_id) = client.user_id() {
        let mut presence = Request::new(user_id.into(), PresenceState::Online);
        presence.status_msg = Some(APP_PRESENCE_TEXT.to_string());
        if let Err(e) = client.send(presence).await {
            tracing::warn!("Failed to set presence: {e:#}");
        }
    }

    let send_queue = client.send_queue();

    tokio::spawn(async move {
        send_queue
            .respawn_tasks_for_rooms_with_unsent_requests()
            .await
    });

    let h1 = client.add_event_handler(crate::message::on_sync_message);
    let h2 = client.add_event_handler(crate::matrix::on_stripped_member);
    let h3 = client.add_event_handler(crate::matrix::on_room_replace);

    if let Err(e) = client.sync(settings).await {
        client.remove_event_handler(h1);
        client.remove_event_handler(h2);
        client.remove_event_handler(h3);
        return Err(e.into());
    }
    Ok(())
}
