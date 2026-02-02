pub(super) mod bilibili;
pub(super) mod bot;
pub(super) mod delete;
pub(super) mod help;
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

use crate::message::{CommandType, Injected};

#[tracing::instrument(name = "command", skip_all)]
pub(super) async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    ty: CommandType,
) -> anyhow::Result<()> {
    match ty {
        CommandType::Profile {
            category,
            response_type,
        } => self::profile::process(ev, room, injected, category, response_type).await,
        CommandType::Ping => self::ping::process(ev, room, injected).await,
        CommandType::Hitokoto => self::hitokoto::process(ev, room, injected).await,
        CommandType::Ignore => self::ignore::process(ev, room, injected).await,
        CommandType::Unignore(user_id) => {
            self::unignore::process(ev, room, injected, user_id).await
        }
        CommandType::Pixiv(command) => self::pixiv::process(ev, room, injected, command).await,
        CommandType::Nixpkgs { pr_number, track } => {
            self::nixpkgs::process(ev, room, injected, pr_number, track).await
        }
        CommandType::Help => self::help::process(ev, room, injected).await,
        CommandType::RoomId => self::room_id::process(ev, room, injected).await,
        CommandType::UserId => self::user_id::process(ev, room, injected).await,
        CommandType::Rooms => self::rooms::process(ev, room, injected).await,
        CommandType::BiliBili(id) => self::bilibili::process(ev, room, injected, &id).await,
        CommandType::Delete => self::delete::process(ev, room, injected).await,
        CommandType::Bot(command) => self::bot::process(ev, room, injected, command).await,
    }
}
