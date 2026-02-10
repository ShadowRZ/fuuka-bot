use std::sync::Arc;

use clap::Parser;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{
    Room, RoomState,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent,
        RoomMessageEventContent,
    },
};
use url::Url;

use crate::MediaProxy;

pub mod command;
pub mod jerryxiao;
pub mod nahida;

static HELP_TEXT: &str = concat!(
    "Fuuka Bot\n\nSource: ",
    env!("CARGO_PKG_REPOSITORY"),
    "\nCommands: https://shadowrz.github.io/fuuka-bot/commands.html",
    "\nSend a feature request: ",
    env!("CARGO_PKG_REPOSITORY"),
    "/issues",
);

/// Injected dependencies.
#[derive(Clone)]
pub struct Injected {
    pub config: tokio::sync::watch::Receiver<crate::Config>,
    pub prefix: String,
    pub http: reqwest::Client,
    pub pixiv: Option<Arc<pixrs::PixivClient>>,
    pub media_proxy: Option<MediaProxy>,
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

    tokio::spawn(async move {
        let result = process(&ev, &room, &injected).await;

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
        Ok(Error::GitHubError(source)) => {
            format!("Error when fetching infomation from GitHub: {source}")
        }
        Err(e) => {
            format!("Unexpected error happened: {e:#}")
        }
    })
    .make_reply_to(ev, ForwardThread::No, AddMentions::Yes);

    if let Err(e) = room.send(body).await {
        tracing::error!(
            room_id = %room.room_id(),
            "Unexpected error happened while sending error content: {e:#}"
        );
    }
}

#[tracing::instrument(
    name = "message",
    skip_all,
    fields(
        event_id = %ev.event_id,
        room_id = %room.room_id()
    )
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
        let content = content.trim();
        tracing::debug!(content, "Received a command request");
        let args = shell_words::split(content)?;
        let args = Args::try_parse_from(args);
        match args {
            Ok(args) => self::command::process(ev, room, injected, args).await?,
            Err(e) => {
                let text = e.render().to_string();
                let body = RoomMessageEventContent::text_plain(text).make_reply_to(
                    ev,
                    ForwardThread::No,
                    AddMentions::Yes,
                );
                room.send(body).await?;
            }
        }
    } else if let Some(content) = body.strip_prefix("@Nahida ") {
        let content = content.trim();
        tracing::debug!(content, "Received a @Nahida request");
        let url = Url::parse(content)?;
        self::nahida::process(ev, room, injected, url).await?;
    } else {
        self::jerryxiao::process(ev, room, injected, body).await?;
    }

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(
    disable_help_flag = true,
    arg_required_else_help = true,
    multicall = true,
    before_help = HELP_TEXT,
)]
pub enum Args {
    /// Print info about the bot.
    About,
    /// Send user profile infomation.
    Profile {
        /// The category to use.
        #[arg(value_enum)]
        category: self::profile::Category,
        /// Response type.
        #[arg(value_enum, default_value_t = self::profile::ResponseType::Current)]
        response_type: self::profile::ResponseType,
    },
    /// Ping the bot.
    Ping,
    /// Print a hitokoto.
    Hitokoto,
    /// Ignore a user.
    Ignore,
    /// Unignore a user.
    Unignore { user_id: OwnedUserId },
    /// Pixiv related commands.
    Pixiv {
        /// Either a numeric illust id, or a ranking category.
        #[arg(name = "MODE_OR_ILLUST_ID", default_value = "daily")]
        command: self::pixiv::PixivCommand,
    },
    /// Bot management commands,
    #[command(subcommand)]
    Bot(self::bot::BotCommand),
    /// Nixpkgs command.
    Nixpkgs {
        pr_number: i32,
        what: Option<self::nixpkgs::NixpkgsCommand>,
    },
    /// Delete a bot message.
    Delete,
    /// Send the room's ID.
    #[command(name = "room_id")]
    RoomId,
    /// Send the user's ID.
    #[command(name = "user_id")]
    UserId,
    /// (Admin only) Print info regarding joined rooms.
    Rooms,
    /// Send infomation of a BiliBili video.
    #[command(name = "bilibili")]
    BiliBili { id: String },
}

pub mod profile {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, clap::ValueEnum)]
    pub enum Category {
        Name,
        Avatar,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, clap::ValueEnum)]
    pub enum ResponseType {
        Current,
        History,
    }
}

pub mod pixiv {
    #[derive(Clone, Copy, Debug)]
    pub enum PixivCommand {
        Ranking(RankingMode),
        Illust(i32),
    }

    impl Default for PixivCommand {
        fn default() -> Self {
            Self::Ranking(RankingMode::Daily)
        }
    }

    impl clap::builder::ValueParserFactory for PixivCommand {
        type Parser = PixivCommandParser;

        fn value_parser() -> Self::Parser {
            PixivCommandParser
        }
    }

    #[doc(hidden)]
    #[derive(Copy, Clone)]
    pub struct PixivCommandParser;

    impl clap::builder::TypedValueParser for PixivCommandParser {
        type Value = PixivCommand;

        fn parse_ref(
            &self,
            cmd: &clap::Command,
            arg: Option<&clap::Arg>,
            value: &std::ffi::OsStr,
        ) -> Result<Self::Value, clap::Error> {
            let ranking_parser = clap::value_parser!(RankingMode);
            if let Ok(ranking) = ranking_parser.parse_ref(cmd, arg, value) {
                return Ok(PixivCommand::Ranking(ranking));
            }

            let illust_id_parser = clap::value_parser!(i32);
            if let Ok(illust_id) = illust_id_parser.parse_ref(cmd, arg, value) {
                return Ok(PixivCommand::Illust(illust_id));
            }

            Err(clap::Error::new(clap::error::ErrorKind::InvalidValue))
        }

        fn possible_values(
            &self,
        ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
            let inner_parser = clap::value_parser!(RankingMode);
            #[allow(clippy::needless_collect)] // Erasing a lifetime
            inner_parser.possible_values().map(|ps| {
                let ps = ps.collect::<Vec<_>>();
                let ps: Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_> =
                    Box::new(ps.into_iter());
                ps
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
    pub enum RankingMode {
        #[default]
        Daily,
        Weekly,
        Monthly,
        Rookie,
        Original,
        Male,
        Female,
        DailyR18,
        WeeklyR18,
        MaleR18,
        FemaleR18,
        R18G,
    }

    impl From<RankingMode> for pixrs::RankingMode {
        fn from(value: RankingMode) -> Self {
            match value {
                RankingMode::Daily => Self::Daily,
                RankingMode::Weekly => Self::Weekly,
                RankingMode::Monthly => Self::Monthly,
                RankingMode::Rookie => Self::Rookie,
                RankingMode::Original => Self::Original,
                RankingMode::Male => Self::Male,
                RankingMode::Female => Self::Female,
                RankingMode::DailyR18 => Self::DailyR18,
                RankingMode::WeeklyR18 => Self::WeeklyR18,
                RankingMode::MaleR18 => Self::MaleR18,
                RankingMode::FemaleR18 => Self::FemaleR18,
                RankingMode::R18G => Self::R18G,
            }
        }
    }
}

pub mod bot {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash, clap::Subcommand)]
    pub enum BotCommand {
        SetAvatar,
        SetDisplayName { display_name: String },
    }
}

pub mod nixpkgs {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash, clap::ValueEnum)]
    pub enum NixpkgsCommand {
        Track,
    }
}
