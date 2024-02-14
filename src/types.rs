//! Types for external API.

use serde::Deserialize;

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
    pub license: String,
    pub num: String,
    pub rust_version: String,
    pub yanked: bool,
}
