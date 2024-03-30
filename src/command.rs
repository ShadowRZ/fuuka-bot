//! Bot commands handler.
#![warn(missing_docs)]
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use file_format::FileFormat;
use file_format::Kind;
use futures_util::pin_mut;
use futures_util::StreamExt;
use matrix_sdk::deserialized_responses::MemberEvent;
use matrix_sdk::event_handler::EventHandlerHandle;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequest;
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
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
use matrix_sdk::ruma::events::sticker::StickerEventContent;
use matrix_sdk::ruma::events::AnyMessageLikeEvent;
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::events::AnyTimelineEvent;
use matrix_sdk::ruma::events::Mentions;
use matrix_sdk::ruma::events::MessageLikeEvent;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::ruma::MxcUri;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::UInt;
use matrix_sdk::Client;
use matrix_sdk::Media;
use matrix_sdk::Room;
use mime::Mime;
use ruma_html::remove_html_reply_fallback;
use time::format_description::well_known::Rfc3339;
use time::macros::offset;
use time::Duration;
use time::OffsetDateTime;
use time::Weekday;
use tokio::task::JoinSet;
use zip::ZipArchive;

use crate::events::sticker::RoomStickerEventContent;
use crate::events::sticker::StickerData;
use crate::events::sticker::StickerPack;
use crate::get_reply_target;
use crate::get_reply_target_fallback;
use crate::stream::StreamFactory;
use crate::traits::MxcUriExt;
use crate::types::HitokotoResult;
use crate::Config;
use crate::Error;
use crate::HandlerContext;
use crate::RoomMemberExt;

/// Dispatches the command and send the command outout.
pub async fn dispatch(
    http: &reqwest::Client,
    config: &Config,
    ctx: &HandlerContext,
    command: &str,
) -> anyhow::Result<()> {
    let mut args = shell_words::split(command)?.into_iter();
    let Some(command) = args.next() else {
        return Ok(());
    };

    let Some(content) = ({
        if let Err(e) = ctx.room.typing_notice(true).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        match command.as_str() {
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
            "hitokoto" => hitokoto(config, http, ctx).await?,
            "unignore" => unignore(ctx, args.next()).await?,
            "remind" => remind(ctx, args.next()).await?,
            "quote" => quote(ctx).await?,
            "upload_sticker" => {
                let pack_name = args.next().ok_or(anyhow::anyhow!("Missing pack name."))?;
                upload_sticker(config, ctx, pack_name).await?
            }
            _ => _unknown(ctx, &command).await?,
        }
    }) else {
        if let Err(e) = ctx.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        return Ok(());
    };

    let content = match content {
        AnyMessageLikeEventContent::RoomMessage(msg) => AnyMessageLikeEventContent::RoomMessage(
            msg.make_reply_to(&ctx.ev, ForwardThread::Yes, AddMentions::Yes),
        ),
        _ => content,
    };
    if let Err(e) = ctx.room.typing_notice(false).await {
        tracing::warn!("Error while updating typing notice: {e:?}");
    };
    ctx.room.send(content).await?;

    Ok(())
}

#[tracing::instrument(skip(_ctx))]
async fn _unknown(
    _ctx: &HandlerContext,
    command: &str,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(format!("Unknown command {command}.")),
    )))
}

static HELP_TEXT: &str = concat!(
    "Fuuka Bot\n\nSource: ",
    env!("CARGO_PKG_REPOSITORY"),
    "\nCommands: https://shadowrz.github.io/fuuka-bot/commands.html",
    "\nSend a feature request: ",
    env!("CARGO_PKG_REPOSITORY"),
    "/issues",
);

static HELP_HTML: &str = concat!(
    "<p>Fuuka Bot</p><p>Source: ",
    env!("CARGO_PKG_REPOSITORY"),
    "<br/>Commands: https://shadowrz.github.io/fuuka-bot/commands.html",
    "<br/>Send a feature request: ",
    env!("CARGO_PKG_REPOSITORY"),
    "/issues</p>",
);

#[tracing::instrument(skip(_ctx))]
async fn help(_ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_html(HELP_TEXT, HELP_HTML),
    )))
}

#[tracing::instrument(skip(_ctx))]
async fn crazy_thursday(
    _ctx: &HandlerContext,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(body),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn ping(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
    let MilliSecondsSinceUnixEpoch(event_ts) = ctx.ev.origin_server_ts;
    let delta: i64 = (now - event_ts).into();
    let body = if delta >= 2000 {
        let duration = Duration::milliseconds(delta);
        format!("Pong after {duration:.3}")
    } else {
        format!("Pong after {}ms", delta)
    };

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(body),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn room_id(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(ctx.room.room_id()),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn user_id(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let user_id = get_reply_target_fallback(&ctx.ev, &ctx.room).await?;

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(user_id.as_str()),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn name_changes(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(body),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn avatar_changes(
    ctx: &HandlerContext,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(body),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn send_avatar(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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
            Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                RoomMessageEventContent::new(MessageType::Image(
                    ImageMessageEventContent::plain(
                        format!("[Avatar of {name}]"),
                        avatar_url.into(),
                    )
                    .info(Some(Box::new(info))),
                )),
            )))
        }
        None => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain("The user has no avatar."),
        ))),
    }
}

