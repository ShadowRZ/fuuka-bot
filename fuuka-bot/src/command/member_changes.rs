use matrix_sdk::deserialized_responses::MemberEvent;
use ruma::MilliSecondsSinceUnixEpoch;

use crate::Context;

use futures_util::pin_mut;
use futures_util::StreamExt;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::UInt;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::functions::member_stream::member_state_stream;
use crate::MxcUriExt;

impl Context {
    #[tracing::instrument(
        skip(self, member),
        fields(
            user_id = %member.user_id(),
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _avatar_changes(
        &self,
        member: RoomMember,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        use matrix_sdk::event_handler::Ctx;

        let homeserver = &self.homeserver;
        let media_proxy = &self.media_proxy;
        let public_url = self.config.media_proxy.as_ref().map(|cfg| &cfg.public_url);

        let mut body = String::from(
            "WARN: If unauthenticated media is frozen on the server, these URLs may not work!\n",
        );
        let current_avatar = member
            .avatar_url()
            .map(|uri| {
                if let (Ctx(Some(media_proxy)), Some(public_url)) = (media_proxy, public_url) {
                    media_proxy.create_media_url(public_url, uri, MilliSecondsSinceUnixEpoch::now())
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

        let event: &MemberEvent = member.event();
        match event {
            MemberEvent::Sync(event) => {
                let stream = member_state_stream(&self.room, event.clone()).peekable();
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
                                    let avatar_link =
                                        if let (Ctx(Some(media_proxy)), Some(public_url)) =
                                            (media_proxy, public_url)
                                        {
                                            media_proxy.create_media_url(
                                                public_url,
                                                avatar_url,
                                                MilliSecondsSinceUnixEpoch::now(),
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
                                    if let (Ctx(Some(media_proxy)), Some(public_url)) =
                                        (media_proxy, public_url)
                                    {
                                        media_proxy.create_media_url(
                                            public_url,
                                            &uri,
                                            MilliSecondsSinceUnixEpoch::now(),
                                        )
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
            _ => tracing::warn!(
                "INTERNAL ERROR: A member event in a joined room should not be stripped."
            ),
        }

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(body),
        )))
    }

    #[tracing::instrument(
        skip(self, member),
        fields(
            user_id = %member.user_id(),
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _name_changes(
        &self,
        member: RoomMember,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let mut body = String::new();
        let current_name = member.display_name().unwrap_or("(None)");
        let result = format!("Current Name: {current_name}\n");
        body.push_str(&result);
        let mut count: i32 = 0;

        let event: &MemberEvent = member.event();
        match event {
            MemberEvent::Sync(event) => {
                let stream = member_state_stream(&self.room, event.clone()).peekable();
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
            _ => tracing::warn!(
                "INTERNAL ERROR: A member event in a joined room should not be stripped."
            ),
        }

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(body),
        )))
    }
}
