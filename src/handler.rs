//! Generic Matrix event callback handler.
#![warn(missing_docs)]
use std::sync::Arc;

use crate::Config;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::{Room, RoomMember};
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent, Relation,
    RoomMessageEventContent,
};
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use matrix_sdk::ruma::events::{AnyMessageLikeEventContent, AnyTimelineEvent};
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{Client as MatrixClient, RoomState};
use url::Url;

/// An action, either a command or a message
#[derive(Clone, Debug)]
pub enum Action {
    /// A command.
    Command(Command),
    /// An actionable message.
    Message(Message),
}

/// All avaliable commands.
#[derive(Clone, Debug)]
pub enum Command {
    /// `help`
    Help,
    /// `send_avatar`
    SendAvatar(RoomMember),
    /// `crazy_thursday`
    CrazyThursday,
    /// `ping`
    Ping,
    /// `room_id`
    RoomId,
    /// `user_id`
    UserId(OwnedUserId),
    /// `name_changes`
    NameChanges(RoomMember),
    /// `avatar_chanegs`
    AvatarChanges(RoomMember),
    /// `divergence`
    Divergence,
    /// `hitokoto`
    Hitokoto,
    /// `remind`
    Remind {
        /// Remind target.
        target: OwnedUserId,
        /// Who will be reminded.
        sender: RoomMember,
        /// Remind text.
        content: Option<String>,
    },
    /// `quote`
    Quote {
        /// Event to be quoted.
        ev: AnyTimelineEvent,
        /// Member.
        member: RoomMember,
    },
    /// `upload_sticker`
    UploadSticker {
        /// Event replied.
        ev: AnyTimelineEvent,
        /// Pack name.
        pack_name: String,
        /// Sticker room.
        sticker_room: Room,
    },
}

/// Actionable message.
#[derive(Clone, Debug)]
pub enum Message {
    /// Slash action output.
    Slash {
        /// Action sender.
        from: RoomMember,
        /// Action target.
        to: RoomMember,
        /// Reference text.
        text: String,
    },
    /// Slash action output (formatted).
    SlashFormatted {
        /// Action sender.
        from: RoomMember,
        /// Action target.
        to: RoomMember,
        /// Reference text with slots.
        text: String,
    },
    /// `@Nahida` action.
    Nahida(Url),
    /// `@@` and `@%`.
    Fortune {
        /// Action sender.
        member: RoomMember,
        /// Content.
        text: String,
        /// Show prob or not.
        prob: bool,
    },
}

/// Context for the handler.
#[non_exhaustive]
pub struct Context {
    /// The event that bot was received.
    pub ev: OriginalRoomMessageEvent,
    /// The room where the event was sent from.
    pub room: Room,
    /// The homeserver URL.
    pub homeserver: Url,
    /// HTTP client.
    pub http: Ctx<reqwest::Client>,
    /// The bot config.
    pub config: Arc<Config>,
    /// The action outcome.
    pub action: Action,
}

impl Context {
    /// Dispatch the event content.
    pub async fn dispatch(
        ev: OriginalSyncRoomMessageEvent,
        room: Room,
        homeserver: Url,
        config: Arc<Config>,
        http: Ctx<reqwest::Client>,
    ) {
        let prefix = &config.command_prefix;
        let ev = ev.into_full_event(room.room_id().into());
        let action = Self::match_action(&ev, &room, prefix, &config).await;
        match action {
            Ok(Some(action)) => {
                let ctx = Self {
                    ev,
                    room,
                    homeserver,
                    action,
                    http,
                    config,
                };
                if let Err(e) = ctx.dispatch_inner().await {
                    tracing::error!("Unexpected error happened: {e:#}")
                }
            }
            Err(e) => {
                match room
                    .send(
                        RoomMessageEventContent::text_plain(format!("{e:#}")).make_reply_to(
                            &ev,
                            ForwardThread::No,
                            AddMentions::Yes,
                        ),
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(e) => tracing::error!("Unexpected error happened: {e:#}"),
                }
            }
            Ok(None) => (),
        }
    }

