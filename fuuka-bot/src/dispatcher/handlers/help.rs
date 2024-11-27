use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;

use super::{Event, OutgoingContent, OutgoingResponse};

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

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Help].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>, room: Arc<Room>| async move {
            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_html(HELP_TEXT, HELP_HTML).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
