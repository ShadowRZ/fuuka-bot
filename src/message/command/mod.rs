pub(super) mod about;
pub(super) mod bilibili;
pub(super) mod bot;
pub(super) mod delete;
pub(super) mod hitokoto;
pub(super) mod ignore;
pub(super) mod nixpkgs;
pub(super) mod ping;
pub(super) mod pixiv;
pub(super) mod profile;
pub(super) mod room_id;
pub(super) mod rooms;
pub(super) mod unignore;
pub(super) mod user_id;

use matrix_sdk::{Room, event_handler::Ctx, ruma::events::room::message::OriginalRoomMessageEvent};

use crate::message::{Args, Injected, nixpkgs::NixpkgsCommand};

#[tracing::instrument(name = "command", skip_all)]
pub(super) async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    args: Args,
) -> anyhow::Result<()> {
    match args {
        Args::About => self::about::process(ev, room, injected).await,
        Args::Profile {
            category,
            response_type,
        } => self::profile::process(ev, room, injected, category, response_type).await,
        Args::Ping => self::ping::process(ev, room, injected).await,
        Args::Hitokoto => self::hitokoto::process(ev, room, injected).await,
        Args::Ignore => self::ignore::process(ev, room, injected).await,
        Args::Unignore { user_id } => self::unignore::process(ev, room, injected, user_id).await,
        Args::Pixiv { command } => self::pixiv::process(ev, room, injected, command).await,
        Args::Nixpkgs { pr_number, what } => {
            let track = what == Some(NixpkgsCommand::Track);
            self::nixpkgs::process(ev, room, injected, pr_number, track).await
        }
        Args::RoomId => self::room_id::process(ev, room, injected).await,
        Args::UserId => self::user_id::process(ev, room, injected).await,
        Args::Rooms => self::rooms::process(ev, room, injected).await,
        Args::BiliBili { id } => self::bilibili::process(ev, room, injected, &id).await,
        Args::Delete => self::delete::process(ev, room, injected).await,
        Args::Bot(command) => self::bot::process(ev, room, injected, command).await,
    }
}