    async fn dispatch_inner(self) -> anyhow::Result<()> {
        let content = {
            if let Err(e) = self.room.typing_notice(true).await {
                tracing::warn!("Error while updating typing notice: {e:#}");
            };
            match self.action {
                Action::Command(ref command) => self.dispatch_command(command.to_owned()).await,
                Action::Message(ref message) => self.dispatch_message(message.to_owned()).await,
            }
        };

        if let Err(e) = self.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:#}");
        };

        let content = match content {
            Ok(Some(content)) => match content {
                AnyMessageLikeEventContent::RoomMessage(msg) => {
                    AnyMessageLikeEventContent::RoomMessage(msg.make_reply_to(
                        &self.ev,
                        ForwardThread::Yes,
                        AddMentions::Yes,
                    ))
                }
                _ => content,
            },
            Err(e) => AnyMessageLikeEventContent::RoomMessage(
                RoomMessageEventContent::text_plain(format!("{e:#}")).make_reply_to(
                    &self.ev,
                    ForwardThread::Yes,
                    AddMentions::Yes,
                ),
            ),
            Ok(None) => return Ok(()),
        };
        if let Err(e) = self.room.typing_notice(false).await {
            tracing::warn!("Error while updating typing notice: {e:#}");
        };
        self.room.send(content).await?;

