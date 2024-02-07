//! Types for external API.

use serde::Deserialize;

/// <https://developer.hitokoto.cn/sentence/#%E8%BF%94%E5%9B%9E%E4%BF%A1%E6%81%AF>
#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct HitokotoResult {
    pub uuid: String,
    pub hitokoto: String,
    pub from: String,
    pub from_who: String,
}
