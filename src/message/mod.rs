use std::sync::Arc;

use matrix_sdk::{
    Room, RoomState,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent,
        RoomMessageEventContent,
    },
};

pub mod command;
pub mod jerryxiao;
pub mod nahida;

/// Injected dependencies.
#[derive(Clone)]
pub struct Injected {
    pub config: tokio::sync::watch::Receiver<crate::Config>,
    pub prefix: String,
    pub http: reqwest::Client,
    pub pixiv: Option<Arc<pixrs::PixivClient>>,
    pub media_proxy: Option<Arc<MediaProxy>>,
}

/// Called when a message is sent.
pub async fn on_sync_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    client: matrix_sdk::Client,
    injected: Ctx<Injected>,
) {
    // It should be a joined room.
    if room.state() != RoomState::Joined {
        return;
    }

    // Ignore messages from ourselves.
    if ev.sender == client.user_id().unwrap() {
        return;
    }

    let room_id = room.room_id().to_owned();
    let ev = ev.into_full_event(room_id);

    let result = process(&ev, &room, &injected).await;

    tokio::spawn(async move {
        if let Err(e) = result {
            send_error_content(&room, e, &ev).await;
        }
    });
}

async fn send_error_content(room: &Room, e: anyhow::Error, ev: &OriginalRoomMessageEvent) {
    use crate::Error;

    let body = RoomMessageEventContent::text_plain(match e.downcast::<crate::Error>() {
        Ok(Error::RequiresReply) => "Replying to a event is required for this command.".to_string(),
        Ok(Error::InvaildArgument { arg, source }) => {
            format!("Invaild argument for {arg}: {source}")
        }
        Ok(Error::MissingArgument(arg)) => format!("Missing argument: {arg}"),
        Ok(Error::UnknownCommand(command)) => format!("Unknown command {command}"),
        Ok(Error::UnexpectedError(e)) => e.to_string(),
        Ok(Error::GraphQLError { service, error }) => {
            format!("GraphQL Error response from {service}: {error:?}")
        }
        Err(e) => {
            tracing::error!("Unexpected error happened: {e:#}");
            format!("Unexpected error happened: {e:#}")
        }
    })
    .make_reply_to(ev, ForwardThread::No, AddMentions::Yes);

    if let Err(e) = room.send(body).await {
        tracing::error!("Unexpected error happened while sending error content: {e:#}");
    }
}

#[tracing::instrument(
    name = "message",
    skip_all,
    fields(
        event_id = %ev.event_id,
        room_id = %room.room_id()
    )
    err
)]
async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let prefix = &injected.prefix;
    use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
    let body = remove_plain_reply_fallback(ev.content.body()).trim();

    if let Some(content) = body.strip_prefix(prefix) {
        tracing::debug!(content, "Received a command request");
        let args = shell_words::split(content)?;
        let ty = self::from_args(args.into_iter())?;
        use self::CommandType;
        match ty {
            Some(ty) => match ty {
                CommandType::Profile {
                    category,
                    response_type,
                } => {
                    self::command::profile::process(ev, room, injected, category, response_type)
                        .await?
                }
                CommandType::Ping => self::command::ping::process(ev, room, injected).await?,
                //CommandType::Divergence => todo!(),
                CommandType::Hitokoto => {
                    self::command::hitokoto::process(ev, room, injected).await?
                }
                //CommandType::Sticker => todo!(),
                CommandType::Ignore => self::command::ignore::process(ev, room, injected).await?,
                CommandType::Unignore(user_id) => {
                    self::command::unignore::process(ev, room, injected, user_id).await?
                }
                CommandType::Pixiv(command) => {
                    self::command::pixiv::process(ev, room, injected, command).await?
                }
                CommandType::Nixpkgs { pr_number, track } => {
                    self::command::nixpkgs::process(ev, room, injected, pr_number, track).await?
                }
                //CommandType::Info => todo!(),
                CommandType::Help => self::command::help::process(ev, room, injected).await?,
                CommandType::RoomId => self::command::room_id::process(ev, room, injected).await?,
                CommandType::UserId => self::command::user_id::process(ev, room, injected).await?,
                CommandType::Rooms => self::command::rooms::process(ev, room, injected).await?,
                CommandType::Quote => self::command::quote::process(ev, room, injected).await?,
            },
            None => return Ok(()),
        }
    } else if let Some(content) = body.strip_prefix("@Nahida ") {
        tracing::debug!(content, "Received a @Nahida request");
        let url = Url::parse(content)?;
        self::nahida::process(ev, room, injected, url).await?;
    } else {
        self::jerryxiao::process(ev, room, injected, body).await?;
    }

    Ok(())
}

