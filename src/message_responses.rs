use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::UserId;
use matrix_sdk::RoomState;
use ruma::events::Mentions;

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

        let from_sender = &ev.sender;
        let Some(to_sender) = get_reply_target(&ev, &room).await? else {
            return Ok(());
        };

        let Some(content) = _dispatch_jerryxiao(&room, body, from_sender, &to_sender).await? else {
            return Ok(());
        };

        let content = content
            .make_reply_to(
                &ev.into_full_event(room.room_id().into()),
                ForwardThread::Yes,
                AddMentions::Yes,
            )
            .add_mentions(Mentions::with_user_ids([to_sender]));
        room.send(content).await?;

        Ok(())
    }
}

async fn _dispatch_jerryxiao(
    room: &Room,
    body: &str,
    from_sender: &UserId,
    to_sender: &UserId,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    if let Some(remaining) = body.strip_prefix('/') {
        Ok(Some(
            make_jerryxiao_event_content(room, from_sender, to_sender, remaining, false).await?,
        ))
    } else if let Some(remaining) = body.strip_prefix("!!") {
        Ok(Some(
            make_jerryxiao_event_content(room, from_sender, to_sender, remaining, false).await?,
        ))
    } else if let Some(remaining) = body.strip_prefix('\\') {
        Ok(Some(
            make_jerryxiao_event_content(room, from_sender, to_sender, remaining, true).await?,
        ))
    } else if let Some(remaining) = body.strip_prefix("¡¡") {
        Ok(Some(
            make_jerryxiao_event_content(room, from_sender, to_sender, remaining, true).await?,
        ))
    } else {
        Ok(None)
    }
}
