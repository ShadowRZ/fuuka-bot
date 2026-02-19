mod about;
mod bilibili;
mod bot;
mod delete;
mod hitokoto;
mod ignore;
mod nixpkgs;
mod ping;
mod pixiv;
mod profile;
mod room_id;
mod rooms;
mod server;
mod unignore;
mod user_id;

use matrix_sdk::{Room, event_handler::Ctx, ruma::events::room::message::OriginalRoomMessageEvent};

use crate::{
    Context,
    message::{Args, nixpkgs::NixpkgsCommand},
};

#[tracing::instrument(name = "command", skip_all)]
pub(super) async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    args: Args,
) -> anyhow::Result<()> {
    match args {
        Args::About => self::about::process(ev, room, context).await,
        Args::Profile {
            category,
            response_type,
        } => self::profile::process(ev, room, context, category, response_type).await,
        Args::Ping => self::ping::process(ev, room, context).await,
        Args::Hitokoto => self::hitokoto::process(ev, room, context).await,
        Args::Ignore => self::ignore::process(ev, room, context).await,
        Args::Unignore { user_id } => self::unignore::process(ev, room, context, user_id).await,
        Args::Pixiv { command } => self::pixiv::process(ev, room, context, command).await,
        Args::Nixpkgs { pr_number, what } => {
            let track = what == Some(NixpkgsCommand::Track);
            self::nixpkgs::process(ev, room, context, pr_number, track).await
        }
        Args::RoomId => self::room_id::process(ev, room, context).await,
        Args::UserId => self::user_id::process(ev, room, context).await,
        Args::Rooms => self::rooms::process(ev, room, context).await,
        Args::BiliBili { id } => self::bilibili::process(ev, room, context, &id).await,
        Args::Delete => self::delete::process(ev, room, context).await,
        Args::Bot(command) => self::bot::process(ev, room, context, command).await,
        Args::Server { server_name } => self::server::process(ev, room, context, server_name).await,
    }
}
