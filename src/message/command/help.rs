use crate::message::Injected;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};

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

#[tracing::instrument(name = "help", skip(ev, room, injected), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let _ = injected;

    room.send(
        RoomMessageEventContent::text_html(HELP_TEXT, HELP_HTML).make_reply_to(
            ev,
            ForwardThread::No,
            AddMentions::Yes,
        ),
    )
    .await?;

    Ok(())
}
