use std::sync::Arc;

use futures_util::pin_mut;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::ruma::UInt;
use matrix_sdk::Room;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::NameChanges(member)].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>, room: Arc<Room>, member: Arc<RoomMember>| async move {
            use crate::RoomExt as _;
            use futures_util::stream::StreamExt as _;

            let mut body = String::new();
            let current_name = member.display_name().unwrap_or("(None)");
            let result = format!("Current Name: {current_name}\n");
            body.push_str(&result);
            let mut count: i32 = 0;

            {
                let stream = room.get_member_membership_changes(&member).peekable();
                pin_mut!(stream);
                while let Some(event) = stream.next().await {
                    if count <= -5 {
                        break;
                    }

                    let prev_event = stream.as_mut().peek().await;
                    let detail = prev_event.map(|e| e.content.details());
                    let change =
                        event
                            .content
                            .membership_change(detail, &event.sender, &event.state_key);
                    match change {
                        MembershipChange::ProfileChanged {
                            displayname_change,
                            avatar_url_change: _,
                        } => {
                            let Some(displayname_change) = displayname_change else {
                                continue;
                            };
                            match displayname_change.new {
                                Some(displayname) => {
                                    count -= 1;
                                    let nanos: i128 =
                                        <UInt as Into<i128>>::into(event.origin_server_ts.0)
                                            * 1000000;
                                    let timestamp =
                                        OffsetDateTime::from_unix_timestamp_nanos(nanos)?
                                            .format(&Rfc3339)?;
                                    let result = format!(
                                        "{count}: Changed to {displayname} ({timestamp})\n"
                                    );
                                    body.push_str(&result);
                                }
                                None => {
                                    let result = format!("{count}: Removed display name.\n");
                                    body.push_str(&result);
                                }
                            }
                        }
                        MembershipChange::Joined => {
                            count -= 1;
                            let result = format!(
                                "{count}: Joined with display name {}\n",
                                event.content.displayname.unwrap_or("(No name)".to_string())
                            );
                            body.push_str(&result);
                        }
                        _ => {}
                    };
                }
            }

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(body).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
