use std::sync::Arc;

use futures_util::pin_mut;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::UInt;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::{Config, MediaProxy};

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::AvatarChanges(member)].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>,
         member: Arc<RoomMember>,
         config: Arc<Config>,
         media_proxy: Option<Arc<MediaProxy>>| async move {
            use crate::RoomExt as _;
            use crate::MxcUriExt as _;
            use futures_util::stream::StreamExt as _;

            let media_proxy = &media_proxy;
            let homeserver = &config.matrix.homeserver;
            let public_url = config.media_proxy.as_ref().map(|cfg| &cfg.public_url);
            let ttl_seconds = config.media_proxy.as_ref().map(|cfg| cfg.ttl_seconds);

            let mut body = String::from(
                "WARN: If unauthenticated media is frozen on the server, these URLs may not work!\n",
            );
            let current_avatar = member
                .avatar_url()
                .map(|uri| {
                    if let (Some(media_proxy), Some(public_url), Some(ttl_seconds)) =
                        (media_proxy, public_url, ttl_seconds)
                    {
                        media_proxy.create_media_url(public_url, uri, ttl_seconds)
                    } else {
                        uri.http_url(homeserver)
                    }
                })
                .transpose()?
                .map(|result| result.to_string())
                .unwrap_or("(None)".to_string());
            let result = format!("Current Avatar: {current_avatar}\n");
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
                            displayname_change: _,
                            avatar_url_change,
                        } => {
                            let Some(avatar_url_change) = avatar_url_change else {
                                continue;
                            };
                            match avatar_url_change.new {
                                Some(avatar_url) => {
                                    count -= 1;
                                    let nanos: i128 =
                                        <UInt as Into<i128>>::into(event.origin_server_ts.0)
                                            * 1000000;
                                    let timestamp =
                                        OffsetDateTime::from_unix_timestamp_nanos(nanos)?
                                            .format(&Rfc3339)?;
                                    let avatar_link = if let (
                                        Some(media_proxy),
                                        Some(public_url),
                                        Some(ttl_seconds),
                                    ) = (media_proxy, public_url, ttl_seconds)
                                    {
                                        media_proxy.create_media_url(
                                            public_url,
                                            avatar_url,
                                            ttl_seconds,
                                        )?
                                    } else {
                                        avatar_url.http_url(homeserver)?
                                    };
                                    let result = format!(
                                        "{count}: Changed to {avatar_link} ({timestamp})\n"
                                    );
                                    body.push_str(&result);
                                }
                                None => {
                                    let result = format!("{count}: Removed avatar.\n");
                                    body.push_str(&result);
                                }
                            }
                        }
                        MembershipChange::Joined => {
                            count -= 1;
                            let avatar_link = event
                                .content
                                .avatar_url
                                .map(|uri| {
                                    if let (
                                        Some(media_proxy),
                                        Some(public_url),
                                        Some(ttl_seconds),
                                    ) = (media_proxy, public_url, ttl_seconds)
                                    {
                                        media_proxy.create_media_url(public_url, &uri, ttl_seconds)
                                    } else {
                                        uri.http_url(homeserver)
                                    }
                                })
                                .transpose()?;
                            let result = format!(
                                "{count}: Joined with avatar {}\n",
                                avatar_link
                                    .map(|link| link.to_string())
                                    .unwrap_or("(No avatar)".to_string())
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
