use anyhow::Context;
use matrix_sdk::{
    config::SyncSettings,
    ruma::{api::client::uiaa, presence::PresenceState},
};
use rpassword::read_password;

/// Wraps [matrix_sdk::encryption::Encryption::bootstrap_cross_signing_if_needed] for CLI,
/// which prompts for the account's password using [rpassword].
///
/// **NOTE**: Only supports password UIAA.
///
/// See referenced function for more detail.
pub(crate) async fn bootstrap_cross_signing_if_needed(
    client: &matrix_sdk::Client,
) -> anyhow::Result<()> {
    if let Err(e) = client
        .encryption()
        .bootstrap_cross_signing_if_needed(None)
        .await
    {
        if let Some(response) = e.as_uiaa_response() {
            use std::io::Write;

            print!("Enter password for preparing cross signing: ");
            std::io::stdout().flush()?;
            let password = read_password()?;
            let mut password = uiaa::Password::new(
                uiaa::UserIdentifier::UserIdOrLocalpart(client.user_id().unwrap().to_string()),
                password,
            );
            password.session = response.session.clone();

            client
                .encryption()
                .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                .await
                .context("Couldn't bootstrap cross signing")?
        } else {
            anyhow::bail!("Error during cross signing bootstrap {:#?}", e);
        }
    }

    Ok(())
}

/// Wraps [matrix_sdk::encryption::Encryption::bootstrap_cross_signing] for CLI,
/// which prompts for the account's password using [rpassword].
///
/// **NOTE**: Only supports password UIAA.
///
/// See referenced function for more detail.
pub(crate) async fn bootstrap_cross_signing(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    if let Err(e) = client.encryption().bootstrap_cross_signing(None).await {
        if let Some(response) = e.as_uiaa_response() {
            use std::io::Write;

            print!("Enter password for preparing cross signing: ");
            std::io::stdout().flush()?;
            let password = read_password()?;
            let mut password = uiaa::Password::new(
                uiaa::UserIdentifier::UserIdOrLocalpart(client.user_id().unwrap().to_string()),
                password,
            );
            password.session = response.session.clone();

            client
                .encryption()
                .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                .await
                .context("Couldn't bootstrap cross signing")?
        } else {
            anyhow::bail!("Error during cross signing bootstrap {:#?}", e);
        }
    }

    Ok(())
}

/// Wraps [matrix_sdk::encryption::Encryption::reset_cross_signing] for CLI,
/// which prompts for the account's password using [rpassword].
///
/// See referenced function for more detail.
pub(crate) async fn reset_cross_signing(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    use matrix_sdk::encryption::CrossSigningResetAuthType;

    if let Some(handle) = client.encryption().reset_cross_signing().await? {
        match handle.auth_type() {
            CrossSigningResetAuthType::Uiaa(uiaa) => {
                use matrix_sdk::ruma::api::client::uiaa;
                use rpassword::read_password;
                use std::io::Write;

                print!("Enter password for resetting cross signing: ");
                std::io::stdout().flush()?;
                let password = read_password()?;
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::UserIdOrLocalpart(client.user_id().unwrap().to_string()),
                    password,
                );
                password.session = uiaa.session.clone();

                handle
                    .auth(Some(uiaa::AuthData::Password(password)))
                    .await?;
            }
            CrossSigningResetAuthType::OAuth(o) => {
                println!(
                    "To reset your end-to-end encryption cross-signing identity, \
                            you first need to approve it at {}",
                    o.approval_url
                );
                handle.auth(None).await?;
            }
        }
    }

    Ok(())
}

/// Recover all the secrets from the homeserver.
/// Prompts for the recovery key using [rpassword].
///
/// See [matrix_sdk::encryption::recovery::Recovery::recover] for more detail.
pub(crate) async fn recover_cross_signing(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    use rpassword::read_password;

    let recovery = client.encryption().recovery();
    print!("Enter recovery key for recovering cross signing: ");
    let recovery_key = read_password()?;
    recovery.recover(&recovery_key).await?;

    Ok(())
}

/// Create a new [matrix_sdk::encryption::secret_storage::SecretStore],
/// also downloads backups to this client.
///
/// See [matrix_sdk::encryption::secret_storage::SecretStorage::create_secret_store] for more detail.
pub(crate) async fn create_secret_store(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let store = client
        .encryption()
        .secret_storage()
        .create_secret_store()
        .await?;
    let key = store.secret_storage_key();
    println!("Your secret storage key is {key}, save it somewhere safe.");
    store.import_secrets().await?;

    Ok(())
}

/// Creates a new backup.
pub(crate) async fn new_backup(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let backups = client.encryption().backups();
    backups.create().await?;

    Ok(())
}

pub(crate) async fn enable_key_backups(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let backup = client.encryption().backups();

    if backup.fetch_exists_on_server().await? {
        tracing::debug!(
            "Bot has an existing server key backup that is valid, skipping recovery provision."
        );
        return Ok(());
    }

    let recovery = client.encryption().recovery();
    let enable = recovery.enable().wait_for_backups_to_upload();

    let mut progress = enable.subscribe_to_progress();

    tokio::spawn(async move {
        use futures_util::StreamExt;
        use matrix_sdk::encryption::recovery::EnableProgress;

        while let Some(update) = progress.next().await {
            let Ok(update) = update else {
                panic!("Update to the enable progress lagged");
            };

            match update {
                EnableProgress::CreatingBackup => {
                    tracing::debug!("Creating a new backup");
                }
                EnableProgress::CreatingRecoveryKey => {
                    tracing::debug!("Creating a new recovery key");
                }
                EnableProgress::Done { .. } => {
                    tracing::debug!("Recovery has been enabled");
                    break;
                }
                _ => (),
            }
        }
    });

    match enable.await {
        Ok(key) => tracing::info!("The recovery key is: {key}"),
        Err(e) => tracing::warn!("Error while enabling backup: {e:#}"),
    }

    Ok(())
}

#[tracing::instrument(name = "encryption", skip_all, err)]
pub(crate) async fn ensure_self_device_verified(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let encryption = client.encryption();
    let has_keys = encryption
        .cross_signing_status()
        .await
        .map(|status| status.has_self_signing && status.has_master)
        .unwrap_or_default();

    if !has_keys {
        tracing::warn!("No self signing key to sign this own device!");
        return Ok(());
    }

    if let Some(device) = encryption.get_own_device().await?
        && !device.is_cross_signed_by_owner()
    {
        device.verify().await?
    }

    Ok(())
}

#[tracing::instrument(skip_all, err)]
pub(crate) async fn initial_sync(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    tracing::info!("Initial sync beginning...");
    client
        .sync_once(SyncSettings::default().set_presence(PresenceState::Online))
        .await?;
    tracing::info!("Initial sync completed.");

    Ok(())
}

pub(crate) async fn log_encryption_info(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let encryption = client.encryption();
    let cross_signing_status = encryption.cross_signing_status().await;
    if let Some(device) = encryption.get_own_device().await? {
        let device_id = device.device_id();
        tracing::debug!(
            "Own device ID: {device_id}, Cross signing status: {cross_signing_status:#?}, is_cross_signed_by_owner = {is_cross_signed_by_owner}, is_verified = {is_verified}, is_verified_with_cross_signing = {is_verified_with_cross_signing}",
            is_cross_signed_by_owner = device.is_cross_signed_by_owner(),
            is_verified = device.is_verified(),
            is_verified_with_cross_signing = device.is_verified_with_cross_signing(),
        );
    }

    Ok(())
}
