use bytes::BytesMut;
use matrix_sdk::ruma::ServerName;
use matrix_sdk::ruma::api::IncomingResponse;
use matrix_sdk::ruma::api::OutgoingRequest;
use matrix_sdk::ruma::api::auth_scheme::SendAccessToken;
use matrix_sdk::ruma::api::federation::discovery::discover_homeserver::Request as DiscoverRequest;
use matrix_sdk::ruma::api::federation::discovery::discover_homeserver::Response as DiscoverRespose;
use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Request as ServerVersionRequest;
use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Response as ServerVersionRespose;

pub async fn discover_federation_endpoint<S: AsRef<ServerName>>(
    client: &reqwest::Client,
    server_name: &S,
) -> anyhow::Result<DiscoverRespose> {
    use http::{Uri, uri::Scheme};
    let request = DiscoverRequest::new();
    let base_url = Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(server_name.as_ref().as_str())
        .path_and_query("/")
        .build()?;
    let request = request
        .try_into_http_request::<BytesMut>(&base_url.to_string(), SendAccessToken::None, ())?
        .map(|body| body.freeze());
    let request = reqwest::Request::try_from(request)?;
    let response = client.execute(request).await?;
    Ok(DiscoverRespose::try_from_http_response(
        crate::utils::response_to_http_response(response).await?,
    )?)
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
    let request = request
        .try_into_http_request::<BytesMut>(&base_url.to_string(), SendAccessToken::None, ())?
        .map(|body| body.freeze());
    let request = reqwest::Request::try_from(request)?;
    let response = client.execute(request).await?;
    Ok(ServerVersionRespose::try_from_http_response(
        crate::utils::response_to_http_response(response).await?,
    )?)
}
