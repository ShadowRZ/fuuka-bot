//! Extracts Pixiv URLs.
use matrix_sdk::{
    Room,
    ruma::events::room::message::{OriginalRoomMessageEvent, RoomMessageEventContent},
};

#[tracing::instrument(name = "illust", skip_all, fields(fuuka_bot.pixiv.illust_id = %illust_id), err)]
pub async fn pixiv_illust(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    pixiv: &pixiv_ajax_api::PixivClient,
    http: &reqwest::Client,
    illust_id: i32,
    context: &crate::services::pixiv::Context,
    send_r18: bool,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    crate::services::pixiv::illust::send(ev, room, pixiv, http, context, illust_id, send_r18)
        .await?;

    return Ok(None);
}
