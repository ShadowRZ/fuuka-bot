use anyhow::Context;
use file_format::FileFormat;
use futures_util::pin_mut;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use matrix_sdk::deserialized_responses::MemberEvent;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequest;
use matrix_sdk::reqwest::Url;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::ImageMessageEventContent;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::ImageInfo;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::ThumbnailInfo;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::ruma::MxcUri;
use matrix_sdk::ruma::UInt;
use matrix_sdk::Client;
use std::io::Cursor;
use time::format_description::well_known::Rfc3339;
use time::macros::offset;
use time::Duration;
use time::OffsetDateTime;
use time::Weekday;
use tokio_stream::StreamExt;

use crate::member_updates::MemberChanges;
use crate::utils::avatar_http_url;
use crate::utils::get_reply_target_fallback;

pub async fn fuuka_bot_dispatch_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
    command: &str,
    homeserver: Url,
) -> anyhow::Result<()> {
    let args: Vec<&str> = command.split_ascii_whitespace().collect();
    if let Some(command) = args.first() {
        match *command {
            "help" => help_command(ev, room).await?,
            "send_avatar" => send_avatar_command(ev, room)
                .await
                .context("Sending avatar failed")?,
            "crazy_thursday" => crazy_thursday_command(ev, room).await?,
            "ping" => ping_command(ev, room).await?,
            "room_id" => room_id_command(ev, room).await?,
            "user_id" => user_id_command(ev, room).await?,
            "name_changes" => name_changes_command(ev, room).await?,
            "avatar_changes" => avatar_changes_command(ev, room, homeserver).await?,
            _ => _unknown_command(ev, room, command).await?,
        }
    }

    Ok(())
}

async fn _unknown_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
    command: &str,
) -> anyhow::Result<()> {
    let content = RoomMessageEventContent::text_plain(format!("Unknown command {command}."))
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn help_command(ev: OriginalSyncRoomMessageEvent, room: Joined) -> anyhow::Result<()> {
    let client = room.client();
    let user_id = client.user_id().unwrap();
    let body = format!("Fuuka Bot\nUser ID: {user_id}");

    let content = RoomMessageEventContent::text_plain(body)
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn crazy_thursday_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
) -> anyhow::Result<()> {
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
    let content = RoomMessageEventContent::text_plain(body)
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn ping_command(ev: OriginalSyncRoomMessageEvent, room: Joined) -> anyhow::Result<()> {
    let now = MilliSecondsSinceUnixEpoch::now().0;
    let event_ts = ev.origin_server_ts.0;
    let delta: i64 = (now - event_ts).into();
    let duration = Duration::milliseconds(delta);
    let body = format!("Pong after {duration:.8}");

    let content = RoomMessageEventContent::text_plain(body)
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn room_id_command(ev: OriginalSyncRoomMessageEvent, room: Joined) -> anyhow::Result<()> {
    let content = RoomMessageEventContent::text_plain(room.room_id())
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn user_id_command(ev: OriginalSyncRoomMessageEvent, room: Joined) -> anyhow::Result<()> {
    let user_id = get_reply_target_fallback(&ev, &room).await?;

    let content = RoomMessageEventContent::text_plain(user_id.as_str())
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    Ok(())
}

async fn name_changes_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
) -> anyhow::Result<()> {
    let user_id = get_reply_target_fallback(&ev, &room).await?;

    let member = room.get_member(&user_id).await?;
    if let Some(member) = member {
        let mut body = String::new();
        let current_name = member.display_name().unwrap_or("(None)");
        let result = format!("Current Name: {current_name}\n");
        body.push_str(&result);
        let mut count: i32 = 0;

        let event: &MemberEvent = member.event();
        match event {
            MemberEvent::Sync(event) => {
                let stream = MemberChanges::new_stream(&room, event.clone()).take(4);
                pin_mut!(stream);
                while let Some(event) = stream.next().await {
                    // `MembershipChange::Joined` because API can only return the current state.
                    if let MembershipChange::Joined = event.membership_change() {
                        match event.content.displayname {
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
                }
            }
            _ => tracing::warn!(
                "INTERNAL ERROR: A member event in a joined room should not be stripped."
            ),
        }

        let content = RoomMessageEventContent::text_plain(body)
            .make_reply_to(&ev.into_full_event(room.room_id().into()));
        room.send(content, None).await?;
    }

    Ok(())
}

async fn avatar_changes_command(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
    homeserver: Url,
) -> anyhow::Result<()> {
    let user_id = get_reply_target_fallback(&ev, &room).await?;

    let member = room.get_member(&user_id).await?;
    if let Some(member) = member {
        let mut body = String::new();
        let current_avatar = avatar_http_url(member.avatar_url(), &homeserver)?
            .map(|result| result.to_string())
            .unwrap_or("(None)".to_string());
        let result = format!("Current Avatar: {current_avatar}\n");
        body.push_str(&result);
        let mut count: i32 = 0;

        let event: &MemberEvent = member.event();
        match event {
            MemberEvent::Sync(event) => {
                let stream = MemberChanges::new_stream(&room, event.clone()).take(4);
                pin_mut!(stream);
                while let Some(event) = stream.next().await {
                    // `MembershipChange::Joined` because API can only return the current state.
                    if let MembershipChange::Joined = event.membership_change() {
                        match event.content.avatar_url {
                            Some(avatar_url) => {
                                count -= 1;
                                let nanos: i128 =
                                    <UInt as Into<i128>>::into(event.origin_server_ts.0) * 1000000;
                                let timestamp = OffsetDateTime::from_unix_timestamp_nanos(nanos)?
                                    .format(&Rfc3339)?;
                                let avatar_link =
                                    avatar_http_url(Some(&avatar_url), &homeserver)?.unwrap();
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
                }
            }
            _ => tracing::warn!(
                "INTERNAL ERROR: A member event in a joined room should not be stripped."
            ),
        }

        let content = RoomMessageEventContent::text_plain(body)
            .make_reply_to(&ev.into_full_event(room.room_id().into()));
        room.send(content, None).await?;
    }

    Ok(())
}

async fn send_avatar_command(ev: OriginalSyncRoomMessageEvent, room: Joined) -> anyhow::Result<()> {
    let target = get_reply_target_fallback(&ev, &room).await?;
    if let Some(member) = room.get_member(&target).await? {
        if let Some(avatar_url) = member.avatar_url() {
            let name = member.display_name().unwrap_or(target.as_str());
            let info = get_image_info(avatar_url, &room.client()).await?;
            let content =
                RoomMessageEventContent::new(MessageType::Image(ImageMessageEventContent::plain(
                    format!("[Avatar of {name}]"),
                    avatar_url.into(),
                    Some(Box::new(info)),
                )));
            let content = content.make_reply_to(&ev.into_full_event(room.room_id().into()));
            room.send(content, None).await?;
        } else {
            let content = RoomMessageEventContent::text_plain("The user has no avatar.")
                .make_reply_to(&ev.into_full_event(room.room_id().into()));
            room.send(content, None).await?;
        }
    }
    Ok(())
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
