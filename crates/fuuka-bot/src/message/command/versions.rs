use anyhow::Context as _;
use matrix_sdk::ruma::events::room::message::ReplacementMetadata;
use std::collections::BTreeMap;
use std::str::FromStr as _;
use std::{collections::HashSet, sync::Arc};

use crate::Context;
use hickory_resolver::{ConnectionProvider, Resolver};
use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Response as ServerVersionRespose;
use matrix_sdk::{
    OwnedServerName, Room, RoomMemberships, ServerName,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use tokio::task::JoinSet;

#[tracing::instrument(name = "versions", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
) -> anyhow::Result<()> {
    let Ctx(Context { http, resolver, .. }) = context;

    let msg = room
        .send(
            RoomMessageEventContent::notice_plain("Please wait...").make_reply_to(
                ev,
                ForwardThread::No,
                AddMentions::Yes,
            ),
        )
        .await?;

    let event_id = msg.response.event_id;

    let members = room.members(RoomMemberships::JOIN).await?;

    let servers: HashSet<OwnedServerName> = members
        .into_iter()
        .map(|member| member.user_id().server_name().to_owned())
        .collect();

    room.send(
        RoomMessageEventContent::notice_plain(format!(
            "Found {count} servers, running version check...",
            count = servers.len()
        ))
        .make_replacement(ReplacementMetadata::new(event_id.clone(), None)),
    )
    .await?;

    let mut tasks = JoinSet::new();

    for server in servers {
        let resolver = Arc::clone(resolver);
        let http = http.clone();

        tasks.spawn(async move {
            async fn server_version<P: ConnectionProvider, S: AsRef<ServerName>>(
                resolver: &Resolver<P>,
                client: &reqwest::Client,
                server_name: S,
            ) -> anyhow::Result<ServerVersionRespose> {
                let federation_server =
                    crate::matrix::federation::lookup_server(resolver, client, &server_name)
                        .await?;

                crate::matrix::federation::server_version(client, federation_server)
                    .await
                    .context(format!(
                        "Failed to query server version for {server_name}",
                        server_name = server_name.as_ref()
                    ))
            }

            let result = server_version(&resolver, &http, &server).await;
            (server, result)
        });
    }

    let mut result = BTreeMap::<Option<ServerInfo>, (u32, Vec<OwnedServerName>)>::new();

    let mut errors = String::new();

    for (server_name, resp) in tasks.join_all().await {
        match resp {
            Ok(resp) => {
                let Some(server) = resp.server else {
                    result
                        .entry(None)
                        .and_modify(|(c, _)| *c += 1)
                        .or_insert((1, Vec::new()));
                    continue;
                };

                let software = server.name.map(|name| match name {
                    _ if name.eq_ignore_ascii_case("Synapse") => ServerSoftware::Synapse,
                    _ if name.eq_ignore_ascii_case("Dendrite") => ServerSoftware::Dendrite,
                    _ if name.eq_ignore_ascii_case("Conduit") => ServerSoftware::Conduit,
                    _ if name.eq_ignore_ascii_case("continuwuity") => ServerSoftware::Continuwuity,
                    _ if name.eq_ignore_ascii_case("Catalyst") => ServerSoftware::Catalyst,
                    _ if name.eq_ignore_ascii_case("Tuwunel") => ServerSoftware::Tuwunel,
                    other => ServerSoftware::Other(other),
                });

                let version = server
                    .version
                    .unwrap_or_default()
                    .split(' ')
                    .next()
                    .and_then(|ver| {
                        tracing::debug!(
                            fuuka_bot.versions.checking_server = %server_name,
                            "Parsing version string {ver}"
                        );
                        pep440_rs::Version::from_str(ver)
                            .inspect_err(|e| {
                                tracing::debug!(
                                    fuuka_bot.versions.checking_server = %server_name,
                                    "Error parsing version string: {e}"
                                )
                            })
                            .ok()
                    });

                result
                    .entry(Some(ServerInfo { software, version }))
                    .and_modify(|(c, servers)| {
                        *c += 1;
                        servers.push(server_name.clone());
                    })
                    .or_insert((1, vec![server_name]));
            }
            Err(e) => errors.push_str(&format!("* {server_name}: {e:#}\n")),
        }
    }

    let mut text = "## Versions\n".to_string();

    for (info, (count, servers)) in result.into_iter().rev() {
        let Some(ServerInfo { software, version }) = info else {
            text.push_str(&format!("* {count} servers are running on (Unknown)\n"));
            continue;
        };

        let info_str = format!(
            "{name} {version}",
            name = match software {
                Some(ServerSoftware::Synapse) => "Synapse".to_string(),
                Some(ServerSoftware::Dendrite) => "Dendrite".to_string(),
                Some(ServerSoftware::Continuwuity) => "continuwuity".to_string(),
                Some(ServerSoftware::Conduit) => "Conduit".to_string(),
                Some(ServerSoftware::Catalyst) => "Catalyst".to_string(),
                Some(ServerSoftware::Tuwunel) => "Tuwunel".to_string(),
                Some(ServerSoftware::Other(software)) => software,
                None => "(Unknown)".to_string(),
            },
            version = version
                .map(|v| v.to_string())
                .unwrap_or("(Unknown)".to_string())
        );

        text.push_str(&format!("* {count} servers running on {info_str}"));
        if !servers.is_empty() {
            let servers: Vec<_> = servers.iter().map(ToString::to_string).collect();
            text.push_str(" (");
            text.push_str(&servers.join(", "));
            text.push(')');
        }
        text.push('\n');
    }
    text.push_str("\n## Errors\n");
    text.push_str(&errors);

    room.send(
        RoomMessageEventContent::notice_markdown(text)
            .make_replacement(ReplacementMetadata::new(event_id.clone(), None)),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ServerInfo {
    software: Option<ServerSoftware>,
    version: Option<pep440_rs::Version>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ServerSoftware {
    Other(String),
    Tuwunel,
    Catalyst,
    Conduit,
    Continuwuity,
    Dendrite,
    Synapse,
}
