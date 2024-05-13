//! Bot commands handler.

mod crazy_thursday;
mod functions;
mod info;
mod member_changes;
mod pixiv;
mod quote;
mod remind;
mod send_avatar;
mod upload_sticker;

use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::ruma::OwnedUserId;
use time::Duration;

use crate::handler::Command;
use crate::types::HitokotoResult;
use crate::Context;

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

impl Context {
    /// Dispatchs a command.
    pub async fn dispatch_command(
        &self,
        command: Command,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        match command {
            Command::Help => self._help().await,
            Command::Info => self._info().await,
            Command::SendAvatar(member) => self._send_avatar(member).await,
            Command::CrazyThursday => self._crazy_thursday().await,
            Command::Ping => self._ping().await,
            Command::RoomId => self._room_id().await,
            Command::UserId(user_id) => self._user_id(user_id).await,
            Command::NameChanges(member) => self._name_changes(member).await,
            Command::AvatarChanges(member) => self._avatar_changes(member).await,
            Command::Divergence => self._divergence().await,
            Command::Hitokoto => self._hitokoto().await,
            Command::Remind {
                target,
                sender,
                content,
            } => self._remind(target, sender, content).await,
            Command::Quote { ev, member } => self._quote(ev, member).await,
            Command::UploadSticker {
                ev,
                pack_name,
                sticker_room,
            } => self._upload_sticker(ev, pack_name, sticker_room).await,
            Command::Ignore(user_id) => self._ignore(user_id).await,
            Command::Unignore(user_id) => self._unignore(user_id).await,
            Command::Pixiv(command) => self._pixiv(command).await,
        }
    }

    async fn _help(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_html(HELP_TEXT, HELP_HTML),
        )))
    }

    async fn _ping(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
        let MilliSecondsSinceUnixEpoch(event_ts) = self.ev.origin_server_ts;
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

    async fn _room_id(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(self.room.room_id()),
        )))
    }

    async fn _user_id(
        &self,
        user_id: OwnedUserId,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(user_id.as_str()),
        )))
    }

    async fn _divergence(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let room_hash = crc32fast::hash(self.room.room_id().as_bytes());
        let event_id_hash = match &self.ev.content.relates_to {
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

    #[tracing::instrument(
        skip(self),
        fields(
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    async fn _hitokoto(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let Some(ref services) = self.config.services else {
            return Ok(None);
        };
        let Some(ref hitokoto) = services.hitokoto else {
            return Ok(None);
        };
        let raw_resp = self
            .http
            .get(hitokoto.to_owned())
            .send()
            .await?
            .error_for_status()?;
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

    #[tracing::instrument(skip(self), err)]
    async fn _ignore(
        &self,
        user_id: OwnedUserId,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let client = self.room.client();
        let account = client.account();
        account.ignore_user(&user_id).await?;
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain("Done."),
        )))
    }

    #[tracing::instrument(skip(self), err)]
    async fn _unignore(
        &self,
        user_id: OwnedUserId,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let client = self.room.client();
        let account = client.account();
        account.unignore_user(&user_id).await?;
        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain("Done."),
        )))
    }
}
