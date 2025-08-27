//! Extracts Pixiv URLs.
use matrix_sdk::{
    Room,
    ruma::events::room::message::{OriginalRoomMessageEvent, RoomMessageEventContent},
};

use crate::config::PixivConfig;

#[tracing::instrument(name = "illust", skip_all, fields(illust_id = %illust_id), err)]
pub async fn pixiv_illust(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    pixiv: &pixrs::PixivClient,
    http: &reqwest::Client,
    illust_id: i32,
    config: &PixivConfig,
    send_r18: bool,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    crate::services::pixiv::illust::send(ev, room, pixiv, http, config, illust_id, send_r18)
        .await?;

    return Ok(None);
}