use matrix_sdk::ruma::{OwnedUserId, UserId};
use url::Url;

use crate::MediaProxy;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub(super) enum CommandType {
    Profile {
        category: self::profile::Category,
        response_type: self::profile::ResponseType,
    },
    Ping,
    //Divergence,
    Hitokoto,
    //Sticker,
    Ignore,
    Unignore(OwnedUserId),
    Pixiv(self::pixiv::PixivCommand),
    Nixpkgs {
        pr_number: i32,
        track: bool,
    },
    //Info,
    Help,
    RoomId,
    UserId,
    Rooms,
    Quote,
}

pub(super) fn from_args(
    mut args: impl Iterator<Item = String>,
) -> anyhow::Result<Option<CommandType>> {
    let Some(command) = args.next() else {
        return Ok(None);
    };

    match command.as_str() {
        "profile" => Ok(
            self::profile::from_args(args)?.map(|(category, response_type)| CommandType::Profile {
                category,
                response_type,
            }),
        ),
        "ping" => Ok(Some(CommandType::Ping)),
        //"divergence" => Ok(Some(CommandType::Divergence)),
        "hitokoto" => Ok(Some(CommandType::Hitokoto)),
        //"sticker" => Ok(Some(CommandType::Sticker)),
        "ignore" => Ok(Some(CommandType::Ignore)),
        "unignore" => Ok(Some(CommandType::Unignore(UserId::parse(
            args.next()
                .ok_or(crate::Error::MissingArgument("user_id"))?,
        )?))),
        "pixiv" => Ok(Some(CommandType::Pixiv(self::pixiv::from_args(args)?))),
        "nixpkgs" => self::nixpkgs::from_args(args)
            .map(|(pr_number, track)| Some(CommandType::Nixpkgs { pr_number, track })),
        //"info" => Ok(Some(CommandType::Info)),
        "help" => Ok(Some(CommandType::Help)),
        "room_id" => Ok(Some(CommandType::RoomId)),
        "user_id" => Ok(Some(CommandType::UserId)),
        "rooms" => Ok(Some(CommandType::Rooms)),
        "quote" => Ok(Some(CommandType::Quote)),
        _ => Result::Err(crate::Error::UnknownCommand(command).into()),
    }
}

pub mod profile {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum Category {
        Name,
        Avatar,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum ResponseType {
        Current,
        History,
    }

    impl ResponseType {
        fn from_args(mut args: impl Iterator<Item = String>) -> anyhow::Result<Self> {
            let Some(command) = args.next() else {
                return Ok(Self::Current);
            };

            match command.as_str() {
                "history" => Ok(Self::History),
                _ => Result::Err(crate::Error::UnknownCommand(command).into()),
            }
        }
    }

    pub(super) fn from_args(
        mut args: impl Iterator<Item = String>,
    ) -> anyhow::Result<Option<(Category, ResponseType)>> {
        let Some(command) = args.next() else {
            return Ok(None);
        };

        match command.as_str() {
            "name" => Ok(Some((Category::Name, ResponseType::from_args(args)?))),
            "avatar" => Ok(Some((Category::Avatar, ResponseType::from_args(args)?))),
            _ => Result::Err(crate::Error::UnknownCommand(command).into()),
        }
    }
}

pub mod pixiv {
    use std::str::FromStr;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum PixivCommand {
        Ranking,
        IllustInfo(i32),
    }

    pub(super) fn from_args(
        mut args: impl Iterator<Item = String>,
    ) -> anyhow::Result<PixivCommand> {
        Ok(match args.next() {
            Some(id) => PixivCommand::IllustInfo(<i32 as FromStr>::from_str(&id)?),
            None => PixivCommand::Ranking,
        })
    }
}

pub mod nixpkgs {
    pub(super) fn from_args(mut args: impl Iterator<Item = String>) -> anyhow::Result<(i32, bool)> {
        let pr_number: i32 = args
            .next()
            .ok_or(crate::Error::MissingArgument("pr_number"))?
            .parse()?;
        let track = args.next().map(|arg| &arg == "track").unwrap_or_default();

        Ok((pr_number, track))
    }
}
