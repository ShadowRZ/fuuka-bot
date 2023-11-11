use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;

use crate::{jerryxiao::make_jerryxiao_event_content, utils::get_reply_target};

pub struct FuukaBotMessages;

impl FuukaBotMessages {
    pub async fn jerryxiao(ev: OriginalSyncRoomMessageEvent, room: Room) -> anyhow::Result<()> {
        if let Room::Joined(room) = room {
            let body = remove_plain_reply_fallback(ev.content.body()).trim();
            let mut splited = body.split_whitespace();
            // If the first part of the message is pure ASCII, skip it
            if splited.next().unwrap().is_ascii() {
                return Ok(());
            };
            let from_sender = &ev.sender;
            if let Some(remaining) = body.strip_prefix('/') {
                if let Some(to_sender) = get_reply_target(&ev, &room).await? {
                    let content = make_jerryxiao_event_content(
                        from_sender,
                        &to_sender,
                        remaining,
                        &room,
                        false,
                    )
                    .await?
                    .make_reply_to(&ev.into_full_event(room.room_id().into()));
                    room.send(content, None).await?;
                }
            } else if let Some(remaining) = body.strip_prefix("!!") {
                if let Some(to_sender) = get_reply_target(&ev, &room).await? {
                    let content = make_jerryxiao_event_content(
                        from_sender,
                        &to_sender,
                        remaining,
                        &room,
                        false,
                    )
                    .await?
                    .make_reply_to(&ev.into_full_event(room.room_id().into()));
                    room.send(content, None).await?;
                }
            } else if let Some(remaining) = body.strip_prefix('\\') {
                if let Some(to_sender) = get_reply_target(&ev, &room).await? {
                    let content = make_jerryxiao_event_content(
                        from_sender,
                        &to_sender,
                        remaining,
                        &room,
                        true,
                    )
                    .await?
                    .make_reply_to(&ev.into_full_event(room.room_id().into()));
                    room.send(content, None).await?;
                }
            } else if let Some(remaining) = body.strip_prefix("¡¡") {
                if let Some(to_sender) = get_reply_target(&ev, &room).await? {
                    let content = make_jerryxiao_event_content(
                        from_sender,
                        &to_sender,
                        remaining,
                        &room,
                        true,
                    )
                    .await?
                    .make_reply_to(&ev.into_full_event(room.room_id().into()));
                    room.send(content, None).await?;
                }
            }
        }

        Ok(())
    }
}
