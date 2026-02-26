use anyhow::Context as _;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::{OwnedServerName, events::room::message::OriginalRoomMessageEvent};
use matrix_sdk::{Room, event_handler::Ctx};

use crate::Context;

#[tracing::instrument(name = "server", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    server_name: Option<OwnedServerName>,
) -> anyhow::Result<()> {
    let Ctx(Context { http, .. }) = context;
    let server_name = server_name.unwrap_or_else(|| room.own_user_id().server_name().to_owned());

    let federation_server =
        crate::matrix::federation::discover_federation_endpoint(http, &server_name)
            .await
            .context(format!(
                "Failed to query federation endpoint for {server_name}"
            ))?;

    let server_version = crate::matrix::federation::server_version(http, federation_server.server)
        .await
        .context(format!("Failed to query server version for {server_name}"))?;

    let (name, version) = server_version
        .server
        .map(|server| (server.name, server.version))
        .unwrap_or_default();

    room.send(
        RoomMessageEventContent::text_plain(format!(
            "{server_name}: {name} {version}",
            name = name.unwrap_or("(Unknown)".to_string()),
            version = version.unwrap_or("(Unknown)".to_string())
        ))
        .make_reply_to(ev, ForwardThread::No, AddMentions::Yes),
    )
    .await?;

    Ok(())
}