        Ok(())
    }

    async fn match_action(
        ev: &OriginalRoomMessageEvent,
        room: &Room,
        prefix: &str,
        config: &Config,
    ) -> anyhow::Result<Option<Action>> {
        use anyhow::Context;
        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        let features = &config.features;
        if let Some(body) = body.strip_prefix(prefix) {
            let mut args = shell_words::split(body)
                .context("Parsing command failed")?
                .into_iter();
            let Some(command) = args.next() else {
                return Ok(None);
            };

            match command.as_str() {
                "help" => Ok(Some(Action::Command(Command::Help))),
                "send_avatar" => {
                    let user_id = Self::reply_target_fallback(ev, room).await?;
                    let Some(member) = room.get_member(&user_id).await? else {
                        return Ok(None);
                    };

                    Ok(Some(Action::Command(Command::SendAvatar(member))))
                }
                "crazy_thursday" => Ok(Some(Action::Command(Command::CrazyThursday))),
                "ping" => Ok(Some(Action::Command(Command::Ping))),
                "room_id" => Ok(Some(Action::Command(Command::RoomId))),
                "user_id" => Ok(Some(Action::Command(Command::UserId(ev.sender.clone())))),
                "name_changes" => {
                    let user_id = Self::reply_target_fallback(ev, room).await?;
                    let Some(member) = room.get_member(&user_id).await? else {
                        return Ok(None);
                    };

                    Ok(Some(Action::Command(Command::NameChanges(member))))
                }
                "avatar_changes" => {
                    let user_id = Self::reply_target_fallback(ev, room).await?;
                    let Some(member) = room.get_member(&user_id).await? else {
                        return Ok(None);
                    };

                    Ok(Some(Action::Command(Command::AvatarChanges(member))))
                }
                "divergence" => Ok(Some(Action::Command(Command::Divergence))),
                "hitokoto" => Ok(Some(Action::Command(Command::Hitokoto))),
                "remind" => {
                    let text = args.next();
                    let target = Self::reply_target(ev, room).await?.ok_or(anyhow::anyhow!(
                        "You need to reply to a event for this command to function."
                    ))?;
                    let Some(sender) = room.get_member(&ev.sender).await? else {
                        return Ok(None);
                    };
                    Ok(Some(Action::Command(Command::Remind {
                        target,
                        sender,
                        content: text,
                    })))
                }
                "quote" => {
                    let ev = Self::reply_event(ev, room).await?.ok_or(anyhow::anyhow!(
                        "You need to reply to a event for this command to function."
                    ))?;
                    let Some(member) = room.get_member(ev.sender()).await? else {
                        return Ok(None);
                    };
                    Ok(Some(Action::Command(Command::Quote { ev, member })))
                }
                "upload_sticker" => {
                    // Check if we enable the command.
                    let Some(ref stickers_config) = config.stickers else {
                        return Ok(None);
                    };
                    let Some(sticker_room) = room.client().get_room(&stickers_config.sticker_room)
                    else {
                        return Ok(None);
                    };
                    let power_level = sticker_room.get_user_power_level(&ev.sender).await?;
                    if power_level < 1 {
                        return Ok(None);
                    }
                    let ev = Self::reply_event(ev, room).await?.ok_or(anyhow::anyhow!(
                        "You need to reply to a event for this command to function."
                    ))?;
                    let pack_name = args
                        .next()
                        .ok_or(anyhow::anyhow!("Missing pack name.").context("Invaild argument"))?;
                    Ok(Some(Action::Command(Command::UploadSticker {
                        ev,
                        pack_name,
                        sticker_room,
                    })))
                }
                _ => anyhow::bail!("Unrecognized command {}", command),
            }
        } else if let Some(text) = body.strip_prefix("//") {
            if !features
                .get(room.room_id())
                .map(|f| f.jerryxiao)
                .unwrap_or_default()
            {
                return Ok(None);
            }
            let from_sender = &ev.sender;
            let Some(to_sender) = Self::reply_target(ev, room).await? else {
                return Ok(None);
            };
            let Some(from_member) = room.get_member(from_sender).await? else {
                return Ok(None);
            };
            let Some(to_member) = room.get_member(&to_sender).await? else {
                return Ok(None);
            };
            Ok(Some(Action::Message(Message::SlashFormatted {
                from: from_member,
                to: to_member,
                text: text.to_owned(),
            })))
        } else if ["/", "!!", "\\", "¡¡", "//"]
            .into_iter()
            .any(|p| body.starts_with(p))
        {
            if !features
                .get(room.room_id())
                .map(|f| f.jerryxiao)
                .unwrap_or_default()
            {
                return Ok(None);
            }
            let from_sender = &ev.sender;
            let Some(to_sender) = Self::reply_target(ev, room).await? else {
                return Ok(None);
            };
            let Some(from_member) = room.get_member(from_sender).await? else {
                return Ok(None);
            };
            let Some(to_member) = room.get_member(&to_sender).await? else {
                return Ok(None);
            };
            if let Some(remaining) = body.strip_prefix('/') {
                Ok(Some(Action::Message(Message::Slash {
                    from: from_member,
                    to: to_member,
                    text: remaining.to_string(),
                })))
            } else if let Some(remaining) = body.strip_prefix("!!") {
                Ok(Some(Action::Message(Message::Slash {
                    from: from_member,
                    to: to_member,
                    text: remaining.to_string(),
                })))
            } else if let Some(remaining) = body.strip_prefix('\\') {
                Ok(Some(Action::Message(Message::Slash {
                    from: from_member,
                    to: to_member,
                    text: remaining.to_string(),
                })))
            } else if let Some(remaining) = body.strip_prefix("¡¡") {
                Ok(Some(Action::Message(Message::Slash {
                    from: from_member,
                    to: to_member,
                    text: remaining.to_string(),
                })))
            } else {
                Ok(None)
            }
        } else if ["@@", "@%"].into_iter().any(|p| body.starts_with(p)) {
            if !features
                .get(room.room_id())
                .map(|f| f.jerryxiao)
                .unwrap_or_default()
            {
                return Ok(None);
            }
            let Some(member) = room.get_member(&ev.sender).await? else {
                return Ok(None);
            };
            if let Some(remaining) = body.strip_prefix("@@") {
                Ok(Some(Action::Message(Message::Fortune {
                    member,
                    text: remaining.to_string(),
                    prob: false,
                })))
            } else if let Some(remaining) = body.strip_prefix("@%") {
                Ok(Some(Action::Message(Message::Fortune {
                    member,
                    text: remaining.to_string(),
                    prob: true,
                })))
            } else {
                Ok(None)
            }
        } else if let Some(url) = body.strip_prefix("@Nahida") {
            let url = Url::parse(url)?;
            Ok(Some(Action::Message(Message::Nahida(url))))
        } else {
            Ok(None)
        }
    }

    /// Given a [OriginalRoomMessageEvent], returns the event being replied to.
    pub(crate) async fn reply_event(
        ev: &OriginalRoomMessageEvent,
        room: &Room,
    ) -> anyhow::Result<Option<AnyTimelineEvent>> {
        match &ev.content.relates_to {
            Some(Relation::Reply { in_reply_to }) => {
                let event_id = &in_reply_to.event_id;
                let event = room.event(event_id).await?.event.deserialize()?;
                Ok(Some(event))
            }
            _ => Ok(None),
        }
    }

    /// Given a [OriginalRoomMessageEvent], returns the user ID of the reply target.
    async fn reply_target(
        ev: &OriginalRoomMessageEvent,
        room: &Room,
    ) -> anyhow::Result<Option<OwnedUserId>> {
        Self::reply_event(ev, room)
            .await
            .map(|ev| ev.map(|ev| ev.sender().to_owned()))
    }

    /// Given a [OriginalRoomMessageEvent], returns the user ID of the reply target,
    /// it that doesn't exist, returns the user ID of the sender.
    async fn reply_target_fallback(
        ev: &OriginalRoomMessageEvent,
        room: &Room,
    ) -> anyhow::Result<OwnedUserId> {
        Self::reply_target(ev, room)
            .await
            .map(|r| r.unwrap_or(ev.sender.clone()))
    }
}

