//! Fuuka Bot configuration.

use cronchik::CronSchedule;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use secrecy::SecretString;
use serde::{Deserialize, Deserializer};
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};
use url::Url;

use crate::IllustTagsInfoExt;

/// The config of Fuuka bot.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Command configs.
    pub command: CommandConfig,
    /// The homeserver URL to connect to.
    pub matrix: MatrixConfig,
    /// Admin user ID.
    pub admin_user: Option<OwnedUserId>,
    /// Pixiv related configs.
    pub pixiv: PixivConfig,
    /// Optional room features.
    #[serde(default)]
    pub features: FeaturesConfig,
    /// Media proxy configuration.
    pub media_proxy: MediaProxyConfig,
    /// HTTP Services configuration.
    pub services: ServiceConfig,
    /// Stickers feature related configuration.
    pub stickers: Option<StickerConfig>,
    /// Nixpkgs PR configuration
    pub nixpkgs_pr: Option<NixpkgsPrConfig>,
}

/// Command configs.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CommandConfig {
    pub prefix: String,
}

/// Matrix related configs.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MatrixConfig {
    pub homeserver: Url,
    #[serde(
        default = "matrix_config_default_timeout",
        deserialize_with = "deserialize_duration_from_seconds"
    )]
    pub timeout: Duration,
}

/// Pixiv feature related configs.
#[derive(Debug, Clone)]
pub enum PixivConfig {
    Disabled,
    Enabled {
        token: SecretString,
        r18: bool,
        tag_triggers: TagTriggers,
    },
}

impl<'de> Deserialize<'de> for PixivConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, rename_all = "kebab-case")]
        #[allow(dead_code)]
        enum PixivConfig {
            Disabled {
                #[serde(default)]
                enabled: serde_bool::False,
            },
            Enabled {
                enabled: serde_bool::True,
                token: SecretString,
                r18: bool,
                tag_triggers: TagTriggers,
            },
        }
        PixivConfig::deserialize(deserializer).map(|value| match value {
            PixivConfig::Disabled { enabled: _ } => Self::Disabled,
            PixivConfig::Enabled {
                enabled: _,
                token,
                r18,
                tag_triggers,
            } => Self::Enabled {
                token,
                r18,
                tag_triggers,
            },
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TriggerItem {
    pub required_tags: Vec<String>,
    pub target: String,
}

#[derive(Debug, Clone, Default)]
pub struct TagTriggers {
    pub(crate) room_scoped_config: HashMap<OwnedRoomId, Vec<TriggerItem>>,
    pub(crate) global_config: Vec<TriggerItem>,
}

impl<'de> Deserialize<'de> for TagTriggers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug, Clone)]
        #[serde(rename_all = "kebab-case")]
        struct TagTriggers {
            #[serde(flatten)]
            item: TriggerItem,
            #[serde(default)]
            rooms: Option<Vec<OwnedRoomId>>,
        }

        Vec::<TagTriggers>::deserialize(deserializer).map(|value| {
            let mut room_scoped_config: HashMap<OwnedRoomId, Vec<TriggerItem>> = HashMap::new();
            let mut global_config = Vec::new();
            for item in value {
                match item.rooms {
                    Some(rooms) => {
                        for room in rooms {
                            room_scoped_config
                                .entry(room)
                                .or_default()
                                .push(item.item.clone())
                        }
                    }
                    None => global_config.push(item.item.clone()),
                }
            }

            Self {
                room_scoped_config,
                global_config,
            }
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeaturesConfig(HashMap<OwnedRoomId, RoomFeatures>);

impl<'de> Deserialize<'de> for FeaturesConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug, Clone)]
        #[serde(rename_all = "kebab-case")]
        struct FeaturesConfig {
            #[serde(flatten)]
            features: RoomFeatures,
            room: OwnedRoomId,
        }
        Vec::<FeaturesConfig>::deserialize(deserializer).map(|value| {
            let mut result = HashMap::new();
            for item in value {
                result.insert(item.room, item.features);
            }
            Self(result)
        })
    }
}

impl FeaturesConfig {
    pub fn room_jerryxiao_enabled(&self, room_id: &RoomId) -> bool {
        self.0
            .get(room_id)
            .map(|res| res.jerryxiao)
            .unwrap_or_default()
    }

    pub fn room_fortune_enabled(&self, room_id: &RoomId) -> bool {
        self.0
            .get(room_id)
            .map(|res| res.fortune)
            .unwrap_or_default()
    }

    pub fn room_pixiv_enabled(&self, room_id: &RoomId) -> bool {
        self.0.get(room_id).map(|res| res.pixiv).unwrap_or_default()
    }

    pub fn room_pixiv_r18_enabled(&self, room_id: &RoomId) -> bool {
        self.0
            .get(room_id)
            .map(|res| res.pixiv_r18)
            .unwrap_or_default()
    }
}

/// Sticker feature config.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StickerConfig {
    /// Room for storing stickers.
    pub send_to: OwnedRoomId,
}

