use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::RoomState;
use ruma::events::room::message::RoomMessageEventContent;

use crate::jerryxiao::make_randomdraw_event_content;
use crate::{jerryxiao::make_jerryxiao_event_content, utils::get_reply_target};

pub struct FuukaBotMessages;

impl FuukaBotMessages {
    pub async fn jerryxiao(ev: OriginalSyncRoomMessageEvent, room: Room) -> anyhow::Result<()> {
        // It should be a joined room.
        if room.state() != RoomState::Joined {
            return Ok(());
        }

        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        let mut splited = body.split_whitespace();
        // If the first part of the message is pure ASCII, skip it
        if splited.next().unwrap().is_ascii() {
            return Ok(());
        };
        let content = _dispatch_jerryxiao(&ev, &room, body).await?;
        if let Some(content) = content {
            let content = content.make_reply_to(
                &ev.into_full_event(room.room_id().into()),
                ForwardThread::Yes,
                AddMentions::Yes,
            );
            room.send(content).await?;
        }
        Ok(())
    }

    pub async fn randomdraw(ev: OriginalSyncRoomMessageEvent, room: Room) -> anyhow::Result<()> {
        // It should be a joined room.
        if room.state() != RoomState::Joined {
            return Ok(());
        }

        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        let content = _dispatch_randomdraw(&ev, &room, body).await?;
        if let Some(content) = content {
            let content = content.make_reply_to(
                &ev.into_full_event(room.room_id().into()),
                ForwardThread::Yes,
                AddMentions::Yes,
            );
            room.send(content).await?;
        }
        Ok(())
    }
}

async fn _dispatch_jerryxiao(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
    body: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let from_sender = &ev.sender;
    if let Some(to_sender) = get_reply_target(ev, room).await? {
        if let Some(remaining) = body.strip_prefix('/') {
            Ok(Some(
                make_jerryxiao_event_content(room, from_sender, &to_sender, remaining, false)
                    .await?,
            ))
        } else if let Some(remaining) = body.strip_prefix("!!") {
            Ok(Some(
                make_jerryxiao_event_content(room, from_sender, &to_sender, remaining, false)
                    .await?,
            ))
        } else if let Some(remaining) = body.strip_prefix('\\') {
            Ok(Some(
                make_jerryxiao_event_content(room, from_sender, &to_sender, remaining, true)
                    .await?,
            ))
        } else if let Some(remaining) = body.strip_prefix("¡¡") {
            Ok(Some(
                make_jerryxiao_event_content(room, from_sender, &to_sender, remaining, true)
                    .await?,
            ))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

async fn _dispatch_randomdraw(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
    body: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = &ev.sender;
    if let Some(remaining) = body.strip_prefix("@@") {
        Ok(Some(
            make_randomdraw_event_content(room, user_id, remaining, false).await?,
        ))
    } else if let Some(remaining) = body.strip_prefix("@%") {
        Ok(Some(
            make_randomdraw_event_content(room, user_id, remaining, true).await?,
        ))
    } else {
        Ok(None)
    }
}
