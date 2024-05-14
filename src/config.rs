//! Fuuka Bot configuration.

use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
/// The config of Fuuka bot.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    /// HTTP Services configuration.
    pub services: Option<ServiceConfig>,
    /// Stickers feature related configuration.
    pub stickers: Option<StickerConfig>,
}

/// Command configs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CommandConfig {
    pub prefix: String,
}

/// Matrix related configs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MatrixConfig {
    pub homeserver: Url,
}

/// Pixiv feature related configs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PixivConfig {
    pub enabled: bool,
    pub r18: bool,
    /// Pixiv PHPSESSID.
    /// See <https://pixivfe.pages.dev/obtaining-pixivfe-token/>
    pub token: Option<String>,
    #[serde(default)]
    pub traps: TrapConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct TrapItemInner {
    #[serde(flatten)]
    item: TrapItem,
    #[serde(default)]
    rooms: Option<Vec<OwnedRoomId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TrapItem {
    pub required_tags: Vec<String>,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(from = "Vec<TrapItemInner>")]
pub struct TrapConfig {
    room_scoped_config: HashMap<OwnedRoomId, Vec<TrapItem>>,
    global_config: Vec<TrapItem>,
}

impl From<Vec<TrapItemInner>> for TrapConfig {
    fn from(value: Vec<TrapItemInner>) -> Self {
        let mut room_scoped_config: HashMap<OwnedRoomId, Vec<TrapItem>> = HashMap::new();
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
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct RoomFeaturesInner {
    #[serde(flatten)]
    features: RoomFeatures,
    room: OwnedRoomId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(from = "Vec<RoomFeaturesInner>")]
pub struct FeaturesConfig(HashMap<OwnedRoomId, RoomFeatures>);

impl From<Vec<RoomFeaturesInner>> for FeaturesConfig {
    fn from(value: Vec<RoomFeaturesInner>) -> Self {
        let mut result = HashMap::new();
        for item in value {
            result.insert(item.room, item.features);
        }
        Self(result)
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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StickerConfig {
    /// Room for storing stickers.
    pub send_to: OwnedRoomId,
}

/// What message features are avaliable.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceConfig {
    /// Hitokoto API endpoint.
    /// The API should implment <https://developer.hitokoto.cn/sentence/#%E6%8E%A5%E5%8F%A3%E8%AF%B4%E6%98%8E>.
    pub hitokoto: Option<Url>,
}
