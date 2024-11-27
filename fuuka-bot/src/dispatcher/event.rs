use std::str::FromStr;
use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::UserId;
use matrix_sdk::ruma::{events::AnyTimelineEvent, OwnedUserId};
use matrix_sdk::{room::RoomMember, Room};
use url::Url;

/// All avaliable events.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    /// `help`
    Help,
    /// `info`
    Info,
    /// `send_avatar`
    SendAvatar(Arc<RoomMember>),
    /// `crazy_thursday`
    CrazyThursday,
    /// `ping`
    Ping,
    /// `ping-admin`
    PingAdmin,
    /// `room_id`
    RoomId,
    /// `user_id`
    UserId,
    /// `name_changes`
    NameChanges(Arc<RoomMember>),
    /// `avatar_chanegs`
    AvatarChanges(Arc<RoomMember>),
    /// `divergence`
    Divergence,
    /// `hitokoto`
    Hitokoto,
    /// `nixpkgs`
    Nixpkgs { pr_number: i32, track: bool },
    /// `remind`
    Remind {
        /// Remind target.
        target: OwnedUserId,
        /// Who will be reminded.
        sender: Arc<RoomMember>,
        /// Remind text.
        content: Option<String>,
    },
    /// `upload_sticker`
    UploadSticker {
        /// Event replied.
        ev: AnyTimelineEvent,
        /// Pack name.
        pack_name: String,
        /// Sticker room.
        sticker_room: Arc<Room>,
    },
    /// `ignore`
    Ignore(OwnedUserId),
    /// `unignore`
    Unignore(OwnedUserId),
    /// `pixiv`
    Pixiv(PixivCommand),
    // Text triggers.
    /// Slash action output.
    Slash {
        /// Action sender.
        from: Arc<RoomMember>,
        /// Action target.
        to: Arc<RoomMember>,
        /// Reference text.
        text: String,
    },
    /// Slash action output (formatted).
    SlashFormatted {
        /// Action sender.
        from: Arc<RoomMember>,
        /// Action target.
        to: Arc<RoomMember>,
        /// Reference text with slots.
        text: String,
    },
    /// `@Nahida` action.
    Nahida(Url),
    /// `@@` and `@%`.
    Fortune {
        /// Action sender.
        member: Arc<RoomMember>,
        /// Content.
        text: String,
        /// Show prob or not.
        prob: bool,
    },
}

/// Pixiv commands.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PixivCommand {
    /// Ranking.
    Ranking,
    /// Illust info
    IllustInfo(i32),
}