#[tracing::instrument(skip(ctx))]
async fn divergence(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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
    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain(format!("{hash:.6}%")),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn ignore(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
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
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(format!(
                "Ignored {} ({})",
                member.display_name().unwrap_or("(No Name)"),
                sender
            )),
        )))
    } else {
        Err(Error::RequiresBannable)?
    }
}

#[tracing::instrument(skip(ctx))]
async fn unignore(
    ctx: &HandlerContext,
    user: Option<String>,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let sender = &ctx.sender;

    let target = OwnedUserId::try_from(user.ok_or(Error::MissingParamter("user"))?)?;

    if ctx.room.can_user_ban(sender).await? {
        let member = ctx
            .room
            .get_member(&target)
            .await?
            .ok_or(Error::UserNotFound)?;
        member.unignore().await?;
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(format!(
                "Unignored {} ({})",
                member.display_name().unwrap_or("(No Name)"),
                sender
            )),
        )))
    } else {
        Err(Error::RequiresBannable)?
    }
}

#[tracing::instrument(skip(client))]
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

#[tracing::instrument(skip(http, _ctx))]
async fn hitokoto(
    config: &Config,
    http: &reqwest::Client,
    _ctx: &HandlerContext,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let Some(ref services) = config.services else {
        return Ok(None);
    };
    let Some(ref hitokoto) = services.hitokoto else {
        return Ok(None);
    };
    let raw_resp = http.get(hitokoto.to_owned()).send().await?.error_for_status()?;
    let resp: HitokotoResult = raw_resp.json().await?;

    let from_who = resp.from_who.unwrap_or_default();

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_html(
            format!(
                "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
                resp.hitokoto, from_who, resp.from, resp.uuid
            ),
            format!(
                "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
                resp.hitokoto, from_who, resp.from, resp.uuid
            ),
        ),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn remind(
    ctx: &HandlerContext,
    content: Option<String>,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let Some(target) = get_reply_target(&ctx.ev, &ctx.room).await? else {
        return Err(Error::RequiresReply)?;
    };

    let sender = ctx.sender.clone();

    let room = &ctx.room;
    let member = room
        .get_member(&sender)
        .await?
        .ok_or(Error::ShouldAvaliable)?;

    room.add_event_handler(
        |ev: OriginalSyncRoomMessageEvent,
         client: Client,
         room: Room,
         handle: EventHandlerHandle| async move {
            let ev = ev.into_full_event(room.room_id().into());
            if ev.sender == target {
                let pill = member.make_pill();
                let reminder = content.unwrap_or("You can ask now.".to_string());
                let content = RoomMessageEventContent::text_html(
                    format!(
                        "Cc {} {}",
                        member.display_name().unwrap_or(sender.as_str()),
                        &reminder
                    ),
                    format!("Cc {} {}", pill, &reminder),
                )
                .make_reply_to(&ev, ForwardThread::No, AddMentions::Yes)
                .add_mentions(Mentions::with_user_ids([target]));
                match room.send(content).await {
                    Ok(_) => (),
                    Err(e) => tracing::error!("Unexpected error happened: {e:?}"),
                }
                client.remove_event_handler(handle);
            };
        },
    );

    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
        RoomMessageEventContent::text_plain("You'll be reminded when the target speaks."),
    )))
}

#[tracing::instrument(skip(ctx))]
async fn quote(ctx: &HandlerContext) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let room_id = &ctx.ev.room_id;
    let ev = crate::get_reply_event(&ctx.ev, &ctx.room)
        .await?
        .ok_or(Error::RequiresReply)?;
    let sender = ev.sender().to_owned();
    let member = ctx
        .room
        .get_member(&sender)
        .await?
        .ok_or(Error::ShouldAvaliable)?;
    match ev {
        AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
            MessageLikeEvent::Original(ev),
        )) => {
            let ev = ev
                .unsigned
                .relations
                .replace
                .clone()
                .map(|ev| ev.into_full_event(room_id.clone()))
                .unwrap_or(ev);
            let content = ev.content;
            let replace_content = content
                .relates_to
                .clone()
                .and_then(|rel| match rel {
                    Relation::Replacement(content) => Some(content),
                    _ => None,
                })
                .map(|replacement| replacement.new_content);
            let content = replace_content.unwrap_or(content.into());
            match content.msgtype {
                MessageType::Text(content) => {
                    let string =
                        format!(
                            "<span size=\"larger\" foreground=\"#1f4788\">{}</span>\n{}",
                            member.name_or_id(),
                            content
                                .formatted
                                .map(|formatted| crate::quote::html2pango(
                                    &remove_html_reply_fallback(&formatted.body)
                                ))
                                .transpose()?
                                .unwrap_or(
                                    html_escape::encode_text(remove_plain_reply_fallback(
                                        &content.body
                                    ))
                                    .to_string()
                                )
                        );
                    let data = crate::quote::quote(
                        member
                            .avatar_url()
                            .map(|url| url.http_url(&ctx.homeserver))
                            .transpose()?
                            .map(|s| s.to_string()),
                        &string,
                    )
                    .await?;
                    let mime: mime::Mime = "image/webp".parse()?;
                    let resp = ctx.room.client().media().upload(&mime, data).await?;
                    let client = &ctx.room.client();
                    let info = get_image_info(&resp.content_uri, client).await?;
                    let send_content =
                        StickerEventContent::new("[Quote]".to_string(), info, resp.content_uri);
                    Ok(Some(AnyMessageLikeEventContent::Sticker(send_content)))
                }
                _ => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(format!(
                        "Unsupported event type, event type in Rust: {}",
                        std::any::type_name_of_val(&content.msgtype)
                    )),
                ))),
            }
        }
        _ => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(format!(
                "Unsupported event type, event type in Rust: {}",
                std::any::type_name_of_val(&ev)
            )),
        ))),
    }
}

