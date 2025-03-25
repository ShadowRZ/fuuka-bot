use crate::message::{
    Injected,
    profile::{Category, ResponseType},
};
use matrix_sdk::{Room, event_handler::Ctx, ruma::events::room::message::OriginalRoomMessageEvent};

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    category: Category,
    response_type: ResponseType,
) -> anyhow::Result<()> {
    match (category, response_type) {
        (Category::Name, ResponseType::Current) => {
            self::name::current::process(ev, room, injected).await
        }
        (Category::Name, ResponseType::History) => {
            self::name::changes::process(ev, room, injected).await
        }
        (Category::Avatar, ResponseType::Current) => {
            self::avatar::current::process(ev, room, injected).await
        }
        (Category::Avatar, ResponseType::History) => {
            self::avatar::changes::process(ev, room, injected).await
        }
    }
}

pub mod avatar {
    pub mod current {
        use crate::message::Injected;
        use file_format::FileFormat;
        use matrix_sdk::{
            Room,
            event_handler::Ctx,
            media::{MediaFormat, MediaRequestParameters},
            ruma::{
                MxcUri, UInt,
                events::room::{
                    ImageInfo, MediaSource, ThumbnailInfo,
                    message::{
                        AddMentions, ForwardThread, ImageMessageEventContent, MessageType,
                        OriginalRoomMessageEvent, RoomMessageEventContent,
                    },
                },
            },
        };

        pub async fn process(
            ev: &OriginalRoomMessageEvent,
            room: &Room,
            injected: &Ctx<Injected>,
        ) -> anyhow::Result<()> {
            let _ = injected;

            use crate::RoomExt as _;
            use crate::RoomMemberExt as _;

            let client = room.client();
            let user_id = room.in_reply_to_target_fallback(ev).await?;
            let Some(member) = room.get_member(&user_id).await? else {
                return Ok(());
            };

            let content = match member.avatar_url() {
                Some(avatar_url) => {
                    let name = member.name_or_id();
                    let info = get_image_info(avatar_url, &client).await?;
                    RoomMessageEventContent::new(MessageType::Image(
                        ImageMessageEventContent::plain(
                            format!("[Avatar of {name}]"),
                            avatar_url.into(),
                        )
                        .info(Some(Box::new(info))),
                    ))
                }
                None => RoomMessageEventContent::text_plain("The user has no avatar."),
            };
            let content = content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes);
            room.send(content).await?;

            Ok(())
        }

