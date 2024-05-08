//! Fuuka Bot configuration.

use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
/// The config of Fuuka bot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Command prefix.
    pub command_prefix: String,
    /// The homeserver URL to connect to.
    pub homeserver_url: Url,
    /// Admin user ID.
    pub admin_user: Option<OwnedUserId>,
    /// Optional room features.
    #[serde(default)]
    pub features: HashMap<OwnedRoomId, RoomFeatures>,
    /// HTTP Services configuration.
    pub services: Option<ServiceBackends>,
    /// Stickers feature related configuration.
    pub stickers: Option<StickerConfig>,
}

/// Sticker feature config.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StickerConfig {
    /// Room for storing stickers.
    pub sticker_room: OwnedRoomId,
}
/// What message features are avaliable.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RoomFeatures {
    /// Enable Jerry Xiao like functions.
    #[serde(default)]
    pub jerryxiao: bool,
    /// Enable randomdraw.
    #[serde(default)]
    pub randomdraw: bool,
}

/// Configure various backend APIs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceBackends {
    /// Hitokoto API endpoint.
    /// The API should implment <https://developer.hitokoto.cn/sentence/#%E6%8E%A5%E5%8F%A3%E8%AF%B4%E6%98%8E>.
    pub hitokoto: Option<Url>,
}
