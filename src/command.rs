//! Bot commands handler.
#![warn(missing_docs)]
use file_format::FileFormat;
use futures_util::pin_mut;
use futures_util::StreamExt;
use matrix_sdk::deserialized_responses::MemberEvent;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequest;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::AddMentions;
use matrix_sdk::ruma::events::room::message::ForwardThread;
use matrix_sdk::ruma::events::room::message::ImageMessageEventContent;
use matrix_sdk::ruma::events::room::message::MessageType;
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
use time::format_description::well_known::Rfc3339;
use time::macros::offset;
use time::Duration;
use time::OffsetDateTime;
use time::Weekday;

use crate::get_reply_target;
use crate::get_reply_target_fallback;
use crate::stream::StreamFactory;
use crate::traits::MxcUriExt;
use crate::types::HitokotoResult;
use crate::BotContext;
use crate::Error;
use crate::HandlerContext;

/// Dispatches the command and send the command outout.
pub async fn dispatch(
    bot_ctx: &BotContext,
    ctx: &HandlerContext,
    command: &str,
) -> anyhow::Result<()> {
    let args: Vec<&str> = command.split_ascii_whitespace().collect();
    let Some(command) = args.first() else {
        return Ok(());
    };

    let Some(content) = (match *command {
        "help" => help(ctx).await?,
        "send_avatar" => send_avatar(ctx).await?,
        "crazy_thursday" => crazy_thursday(ctx).await?,
        "ping" => ping(ctx).await?,
        "room_id" => room_id(ctx).await?,
        "user_id" => user_id(ctx).await?,
        "name_changes" => name_changes(ctx).await?,
        "avatar_changes" => avatar_changes(ctx).await?,
        "divergence" => divergence(ctx).await?,
        "ignore" => ignore(ctx).await?,
        "hitokoto" => hitokoto(bot_ctx, ctx).await?,
        "unignore" => unignore(ctx, args.get(1).copied()).await?,
        _ => _unknown(ctx, command).await?,
    }) else {
        return Ok(());
    };

    let content = content.make_reply_to(&ctx.ev, ForwardThread::Yes, AddMentions::Yes);
    ctx.room.send(content).await?;

    Ok(())
}

#[tracing::instrument(skip(_ctx), err)]
async fn _unknown(
    _ctx: &HandlerContext,
    command: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    Ok(Some(RoomMessageEventContent::text_plain(format!(
        "Unknown command {command}."
    ))))
}

#[tracing::instrument(skip(ctx), err)]
async fn help(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let client = ctx.room.client();
    let Some(user_id) = client.user_id() else {
        tracing::error!("INTERNAL ERROR: When sync happens, the client should have known our user ID but it doesn't ?!");
        return Ok(None);
    };

    Ok(Some(RoomMessageEventContent::text_html(
        format!("Fuuka Bot - User ID: {user_id}\nCommand reference: https://github.com/ShadowRZ/fuuka-bot/blob/master/COMMANDS.md"),
        format!("Fuuka Bot - User ID: {user_id}<br/>Command reference: https://github.com/ShadowRZ/fuuka-bot/blob/master/COMMANDS.md"),
    )))
}

