//! Responses to messages that are not commands.

use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::UserId;
use matrix_sdk::RoomState;
use ruma::events::Mentions;

use crate::dicer::DiceCandidate;
use crate::jerryxiao::make_randomdraw_event_content;
use crate::utils::nom_error_message;
use crate::{jerryxiao::make_jerryxiao_event_content, utils::get_reply_target};

/// A ZST for containing messages responses.
pub struct FuukaBotMessages;

impl FuukaBotMessages {
    /// The callback handler for Jerry Xiao functions.
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

    /// The callback handler for randomdraw.
    pub async fn randomdraw(ev: OriginalSyncRoomMessageEvent, room: Room) -> anyhow::Result<()> {
        // It should be a joined room.
        if room.state() != RoomState::Joined {
            return Ok(());
        }

        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        let Some(content) = _dispatch_randomdraw(&ev, &room, body).await? else {
            return Ok(());
        };

        let content = content.make_reply_to(
            &ev.into_full_event(room.room_id().into()),
            ForwardThread::Yes,
            AddMentions::Yes,
        );
        room.send(content).await?;
        Ok(())
    }

    /// The callback handler for dicer.
    pub async fn dicer(ev: OriginalSyncRoomMessageEvent, room: Room) -> anyhow::Result<()> {
        // It should be a joined room.
        if room.state() != RoomState::Joined {
            return Ok(());
        }
        let client = room.client();
        let user_id = client.user_id().unwrap();
        if ev.sender == user_id {
            return Ok(());
        }

        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        let Some(content) = _dispatch_dicer(body).await? else {
            return Ok(());
        };

        let content = content.make_reply_to(
            &ev.into_full_event(room.room_id().into()),
            ForwardThread::Yes,
            AddMentions::Yes,
        );
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

async fn _dispatch_dicer(body: &str) -> anyhow::Result<Option<RoomMessageEventContent>> {
    if let Some(expr) = body.strip_prefix("@=") {
        let expr = expr.trim();
        let cand = match expr.parse::<DiceCandidate>() {
            Ok(cand) => cand,
            Err(e) => {
                return Ok(Some(nom_error_message(expr, e)));
            }
        };
        let result = cand.expr.eval()?;
        let string = match cand.target {
            Some(target) => {
                if result < (target as i32) {
                    Some("Success")
                } else {
                    Some("Failed")
                }
            }
            None => None,
        };
        Ok(Some(RoomMessageEventContent::text_html(
            format!(
                "{}{}",
                result,
                string.map(|s| format!(" ({s})")).unwrap_or("".to_string())
            ),
            format!(
                "{}{}",
                result,
                string
                    .map(|s| format!(" <b>({s})</b>"))
                    .unwrap_or("".to_string())
            ),
        )))
    } else {
        Ok(None)
    }
}
