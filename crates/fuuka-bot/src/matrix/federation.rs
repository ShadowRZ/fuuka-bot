use std::collections::BTreeMap;

use bytes::BytesMut;
use hickory_resolver::ConnectionProvider;
use hickory_resolver::Resolver;
use hickory_resolver::proto::rr::RData;
use matrix_sdk::OwnedServerName;
use matrix_sdk::ServerName;
use matrix_sdk::ruma::api::IncomingResponseExt as _;
use matrix_sdk::ruma::api::OutgoingRequestExt as _;
use matrix_sdk::ruma::api::federation::discovery::discover_homeserver::Request as DiscoverRequest;
use matrix_sdk::ruma::api::federation::discovery::discover_homeserver::Response as DiscoverRespose;
use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Request as ServerVersionRequest;
use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Response as ServerVersionRespose;

pub async fn discover_federation_endpoint<S: AsRef<ServerName>>(
    client: &reqwest::Client,
    server_name: S,
) -> anyhow::Result<DiscoverRespose> {
    use http::{Uri, uri::Scheme};
    let request = DiscoverRequest::new();
    let base_url = Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(server_name.as_ref().as_str())
        .path_and_query("/")
        .build()?;
    let request = request
        .try_into_http_request::<BytesMut>(&base_url.to_string(), (), ())?
        .map(|body| body.freeze());
    let request = reqwest::Request::try_from(request)?;
    let response = client.execute(request).await?;
    let response = crate::utils::response_to_http_response(response).await?;
    let (parts, body) = response.into_parts();
    let response = http::Response::from_parts(parts, body.as_ref());
    Ok(DiscoverRespose::try_from_http_response(response)?)
}

pub async fn lookup_server<P: ConnectionProvider, S: AsRef<ServerName>>(
    resolver: &Resolver<P>,
    client: &reqwest::Client,
    server_name: S,
) -> anyhow::Result<OwnedServerName> {
    // Step 1: Literal IP address
    if server_name.as_ref().is_ip_literal() {
        Ok(ServerName::parse(format!(
            "{host}:{port}",
            host = server_name.as_ref().host(),
            port = server_name.as_ref().port().unwrap_or(8448)
        ))?)
    } else if server_name.as_ref().port().is_some() {
        // Step 2: Explict port
        Ok(server_name.as_ref().to_owned())
    } else {
        // Step 3: Delegation resolve
        match discover_federation_endpoint(client, &server_name).await {
            Ok(resp) => {
                let names = srv_lookup(resolver, &resp.server).await?;
                Ok(match names.into_values().next() {
                    Some(server_name) => server_name,
                    None => resp.server,
                })
            }
            Err(e) => {
                tracing::info!(
                    fuuka_bot.federation.host_for_discovery = server_name.as_ref().host(),
                    "Failed to query for server discovery endpoint: {e}",
                );
                let names = srv_lookup(resolver, &server_name).await?;
                Ok(match names.into_values().next() {
                    Some(server_name) => server_name,
                    None => server_name.as_ref().to_owned(),
                })
            }
        }
    }
}

async fn srv_lookup<P: ConnectionProvider, S: AsRef<ServerName>>(
    resolver: &Resolver<P>,
    server_name: S,
) -> anyhow::Result<BTreeMap<u16, OwnedServerName>> {
    let host = server_name.as_ref().host();
    let fed_name = format!("_matrix-fed._tcp.{host}.");
    let matrix_name = format!("_matrix._tcp.{host}.");

    let (fed_result, matrix_result) = tokio::join!(
        resolver.srv_lookup(&fed_name),
        resolver.srv_lookup(&matrix_name)
    );

    let mut server_names = BTreeMap::new();

    match fed_result {
        Ok(result) => {
            for record in result.answers() {
                let RData::SRV(srv) = &record.data else {
                    continue;
                };
                let host = srv.target.to_ascii();
                let port = srv.port;
                match ServerName::parse(format!("{host}:{port}",)) {
                    Ok(server_name) => server_names.insert(srv.priority, server_name),
                    Err(e) => {
                        tracing::warn!(
                            fuuka_bot.federation.host_for_srv_lookup = host,
                            "Failed to convert the SRV record {host}:{port} to valid Matrix server name: {e}",
                        );
                        continue;
                    }
                };
            }
        }
        Err(e) => tracing::info!(
            fuuka_bot.federation.host_for_srv_lookup = host,
            "Failed to resolve {fed_name} SRV record: {e}",
        ),
    }

    match matrix_result {
        Ok(result) => {
            for record in result.answers() {
                let RData::SRV(srv) = &record.data else {
                    continue;
                };
                let host = srv.target.to_ascii();
                let port = srv.port;
                match ServerName::parse(format!("{host}:{port}",)) {
                    Ok(server_name) => server_names.insert(srv.priority, server_name),
                    Err(e) => {
                        tracing::warn!(
                            fuuka_bot.federation.host_for_srv_lookup = host,
                            "Failed to convert the SRV record {host}:{port} to valid Matrix server name: {e}",
                        );
                        continue;
                    }
                };
            }
        }
        Err(e) => tracing::info!(
            fuuka_bot.federation.host_for_srv_lookup = host,
            "Failed to resolve {matrix_name} SRV record: {e}",
        ),
    }

    Ok(server_names)
}

pub async fn server_version<S: AsRef<ServerName>>(
    client: &reqwest::Client,
    server_name: S,
) -> anyhow::Result<ServerVersionRespose> {
    use http::{Uri, uri::Scheme};
    let request = ServerVersionRequest::new();
    let base_url = Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(server_name.as_ref().as_str())
        .path_and_query("/")
        .build()?;
    tracing::debug!(fuuka_bot.federation.server_name = %server_name.as_ref(), "Querying {base_url} for server versions...");
    let request = request
        .try_into_http_request::<BytesMut>(&base_url.to_string(), (), ())?
        .map(|body| body.freeze());
    let request = reqwest::Request::try_from(request)?;
    let response = client.execute(request).await?;
    let response = crate::utils::response_to_http_response(response).await?;
    let (parts, body) = response.into_parts();
    let response = http::Response::from_parts(parts, body.as_ref());
    Ok(ServerVersionRespose::try_from_http_response(response)?)
}