#[tracing::instrument(skip(_ctx), err)]
async fn crazy_thursday(_ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
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

#[tracing::instrument(skip(ctx), err)]
async fn ping(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let now = MilliSecondsSinceUnixEpoch::now().0;
    let event_ts = ctx.ev.origin_server_ts.0;
    let delta: i64 = (now - event_ts).into();
    let duration = Duration::milliseconds(delta);
    let body = format!("Pong after {duration:.8}");

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

#[tracing::instrument(skip(ctx), err)]
async fn room_id(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    Ok(Some(RoomMessageEventContent::text_plain(
        ctx.room.room_id(),
    )))
}

#[tracing::instrument(skip(ctx), err)]
async fn user_id(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = get_reply_target_fallback(&ctx.ev, &ctx.room).await?;

    Ok(Some(RoomMessageEventContent::text_plain(user_id.as_str())))
}

#[tracing::instrument(skip(ctx), err)]
async fn name_changes(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let user_id = get_reply_target_fallback(&ctx.ev, &ctx.room).await?;
    let member = ctx
        .room
        .get_member(&user_id)
        .await?
        .ok_or(Error::ShouldAvaliable)?;

    let mut body = String::new();
    let current_name = member.display_name().unwrap_or("(None)");
    let result = format!("Current Name: {current_name}\n");
    body.push_str(&result);
    let mut count: i32 = 0;

    let event: &MemberEvent = member.event();
    match event {
        MemberEvent::Sync(event) => {
            let stream = StreamFactory::member_state_stream(&ctx.room, event.clone()).peekable();
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

#[tracing::instrument(skip(ctx), err)]
async fn avatar_changes(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let homeserver = &ctx.homeserver;
    let user_id = get_reply_target_fallback(&ctx.ev, &ctx.room).await?;
    let member = ctx
        .room
        .get_member(&user_id)
        .await?
        .ok_or(Error::ShouldAvaliable)?;

    let mut body = String::new();
    let current_avatar = member
        .avatar_url()
        .map(|url| url.http_url(homeserver))
        .transpose()?
        .map(|result| result.to_string())
        .unwrap_or("(None)".to_string());
    let result = format!("Current Avatar: {current_avatar}\n");
    body.push_str(&result);
    let mut count: i32 = 0;

    let event: &MemberEvent = member.event();
    match event {
        MemberEvent::Sync(event) => {
            let stream = StreamFactory::member_state_stream(&ctx.room, event.clone()).peekable();
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
                                let avatar_link = avatar_url.http_url(homeserver)?;
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
                        let avatar_link = event
                            .content
                            .avatar_url
                            .map(|uri| uri.http_url(homeserver))
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

    Ok(Some(RoomMessageEventContent::text_plain(body)))
}

#[tracing::instrument(skip(ctx), err)]
async fn send_avatar(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let target = get_reply_target_fallback(&ctx.ev, &ctx.room).await?;
    let member = ctx
        .room
        .get_member(&target)
        .await?
        .ok_or(Error::ShouldAvaliable)?;

    match member.avatar_url() {
        Some(avatar_url) => {
            let name = member.display_name().unwrap_or(target.as_str());
            let info = get_image_info(avatar_url, &ctx.room.client()).await?;
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

#[tracing::instrument(skip(ctx), err)]
async fn divergence(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let room_hash = crc32fast::hash(ctx.room.room_id().as_bytes());
    let event_id_hash = match &ctx.ev.content.relates_to {
        Some(Relation::Reply { in_reply_to }) => {
            let event_id = &in_reply_to.event_id;
            Some(crc32fast::hash(event_id.as_bytes()))
        }
        _ => None,
    };
    let hash = {
        let seed = room_hash + event_id_hash.unwrap_or(0);
        let mut rng = fastrand::Rng::with_seed(seed.into());
        rng.f32() + if rng.bool() { 1.0 } else { 0.0 }
    };
    Ok(Some(RoomMessageEventContent::text_plain(format!(
        "{hash:.6}%"
    ))))
}

#[tracing::instrument(skip(ctx), err)]
async fn ignore(ctx: &HandlerContext) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let sender = &ctx.sender;

    let Some(target) = get_reply_target(&ctx.ev, &ctx.room).await? else {
        return Err(Error::RequiresReply)?;
    };

    if ctx.room.can_user_ban(sender).await? {
        let member = ctx
            .room
            .get_member(&target)
            .await?
            .ok_or(Error::ShouldAvaliable)?;
        member.ignore().await?;
        Ok(Some(RoomMessageEventContent::text_plain(format!(
            "Ignored {} ({})",
            member.display_name().unwrap_or("(No Name)"),
            sender
        ))))
    } else {
        Err(Error::RequiresBannable)?
    }
}

#[tracing::instrument(skip(ctx), err)]
async fn unignore(
    ctx: &HandlerContext,
    user: Option<&str>,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let sender = &ctx.sender;

    let target = OwnedUserId::try_from(user.ok_or(Error::MissingParamter("user"))?)?;

    if ctx.room.can_user_ban(sender).await? {
        let member = ctx
            .room
            .get_member(&target)
            .await?
            .ok_or(Error::UserNotFound)?;
        member.unignore().await?;
        Ok(Some(RoomMessageEventContent::text_plain(format!(
            "Unignored {} ({})",
            member.display_name().unwrap_or("(No Name)"),
            sender
        ))))
    } else {
        Err(Error::RequiresBannable)?
    }
}

#[tracing::instrument(skip(client), err)]
async fn get_image_info(avatar_url: &MxcUri, client: &Client) -> anyhow::Result<ImageInfo> {
    let request = MediaRequest {
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

#[tracing::instrument(skip(bot_ctx, _ctx), err)]
async fn hitokoto(
    bot_ctx: &BotContext,
    _ctx: &HandlerContext,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let raw_resp = bot_ctx
        .http_client
        .get(&bot_ctx.config.services.hitokoto)
        .send()
        .await?;
    let resp: HitokotoResult = raw_resp.json().await?;

    Ok(Some(RoomMessageEventContent::text_html(
        format!(
            "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
            resp.hitokoto, resp.from_who, resp.from, resp.uuid
        ),
        format!(
            "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
            resp.hitokoto, resp.from_who, resp.from, resp.uuid
        ),
    )))
}
