//! Types dealing with illustrations.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::common::{AIType, Restriction};

/// Illust info.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IllustInfo {
    /// The ID of the illust.
    #[serde(with = "crate::serde::from_str")]
    pub id: u64,
    /// The title of the illust.
    pub title: String,
    /// The description of the illust in HTML format.
    pub description: String,
    /// The type of the illust.
    pub illust_type: IllustType,
    /// The date the illust is created.
    pub create_date: crate::DateTime,
    /// The date the illust is uploaded.
    pub upload_date: crate::DateTime,
    /// The restriction type for the illust.
    #[serde(rename = "xRestrict")]
    pub restriction: Restriction,
    /// Whether this artwork is generated with AI.
    pub ai_type: AIType,
    /// The URLs avaliable in the (first) image of the illust.
    pub urls: IllustImageUrls,
    /// The tags infomation of the illust.
    pub tags: IllustTagsInfo,
    /// The User ID of the author.
    #[serde(with = "crate::serde::from_str")]
    pub user_id: u64,
    /// The name of the author.
    pub user_name: String,
    /// All illust IDs by the same author.
    #[serde(deserialize_with = "crate::serde::dict_key_as_vec::deserialize")]
    pub user_illusts: Vec<u64>,
    /// Whether the account holder has liked the illust.
    #[serde(rename = "likeData")]
    pub liked: bool,
    /// The width of the (first) illust.
    pub width: u64,
    /// The height of the (first) illust.
    pub height: u64,
    /// How many pages the illust have.
    pub page_count: u64,
    /// How many bookmarks the illust have.
    pub bookmark_count: u64,
    /// How many likes the illust have.
    pub like_count: u64,
    /// How many comments the illust have.
    pub comment_count: u64,
    #[allow(missing_docs)]
    pub response_count: u64,
    /// How many views the illust have.
    pub view_count: u64,
    /// Whether this illust is original work.
    #[serde(rename = "isOriginal")]
    pub original: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum IllustType {
    Illustration = 0,
    Manga = 1,
    Animation = 2,
}

impl<'de> Deserialize<'de> for IllustType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Illustration),
            1 => Ok(Self::Manga),
            2 => Ok(Self::Animation),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value.into()),
                &"0, 1 or 2",
            )),
        }
    }
}

impl Serialize for IllustType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// The URLs avaliable in the image.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IllustImageUrls {
    /// The small variant URL of the image.
    pub small: String,
    /// The medium variant URL of the image.
    pub regular: String,
    /// The original variant URL of the image.
    pub original: String,
    // TODO: Thumbs
}

/// Illust tag base info.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IllustTagsInfo {
    /// The illust author ID.
    #[serde(with = "crate::serde::from_str")]
    pub author_id: u64,
    /// Whether the tags has been locked.
    pub is_locked: bool,
    /// Avaliable tags.
    pub tags: Vec<IllustTag>,
    /// Whether tags can be added.
    pub writable: bool,
}

/// A tag of an illust.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IllustTag {
    /// The untranslated tag.
    pub tag: String,
    /// Whether this tag has been locked.
    pub locked: bool,
    /// Whether this tag can be deleted.
    pub deletable: bool,
    /// The user ID of the tagger.
    #[serde(default, with = "crate::serde::from_str::option")]
    pub user_id: Option<u64>,
    /// The user name of the tagger.
    #[serde(default)]
    pub user_name: Option<String>,
    /// Translations of the tag.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translation: BTreeMap<String, String>,
}
