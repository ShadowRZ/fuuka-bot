//! Types for external API.

use serde::Deserialize;

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
