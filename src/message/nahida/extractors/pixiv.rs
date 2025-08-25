//! Extracts Pixiv URLs.
use matrix_sdk::ruma::{RoomId, events::room::message::RoomMessageEventContent};

use crate::config::PixivConfig;

#[tracing::instrument(name = "illust", skip(pixiv, config, room_id), err)]
pub async fn pixiv_illust(
    pixiv: &pixrs::PixivClient,
    artwork_id: i32,
    config: &PixivConfig,
    room_id: &RoomId,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp = pixiv.illust_info(artwork_id).with_lang("zh").await?;
    let send_r18 = config.r18;
    Ok(crate::services::pixiv::illust::format(
        resp, config, send_r18, room_id, true,
    ))
}
