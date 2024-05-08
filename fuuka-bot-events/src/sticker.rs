//! Events for stickers.

use std::collections::{HashMap, HashSet};

use matrix_sdk::ruma::{
    events::{macros::EventContent, room::ImageInfo},
    OwnedMxcUri,
};
use serde::{Deserialize, Serialize};

/// The contents for a room sticker.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "im.ponies.room_emotes", kind = State, state_key_type = String)]
pub struct RoomStickerEventContent {
    /// A list of images avaliable.
    pub images: HashMap<String, StickerData>,
    /// Sticker pack info.
    pub pack: StickerPack,
}

/// Sticker data.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StickerData {
    /// Sticker URI.
    pub url: OwnedMxcUri,
    /// Sticker image info.
    pub info: ImageInfo,
}

/// Sticker pack info.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StickerPack {
    /// Sticker URI.
    pub avatar_url: OwnedMxcUri,
    /// Sticker pack name.
    pub display_name: String,
    /// Usages for the sticker.
    pub usage: HashSet<StickerUsage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StickerUsage {
    Emoticon, Sticker
}