/// What message features are avaliable.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RoomFeatures {
    /// Enable Jerry Xiao like functions.
    #[serde(default)]
    pub jerryxiao: bool,
    /// Enable fortune.
    #[serde(default)]
    pub fortune: bool,
    /// Enable pixiv.
    #[serde(default)]
    pub pixiv: bool,
    /// Enable pixiv (R18).
    #[serde(default)]
    pub pixiv_r18: bool,
}

/// Configure various backend APIs
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceConfig {
    pub hitokoto: HitokotoConfig,
    pub github: GitHubConfig,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct HitokotoConfig {
    /// Hitokoto API endpoint.
    /// The API should implment <https://developer.hitokoto.cn/sentence/#%E6%8E%A5%E5%8F%A3%E8%AF%B4%E6%98%8E>.
    #[serde(default = "hitokoto_config_default_base_url")]
    pub base_url: Url,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GitHubConfig {
    /// GitHub API endpoint.
    #[serde(default = "github_config_default_base_url")]
    pub base_url: Url,
    pub pr_tracker: PrTrackerConfig,
}

#[derive(Default, Debug, Clone)]
pub enum PrTrackerConfig {
    #[default]
    Disabled,
    Enabled {
        cron: Box<Option<CronSchedule>>,
        targets: BTreeMap<RepositoryParts, BTreeMap<String, String>>,
    },
}

impl<'de> Deserialize<'de> for PrTrackerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, rename_all = "kebab-case")]
        #[allow(dead_code)]
        enum PrTrackerConfig {
            Disabled {
                #[serde(default)]
                enabled: serde_bool::False,
            },
            Enabled {
                enabled: serde_bool::True,
                cron: Box<Option<CronSchedule>>,
                targets: BTreeMap<RepositoryParts, BTreeMap<String, String>>,
            },
        }
        PrTrackerConfig::deserialize(deserializer).map(|value| match value {
            PrTrackerConfig::Disabled { enabled: _ } => Self::Disabled,
            PrTrackerConfig::Enabled {
                enabled: _,
                targets,
                cron,
            } => Self::Enabled { targets, cron },
        })
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct RepositoryParts {
    pub owner: String,
    pub repo: String,
}

impl TryFrom<String> for RepositoryParts {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let res: Vec<_> = value.split('/').collect();
        let owner = res.first().map(|str| str.to_string());
        let repo = res.get(1).map(|str| str.to_string());
        let (Some(owner), Some(repo)) = (owner, repo) else {
            anyhow::bail!("invalid format: not in [owner]/[repo] format");
        };

        Ok(Self { owner, repo })
    }
}

impl TagTriggers {
    pub fn check_for_tag_triggers(
        &self,
        tags: &pixrs::IllustTagsInfo,
        room_id: &RoomId,
    ) -> Option<&str> {
        if let Some(infos) = self.room_scoped_config.get(room_id) {
            for item in infos {
                if tags.has_any_tag(
                    &item
                        .required_tags
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>(),
                ) {
                    return Some(&item.target);
                }
            }
        } else {
            for item in &self.global_config {
                if tags.has_any_tag(
                    &item
                        .required_tags
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>(),
                ) {
                    return Some(&item.target);
                }
            }
        }

        None
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NixpkgsPrConfig {
    pub token: String,
    pub cron: Option<CronSchedule>,
}

#[derive(Debug, Clone)]
pub enum MediaProxyConfig {
    Disabled,
    Enabled {
        listen: String,
        public_url: Url,
        ttl_seconds: u32,
    },
}

impl<'de> Deserialize<'de> for MediaProxyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, rename_all = "kebab-case")]
        #[allow(dead_code)]
        enum MediaProxyConfig {
            Disabled {
                #[serde(default)]
                enabled: serde_bool::False,
            },
            Enabled {
                enabled: serde_bool::True,
                listen: String,
                public_url: Url,
                ttl_seconds: u32,
            },
        }
        MediaProxyConfig::deserialize(deserializer).map(|value| match value {
            MediaProxyConfig::Disabled { enabled: _ } => Self::Disabled,
            MediaProxyConfig::Enabled {
                enabled: _,
                listen,
                public_url,
                ttl_seconds,
            } => Self::Enabled {
                listen,
                public_url,
                ttl_seconds,
            },
        })
    }
}

/// Returns the default Hitokoto service API URL,
/// which is https://v1.hitokoto.cn
fn hitokoto_config_default_base_url() -> Url {
    "https://v1.hitokoto.cn".parse().unwrap()
}

/// Returns the default Hitokoto service API URL,
/// which is https://v1.hitokoto.cn
fn github_config_default_base_url() -> Url {
    "https://api.github.com".parse().unwrap()
}

/// Returns the default duration of Matrix connection timeout,
/// which is 5 minutes.
fn matrix_config_default_timeout() -> Duration {
    static DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

    DEFAULT_TIMEOUT
}

fn deserialize_duration_from_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let duration_sec = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(duration_sec))
}
