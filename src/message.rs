//! Responses to messages that are not commands.
#![warn(missing_docs)]
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::Mentions;
use matrix_sdk::ruma::UserId;
use url::Url;

use crate::dicer::DiceCandidate;
use crate::dicer::ParseError;
use crate::jerryxiao::make_randomdraw_event_content;
use crate::traits::IntoEventContent;
use crate::BotContext;
use crate::Error;
use crate::HandlerContext;
use crate::{get_reply_target, jerryxiao::make_jerryxiao_event_content};

/// Dispatch the messages that are not commands but otherwise actionable.
pub async fn dispatch(bot_ctx: &BotContext, ctx: &HandlerContext) -> anyhow::Result<()> {
    let features = &bot_ctx.config.features;
    let content = if ["/", "!!", "\\", "¡¡"]
        .iter()
        .any(|p| ctx.body.starts_with(*p))
    {
        if !features
            .get(ctx.room.room_id())
            .map(|f| f.jerryxiao)
            .unwrap_or_default()
        {
            return Ok(());
        }
        let mut splited = ctx.body.split_whitespace();
        // If the first part of the message is pure ASCII, skip it
        if splited.next().map(str::is_ascii).unwrap_or(true) {
            return Ok(());
        };

        let from_sender = &ctx.sender;
        let Some(to_sender) = get_reply_target(&ctx.ev, &ctx.room).await? else {
            return Ok(());
        };

        if let Err(e) = ctx.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        _dispatch_jerryxiao(&ctx.room, &ctx.body, from_sender, &to_sender)
            .await?
            .map(|c| c.add_mentions(Mentions::with_user_ids([to_sender])))
    } else if ["@@", "@%"].iter().any(|p| ctx.body.starts_with(*p)) {
        if !features
            .get(ctx.room.room_id())
            .map(|f| f.randomdraw)
            .unwrap_or_default()
        {
            return Ok(());
        }
        if let Err(e) = ctx.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        _dispatch_randomdraw(&ctx.ev, &ctx.room, &ctx.body).await?
    } else if ctx.body.starts_with("@=") {
        if let Err(e) = ctx.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        _dispatch_dicer(&ctx.body).await?
    } else if ctx.body.starts_with("@Nahida") {
        if let Err(e) = ctx.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:?}");
        };
        _dispatch_nahida(bot_ctx, &ctx.body).await?
    } else {
        None
    };

    if let Err(e) = ctx.room.typing_notice(false).await {
        tracing::warn!("Error while updating typing notice: {e:?}");
    };
    let Some(content) = content else {
        return Ok(());
    };

    let content = content.make_reply_to(&ctx.ev, ForwardThread::Yes, AddMentions::Yes);
    ctx.room.send(content).await?;

    Ok(())
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
    ev: &OriginalRoomMessageEvent,
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
            Err(err) => {
                let error = ParseError {
                    input: expr.to_string(),
                    err,
                };
                return Ok(Some(error.event_content()));
            }
        };
        let task = tokio::task::spawn_blocking(|| cand.expr.eval());
        let result = match task.await? {
            Ok(result) => result,
            Err(err) => return Ok(Some(err.event_content())),
        };
        let string = match cand.target {
            Some(target) => {
                if result < (target.into()) {
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

async fn _dispatch_nahida(
    bot_ctx: &BotContext,
    body: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    if let Some(url) = body.strip_prefix("@Nahida") {
        let url = Url::parse(url.trim()).map_err(Error::InvaildUrl)?;
        crate::nahida::dispatch(&url, &bot_ctx.http_client).await
    } else {
        Ok(None)
    }
}
