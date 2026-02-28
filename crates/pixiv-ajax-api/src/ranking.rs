use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct Ranking {
    pub contents: Vec<RankingItem>,
    #[serde(deserialize_with = "crate::serde::false_is_none::deserialize")]
    pub prev: Option<u32>,
    #[serde(deserialize_with = "crate::serde::false_is_none::deserialize")]
    pub next: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct RankingItem {
    pub title: String,
    pub tags: Vec<String>,
    pub user_name: String,
    pub profile_img: String,
    pub illust_id: u64,
    pub user_id: u64,
    pub width: u64,
    pub height: u64,
    pub view_count: u64,
}

/// The ranking mode.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingMode {
    Daily,
    Weekly,
    Monthly,
    Rookie,
    Original,
    Male,
    Female,
    DailyR18,
    WeeklyR18,
    MaleR18,
    FemaleR18,
    R18G,
}

/// The content in ranking.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingContent {
    All,
    Illust,
    Ugoira,
    Manga,
}