pub(super) async fn event_from_incoming_event(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    config: &crate::Config,
) -> anyhow::Result<Option<Event>> {
    use crate::RoomExt as _;
    use anyhow::Context;
    use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;

    let prefix = &config.command.prefix;

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
            "help" => Ok(Some(Event::Help)),
            "info" => Ok(Some(Event::Info)),
            "send_avatar" => {
                //let user_id = room.in_reply_to_target_fallback(ev).await?;
                let user_id = room.in_reply_to_target_fallback(ev).await?;
                let Some(member) = room.get_member(&user_id).await? else {
                    return Ok(None);
                };

                Ok(Some(Event::SendAvatar(Arc::new(member))))
            }
            "crazy_thursday" => Ok(Some(Event::CrazyThursday)),
            "ping" => Ok(Some(Event::Ping)),
            "ping-admin" => {
                let Some(ref _admin_user) = config.admin_user else {
                    return Ok(None);
                };
                Ok(Some(Event::PingAdmin))
            }
            "room_id" => Ok(Some(Event::RoomId)),
            "user_id" => Ok(Some(Event::UserId)),
            "name_changes" => {
                let user_id = room.in_reply_to_target_fallback(ev).await?;
                let Some(member) = room.get_member(&user_id).await? else {
                    return Ok(None);
                };

                Ok(Some(Event::NameChanges(Arc::new(member))))
            }
            "avatar_changes" => {
                let user_id = room.in_reply_to_target_fallback(ev).await?;
                let Some(member) = room.get_member(&user_id).await? else {
                    return Ok(None);
                };

                Ok(Some(Event::AvatarChanges(Arc::new(member))))
            }
            "divergence" => Ok(Some(Event::Divergence)),
            "hitokoto" => Ok(Some(Event::Hitokoto)),
            "remind" => {
                let text = args.next();
                let target = room
                    .in_reply_to_target(ev)
                    .await?
                    .ok_or(crate::Error::RequiresReply)?;
                let Some(sender) = room.get_member(&ev.sender).await? else {
                    return Ok(None);
                };
                Ok(Some(Event::Remind {
                    target,
                    sender: Arc::new(sender),
                    content: text,
                }))
            }
            "upload_sticker" => {
                // Check if we enable the command.
                let Some(ref stickers_config) = config.stickers else {
                    return Ok(None);
                };
                let Some(sticker_room) = room.client().get_room(&stickers_config.send_to) else {
                    return Ok(None);
                };
                let power_level = sticker_room.get_user_power_level(&ev.sender).await?;
                if power_level < 1 {
                    return Ok(None);
                }
                let ev = room
                    .in_reply_to_event(ev)
                    .await?
                    .ok_or(crate::Error::RequiresReply)?;
                let pack_name = args
                    .next()
                    .ok_or(crate::Error::MissingArgument("pack_name"))?;
                Ok(Some(Event::UploadSticker {
                    ev,
                    pack_name,
                    sticker_room: Arc::new(sticker_room),
                }))
            }
            "ignore" => {
                let Some(ref admin_user) = config.admin_user else {
                    return Ok(None);
                };
                if ev.sender != *admin_user {
                    return Ok(None);
                }
                let ev = room
                    .in_reply_to_event(ev)
                    .await?
                    .ok_or(crate::Error::RequiresReply)?;
                Ok(Some(Event::Ignore(ev.sender().to_owned())))
            }
            "unignore" => {
                let user_id = args
                    .next()
                    .ok_or(crate::Error::MissingArgument("user_id"))?;
                let user_id =
                    UserId::parse(user_id).map_err(|e| crate::Error::InvaildArgument {
                        arg: "User ID",
                        source: e.into(),
                    })?;
                Ok(Some(Event::Unignore(user_id)))
            }
            "pixiv" => {
                if !features.room_pixiv_enabled(room.room_id()) {
                    return Ok(None);
                }
                let illust_id = args.next();
                match illust_id {
                    Some(illust_id) => {
                        let illust_id = <i32 as FromStr>::from_str(&illust_id).map_err(|e| {
                            crate::Error::InvaildArgument {
                                arg: "Illust ID",
                                source: e.into(),
                            }
                        })?;
                        Ok(Some(Event::Pixiv(PixivCommand::IllustInfo(illust_id))))
                    }
                    None => Ok(Some(Event::Pixiv(PixivCommand::Ranking))),
                }
            }
            "nixpkgs" => {
                let pr_number = args
                    .next()
                    .ok_or(crate::Error::MissingArgument("pr_number"))?;
                let track = args.next().map(|s| s == "track").unwrap_or_default();
                let pr_number = i32::from_str(&pr_number)?;
                Ok(Some(Event::Nixpkgs { pr_number, track }))
            }
            _ => Result::Err(crate::Error::UnknownCommand(command).into()),
        }
    } else if let Some(text) = body.strip_prefix("//") {
        if !features.room_jerryxiao_enabled(room.room_id()) {
            return Ok(None);
        }
        let from_sender = &ev.sender;
        let Some(to_sender) = room.in_reply_to_target(ev).await? else {
            return Ok(None);
        };
        let Some(from_member) = room.get_member(from_sender).await? else {
            return Ok(None);
        };
        let Some(to_member) = room.get_member(&to_sender).await? else {
            return Ok(None);
        };
        Ok(Some(Event::SlashFormatted {
            from: Arc::new(from_member),
            to: Arc::new(to_member),
            text: text.to_owned(),
        }))
    } else if ["/", "!!", "\\", "¡¡", "//"]
        .into_iter()
        .any(|p| body.starts_with(p))
    {
        if !features.room_jerryxiao_enabled(room.room_id()) {
            return Ok(None);
        }
        let from_sender = &ev.sender;
        let Some(to_sender) = room.in_reply_to_target(ev).await? else {
            return Ok(None);
        };
        let Some(from_member) = room.get_member(from_sender).await? else {
            return Ok(None);
        };
        let Some(to_member) = room.get_member(&to_sender).await? else {
            return Ok(None);
        };
        if let Some(remaining) = body.strip_prefix('/') {
            Ok(Some(Event::Slash {
                from: Arc::new(from_member),
                to: Arc::new(to_member),
                text: remaining.to_string(),
            }))
        } else if let Some(remaining) = body.strip_prefix("!!") {
            Ok(Some(Event::Slash {
                from: Arc::new(from_member),
                to: Arc::new(to_member),
                text: remaining.to_string(),
            }))
        } else if let Some(remaining) = body.strip_prefix('\\') {
            Ok(Some(Event::Slash {
                from: Arc::new(to_member),
                to: Arc::new(from_member),
                text: remaining.to_string(),
            }))
        } else if let Some(remaining) = body.strip_prefix("¡¡") {
            Ok(Some(Event::Slash {
                from: Arc::new(to_member),
                to: Arc::new(from_member),
                text: remaining.to_string(),
            }))
        } else {
            Ok(None)
        }
    } else if ["@@", "@%"].into_iter().any(|p| body.starts_with(p)) {
        if !features.room_jerryxiao_enabled(room.room_id()) {
            return Ok(None);
        }
        let Some(member) = room.get_member(&ev.sender).await? else {
            return Ok(None);
        };
        if let Some(remaining) = body.strip_prefix("@@") {
            Ok(Some(Event::Fortune {
                member: Arc::new(member),
                text: remaining.to_string(),
                prob: false,
            }))
        } else if let Some(remaining) = body.strip_prefix("@%") {
            Ok(Some(Event::Fortune {
                member: Arc::new(member),
                text: remaining.to_string(),
                prob: true,
            }))
        } else {
            Ok(None)
        }
    } else if let Some(url) = body.strip_prefix("@Nahida") {
        let url = Url::parse(url)?;
        Ok(Some(Event::Nahida(url)))
    } else {
        Ok(None)
    }
}