#[tracing::instrument(skip(ctx))]
async fn upload_sticker(
    config: &Config,
    ctx: &HandlerContext,
    pack_name: String,
) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
    let ev = crate::get_reply_event(&ctx.ev, &ctx.room)
        .await?
        .ok_or(Error::RequiresReply)?;
    let sender = &ctx.sender;
    let Some(ref stickers_config) = config.stickers else {
        return Ok(None);
    };
    let Some(sticker_room) = ctx.room.client().get_room(&stickers_config.sticker_room) else {
        return Ok(None);
    };
    let power_level = sticker_room.get_user_power_level(sender).await?;
    if power_level < 1 {
        return Ok(None);
    }

    match ev {
        AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
            MessageLikeEvent::Original(ev),
        )) => {
            let content = ev.content;
            match content.msgtype {
                MessageType::File(event_content) => {
                    let name = event_content
                        .filename
                        .clone()
                        .unwrap_or(format!("{}", ev.origin_server_ts.0));
                    let data = ctx
                        .room
                        .client()
                        .media()
                        .get_file(&event_content, false)
                        .await?
                        .ok_or(anyhow::anyhow!("File has no data!"))?;
                    let format = FileFormat::from_bytes(&data);
                    let mimetype = format.media_type();
                    if mimetype != "application/zip" {
                        anyhow::bail!("File is not a ZIP file!");
                    }
                    let content =
                        prepare_sticker_upload_event_content(&ctx.room.client(), data, pack_name)
                            .await?;
                    sticker_room
                        .send_state_event_for_key(&name, content)
                        .await?;
                    Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                        RoomMessageEventContent::text_plain("Done."),
                    )))
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

async fn prepare_sticker_upload_event_content(
    client: &Client,
    data: Vec<u8>,
    display_name: String,
) -> anyhow::Result<RoomStickerEventContent> {
    let media: Arc<Media> = Arc::new(client.media());
    let mut set = JoinSet::new();
    let data = Cursor::new(data);
    let mut archive = ZipArchive::new(data)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if !entry.is_file() {
            continue;
        }
        let path = Path::new(entry.name()).to_owned();
        let Some(name) = path
            .file_name()
            .and_then(|data| data.to_str())
            .map(ToString::to_string)
        else {
            continue;
        };
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let format = FileFormat::from_bytes(&data);
        if format.kind() != Kind::Image {
            continue;
        }
        let mimetype = format.media_type();
        let mime = mimetype.parse::<Mime>()?;
        let dimensions = imagesize::blob_size(&data)?;
        let (width, height) = (dimensions.width, dimensions.height);
        let mut info = ImageInfo::new();
        let width = UInt::try_from(width)?;
        let height = UInt::try_from(height)?;
        let size = data.len();
        let size = UInt::try_from(size)?;
        info.width = Some(width);
        info.height = Some(height);
        info.mimetype = Some(mimetype.to_string());
        info.size = Some(size);

        let media = media.clone();
        set.spawn(async move {
            match media.upload(&mime, data).await {
                Ok(resp) => Some((name, resp.content_uri, info)),
                Err(e) => {
                    tracing::error!("Unexpected error while uploading '{name}': {e:?}");
                    None
                }
            }
        });
    }

    let mut images = HashMap::new();
    while let Some(res) = set.join_next().await {
        if let Some((name, url, info)) = res? {
            images.insert(name, StickerData { url, info });
        }
    }
    let avatar_url = images
        .values()
        .next()
        .map(|data| data.url.clone())
        .ok_or(anyhow::anyhow!("No image was uploaded!"))?;
    Ok(RoomStickerEventContent {
        images,
        pack: StickerPack {
            avatar_url,
            display_name,
            usage: HashSet::from(["sticker".to_string()]),
        },
    })
}
