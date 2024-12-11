use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};

use super::RequestType;

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
    dptree::case![RequestType::Help].endpoint(|request: super::IncomingRequest| async move {
        let super::IncomingRequest { ev, room } = request;

        room.send(
            RoomMessageEventContent::text_html(HELP_TEXT, HELP_HTML).make_reply_to(
                &ev,
                ForwardThread::No,
                AddMentions::Yes,
            ),
        )
        .await?;

        Ok(())
    })
}
