//! Bot commands handler.

use anyhow::Context;
use file_format::FileFormat;
use futures_util::pin_mut;
use futures_util::StreamExt;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use matrix_sdk::deserialized_responses::MemberEvent;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequest;
use matrix_sdk::reqwest::Url;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::AddMentions;
use matrix_sdk::ruma::events::room::message::ForwardThread;
use matrix_sdk::ruma::events::room::message::ImageMessageEventContent;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::ImageInfo;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::ThumbnailInfo;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::ruma::MxcUri;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::UInt;
use matrix_sdk::Client;
use std::io::Cursor;
use time::format_description::well_known::Rfc3339;
use time::macros::offset;
use time::Duration;
use time::OffsetDateTime;
use time::Weekday;

use crate::member_updates::MemberChanges;
use crate::utils::avatar_http_url;
use crate::utils::get_reply_target;
use crate::utils::get_reply_target_fallback;
use crate::utils::make_divergence;
use crate::FuukaBotError;

/// Dispatches the command and send the command outout.
pub async fn fuuka_bot_dispatch_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    command: &str,
    homeserver: Url,
) -> anyhow::Result<()> {
    let args: Vec<&str> = command.split_ascii_whitespace().collect();
    let Some(command) = args.first() else {
        return Ok(());
    };

    let Some(content) = (match *command {
        "help" => help_command(&room).await?,
        "send_avatar" => send_avatar_command(&ev, &room)
            .await
            .context("Sending avatar failed")?,
        "crazy_thursday" => crazy_thursday_command().await?,
        "ping" => ping_command(&ev).await?,
        "room_id" => room_id_command(&room).await?,
        "user_id" => user_id_command(&ev, &room).await?,
        "name_changes" => name_changes_command(&ev, &room).await?,
        "avatar_changes" => avatar_changes_command(&ev, &room, &homeserver).await?,
        "divergence" => divergence_command(&ev, &room).await?,
        "ignore" => ignore_command(&ev, &room).await?,
        "unignore" => unignore_command(&ev, &room, args.get(1).copied()).await?,
        _ => _unknown_command(command).await?,
    }) else {
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

async fn _unknown_command(command: &str) -> anyhow::Result<Option<RoomMessageEventContent>> {
    Ok(Some(RoomMessageEventContent::text_plain(format!(
        "Unknown command {command}."
    ))))
}

async fn help_command(room: &Room) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let client = room.client();
    let user_id = client.user_id().unwrap();

    Ok(Some(RoomMessageEventContent::text_html(
        format!("Fuuka Bot - User ID: {user_id}\nCommand reference: https://github.com/ShadowRZ/fuuka-bot/blob/master/COMMANDS.md"),
        format!("Fuuka Bot - User ID: {user_id}<br/>Command reference: https://github.com/ShadowRZ/fuuka-bot/blob/master/COMMANDS.md"),
    )))
}

async fn crazy_thursday_command() -> anyhow::Result<Option<RoomMessageEventContent>> {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let body = if now.weekday() != Weekday::Thursday {
        let date = now.date().next_occurrence(time::Weekday::Thursday);
        let target = date.with_hms(0, 0, 0)?.assume_offset(offset!(+8));
        let dur = target - now;
        {
            let whole_seconds = dur.whole_seconds().unsigned_abs();
            let seconds = whole_seconds % 60;
            let whole_minutes = dur.whole_minutes().unsigned_abs();
            let minutes = whole_minutes % 60;
            let whole_hours = dur.whole_hours().unsigned_abs();
            let hours = whole_hours % 24;
            let days = dur.whole_days();
            format!("Time until next thursday ({date}): {days} days, {hours:0>2}:{minutes:0>2}:{seconds:0>2}")
        }
    } else {
        "Crazy Thursday!".to_string()
    };

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

async fn ping_command(
    ev: &OriginalSyncRoomMessageEvent,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let now = MilliSecondsSinceUnixEpoch::now().0;
    let event_ts = ev.origin_server_ts.0;
    let delta: i64 = (now - event_ts).into();
    let duration = Duration::milliseconds(delta);
    let body = format!("Pong after {duration:.8}");

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

async fn room_id_command(room: &Room) -> anyhow::Result<Option<RoomMessageEventContent>> {
    Ok(Some(RoomMessageEventContent::text_plain(room.room_id())))
}

async fn user_id_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = get_reply_target_fallback(ev, room).await?;

    Ok(Some(RoomMessageEventContent::text_plain(user_id.as_str())))
}

async fn name_changes_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = get_reply_target_fallback(ev, room).await?;

    let Some(member) = room.get_member(&user_id).await? else {
        return Err(FuukaBotError::ShouldAvaliable)?;
    };

    let mut body = String::new();
    let current_name = member.display_name().unwrap_or("(None)");
    let result = format!("Current Name: {current_name}\n");
    body.push_str(&result);
    let mut count: i32 = 0;

    let event: &MemberEvent = member.event();
    match event {
        MemberEvent::Sync(event) => {
            let stream = MemberChanges::new_stream(room, event.clone()).peekable();
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
                                    <UInt as Into<i128>>::into(event.origin_server_ts.0) * 1000000;
                                let timestamp = OffsetDateTime::from_unix_timestamp_nanos(nanos)?
                                    .format(&Rfc3339)?;
                                let result =
                                    format!("{count}: Changed to {displayname} ({timestamp})\n");
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

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

async fn avatar_changes_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
    homeserver: &Url,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = get_reply_target_fallback(ev, room).await?;

    let Some(member) = room.get_member(&user_id).await? else {
        return Err(FuukaBotError::ShouldAvaliable)?;
    };

    let mut body = String::new();
    let current_avatar = avatar_http_url(member.avatar_url(), homeserver)?
        .map(|result| result.to_string())
        .unwrap_or("(None)".to_string());
    let result = format!("Current Avatar: {current_avatar}\n");
    body.push_str(&result);
    let mut count: i32 = 0;

    let event: &MemberEvent = member.event();
    match event {
        MemberEvent::Sync(event) => {
            let stream = MemberChanges::new_stream(room, event.clone()).peekable();
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
                                    <UInt as Into<i128>>::into(event.origin_server_ts.0) * 1000000;
                                let timestamp = OffsetDateTime::from_unix_timestamp_nanos(nanos)?
                                    .format(&Rfc3339)?;
                                let avatar_link =
                                    avatar_http_url(Some(avatar_url), homeserver)?.unwrap();
                                let result =
                                    format!("{count}: Changed to {avatar_link} ({timestamp})\n");
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
                        let avatar_link =
                            avatar_http_url(event.content.avatar_url.as_deref(), homeserver)?;
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

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

async fn send_avatar_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let target = get_reply_target_fallback(ev, room).await?;

    let Some(member) = room.get_member(&target).await? else {
        return Err(FuukaBotError::RequiresReply)?;
    };

    match member.avatar_url() {
        Some(avatar_url) => {
            let name = member.display_name().unwrap_or(target.as_str());
            let info = get_image_info(avatar_url, &room.client()).await?;
            Ok(Some(RoomMessageEventContent::new(MessageType::Image(
                ImageMessageEventContent::plain(format!("[Avatar of {name}]"), avatar_url.into())
                    .info(Some(Box::new(info))),
            ))))
        }
        None => Ok(Some(RoomMessageEventContent::text_plain(
            "The user has no avatar.",
        ))),
    }
}

async fn get_image_info(avatar_url: &MxcUri, client: &Client) -> anyhow::Result<ImageInfo> {
    let request = MediaRequest {
        source: MediaSource::Plain(avatar_url.into()),
        format: MediaFormat::File,
    };
    let data = client.media().get_media_content(&request, false).await?;
    let image = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()?
        .decode()?;
    let (width, height) = image.dimensions();
    let format = FileFormat::from_bytes(&data);
    let mimetype = format.media_type();
    let size = data.len();
    let mut thumb = ThumbnailInfo::new();
    thumb.width = UInt::new(width.into());
    thumb.height = UInt::new(height.into());
    thumb.mimetype = Some(mimetype.to_string());
    thumb.size = UInt::new(size.try_into().unwrap_or(u64::MAX));
    let mut info = ImageInfo::new();
    info.width = UInt::new(width.into());
    info.height = UInt::new(height.into());
    info.mimetype = Some(mimetype.to_string());
    info.size = UInt::new(size.try_into().unwrap_or(u64::MAX));
    info.thumbnail_info = Some(Box::new(thumb));
    info.thumbnail_source = Some(MediaSource::Plain(avatar_url.into()));

    Ok(info)
}

async fn divergence_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let room_hash = crc32fast::hash(room.room_id().as_bytes());
    let event_id_hash = match &ev.content.relates_to {
        Some(Relation::Reply { in_reply_to }) => {
            let event_id = &in_reply_to.event_id;
            Some(crc32fast::hash(event_id.as_bytes()))
        }
        _ => None,
    };
    let hash = make_divergence(room_hash, event_id_hash);
    Ok(Some(RoomMessageEventContent::text_plain(format!(
        "{hash:.6}%"
    ))))
}

async fn ignore_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let sender = &ev.sender;

    let Some(target) = get_reply_target(ev, room).await? else {
        return Err(FuukaBotError::RequiresReply)?;
    };

    if room.can_user_ban(sender).await? {
        let member = room
            .get_member(&target)
            .await?
            .ok_or(FuukaBotError::ShouldAvaliable)?;
        member.ignore().await?;
        Ok(Some(RoomMessageEventContent::text_plain(format!(
            "Ignored {} ({})",
            member.display_name().unwrap_or("(No Name)"),
            sender
        ))))
    } else {
        Err(FuukaBotError::RequiresBannable)?
    }
}

async fn unignore_command(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
    user: Option<&str>,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let sender = &ev.sender;

    let target = match user {
        Some(user) => OwnedUserId::try_from(user)?,
        None => return Err(FuukaBotError::MissingParamter("user"))?,
    };

    if room.can_user_ban(sender).await? {
        let member = room
            .get_member(&target)
            .await?
            .ok_or(FuukaBotError::UserNotFound)?;
        member.unignore().await?;
        Ok(Some(RoomMessageEventContent::text_plain(format!(
            "Unignored {} ({})",
            member.display_name().unwrap_or("(No Name)"),
            sender
        ))))
    } else {
        Err(FuukaBotError::RequiresBannable)?
    }
}