/// Called when a message is sent.
#[tracing::instrument(skip_all)]
pub async fn on_sync_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    client: MatrixClient,
    config: Ctx<Arc<Config>>,
    http: Ctx<reqwest::Client>,
) {
    // It should be a joined room.
    if room.state() != RoomState::Joined {
        return;
    }

    // Ignore messages from ourselves.
    if ev.sender == client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        let Ctx(config) = config;
        Context::dispatch(ev, room, client.homeserver(), config, http).await;
    });
}

/// Called when a member event is from an invited room.
#[tracing::instrument(skip_all)]
pub async fn on_stripped_member(ev: StrippedRoomMemberEvent, room: Room, client: MatrixClient) {
    // Ignore state events not for ourselves.
    if ev.state_key != client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        let room_id = room.room_id();
        tracing::info!("Autojoining room {}", room_id);
        let mut delay = 2;
        while let Err(e) = room.join().await {
            use tokio::time::{sleep, Duration};
            tracing::warn!("Failed to join room {room_id} ({e:#}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:#})");
                break;
            }
        }
    });
}

/// Called when we have a tombstone event.
#[tracing::instrument(skip_all)]
pub async fn on_room_replace(ev: OriginalSyncRoomTombstoneEvent, room: Room, client: MatrixClient) {
    tokio::spawn(async move {
        let room_id = ev.content.replacement_room;
        tracing::info!("Room replaced, Autojoining new room {}", room_id);
        let mut delay = 2;
        while let Err(e) = client.join_room_by_id(&room_id).await {
            use tokio::time::{sleep, Duration};
            tracing::warn!("Failed to join room {room_id} ({e:#}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:#})");
                break;
            }
        }
        if let Some(room) = client.get_room(room.room_id()) {
            tokio::spawn(async move {
                if let Err(e) = room.leave().await {
                    tracing::error!("Can't leave the original room {} ({e:#})", room.room_id());
                }
            });
        }
    });
}