        async fn get_image_info(
            avatar_url: &MxcUri,
            client: &matrix_sdk::Client,
        ) -> anyhow::Result<ImageInfo> {
            let request = MediaRequestParameters {
                source: MediaSource::Plain(avatar_url.into()),
                format: MediaFormat::File,
            };
            let data = client.media().get_media_content(&request, false).await?;
            let dimensions = imagesize::blob_size(&data)?;
            let (width, height) = (dimensions.width, dimensions.height);
            let format = FileFormat::from_bytes(&data);
            let mimetype = format.media_type();
            let size = data.len();
            let mut thumb = ThumbnailInfo::new();
            let width = UInt::try_from(width)?;
            let height = UInt::try_from(height)?;
            let size = UInt::try_from(size)?;
            thumb.width = Some(width);
            thumb.height = Some(height);
            thumb.mimetype = Some(mimetype.to_string());
            thumb.size = Some(size);
            let mut info = ImageInfo::new();
            info.width = Some(width);
            info.height = Some(height);
            info.mimetype = Some(mimetype.to_string());
            info.size = Some(size);
            info.thumbnail_info = Some(Box::new(thumb));
            info.thumbnail_source = Some(MediaSource::Plain(avatar_url.into()));

            Ok(info)
        }
    }

    pub mod changes {
        use crate::message::Injected;
        use futures_util::pin_mut;
        use matrix_sdk::Room;
        use matrix_sdk::event_handler::Ctx;
        use matrix_sdk::ruma::UInt;
        use matrix_sdk::ruma::events::room::member::MembershipChange;
        use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
        use matrix_sdk::ruma::events::room::message::{
            OriginalRoomMessageEvent, RoomMessageEventContent,
        };
        use time::OffsetDateTime;
        use time::format_description::well_known::Rfc3339;

        pub async fn process(
            ev: &OriginalRoomMessageEvent,
            room: &Room,
            injected: &Ctx<Injected>,
        ) -> anyhow::Result<()> {
            use crate::MxcUriExt as _;
            use crate::RoomExt as _;
            use futures_util::stream::StreamExt as _;

            let room = &room;

            let user_id = room.in_reply_to_target_fallback(ev).await?;
            let Some(member) = room.get_member(&user_id).await? else {
                return Ok(());
            };

            let media_proxy = &injected.media_proxy;
            let homeserver = {
                injected.config.borrow().matrix.homeserver.clone()
            };
            let (public_url, ttl_seconds) = {
                let config = injected.config.borrow();

                let public_url = config
                    .media_proxy
                    .as_ref()
                    .map(|cfg| cfg.public_url.clone());
                let ttl_seconds = config.media_proxy.as_ref().map(|cfg| cfg.ttl_seconds);

                (public_url, ttl_seconds)
            };
            let public_url = public_url.as_ref();

            let mut body = String::new();
            let current_avatar = member
                .avatar_url()
                .map(|uri| {
                    if let (Some(media_proxy), Some(public_url), Some(ttl_seconds)) =
                        (media_proxy, public_url, ttl_seconds)
                    {
                        media_proxy.create_media_url(public_url, uri, ttl_seconds)
                    } else {
                        uri.http_url(&homeserver)
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
                                        avatar_url.http_url(&homeserver)?
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
                                        uri.http_url(&homeserver)
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

            room.send(RoomMessageEventContent::text_plain(body).make_reply_to(
                ev,
                ForwardThread::No,
                AddMentions::Yes,
            ))
            .await?;

            Ok(())
        }
    }
}

pub mod name {
    pub mod current {
        use crate::message::Injected;
        use matrix_sdk::{
            Room,
            event_handler::Ctx,
            ruma::events::room::message::{OriginalRoomMessageEvent, RoomMessageEventContent},
        };

        pub async fn process(
            ev: &OriginalRoomMessageEvent,
            room: &Room,
            injected: &Ctx<Injected>,
        ) -> anyhow::Result<()> {
            let _ = injected;
            use crate::RoomExt as _;

            let user_id = room.in_reply_to_target_fallback(ev).await?;
            let Some(member) = room.get_member(&user_id).await? else {
                return Ok(());
            };

            room.send(RoomMessageEventContent::text_plain(member.name()));

            Ok(())
        }
    }

    pub mod changes {
        use futures_util::pin_mut;
        use matrix_sdk::Room;
        use matrix_sdk::event_handler::Ctx;
        use matrix_sdk::ruma::UInt;
        use matrix_sdk::ruma::events::room::member::MembershipChange;
        use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
        use matrix_sdk::ruma::events::room::message::{
            OriginalRoomMessageEvent, RoomMessageEventContent,
        };
        use time::OffsetDateTime;
        use time::format_description::well_known::Rfc3339;

        use crate::message::Injected;

        pub async fn process(
            ev: &OriginalRoomMessageEvent,
            room: &Room,
            injected: &Ctx<Injected>,
        ) -> anyhow::Result<()> {
            let _ = injected;
            let _ = ev;
            use crate::RoomExt as _;
            use futures_util::stream::StreamExt as _;

            let room = &room;

            let user_id = room.in_reply_to_target_fallback(ev).await?;
            let Some(member) = room.get_member(&user_id).await? else {
                return Ok(());
            };

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

            room.send(RoomMessageEventContent::text_plain(body).make_reply_to(
                ev,
                ForwardThread::No,
                AddMentions::Yes,
            ))
            .await?;

            Ok(())
        }
    }
}
