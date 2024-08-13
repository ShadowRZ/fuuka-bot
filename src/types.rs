//! Types for external API.

use serde::Deserialize;
use thiserror::Error;
use time::OffsetDateTime;

/// <https://developer.hitokoto.cn/sentence/#%E8%BF%94%E5%9B%9E%E4%BF%A1%E6%81%AF>
#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct HitokotoResult {
    pub uuid: String,
    pub hitokoto: String,
    pub from: String,
    pub from_who: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateMetadata {
    #[serde(rename = "crate")]
    pub crate_info: CrateInfo,
    pub versions: Vec<CrateVersion>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateInfo {
    pub description: Option<String>,
    pub name: String,
    pub max_stable_version: String,
    pub max_version: String,
    pub downloads: u32,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateVersion {
    pub license: Option<String>,
    pub num: String,
    pub rust_version: Option<String>,
    pub yanked: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct BiliBiliVideo {
    pub video_data: BiliBiliVideoData,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct BiliBiliVideoData {
    pub bvid: String,
    pub title: String,
    pub desc: String,
    #[serde(deserialize_with = "crate::de::deserialize_unix_timestamp")]
    pub pubdate: OffsetDateTime,
    pub owner: BiliBiliVideoOwner,
    pub stat: BiliBiliVideoStat,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct BiliBiliVideoStat {
    pub view: i128,
    pub danmaku: i128,
    pub reply: i128,
    pub favorite: i128,
    pub coin: i128,
    pub share: i128,
    pub like: i128,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct BiliBiliVideoOwner {
    pub mid: i128,
    pub name: String,
}

/// Error types.
#[derive(Error, Debug)]
pub enum Error {
    /// This command requires replying to an event.
    #[error("Replying to a event is required for this command")]
    RequiresReply,
    /// This command is missing an argument.
    #[error("Missing an argument: {0}")]
    MissingArgument(&'static str),
    /// Invaild argument passed into an argument.
    #[error("Invaild argument passed for {arg}: {source}")]
    InvaildArgument {
        /// The argument that is invaild.
        arg: &'static str,
        #[source]
        /// The source error that caused it to happen.
        source: anyhow::Error,
    },
    /// An unexpected error happened.
    #[error("{0}")]
    UnexpectedError(&'static str),
    /// An unknown command was passed.
    #[error("Unrecognized command {0}")]
    UnknownCommand(String),
}
