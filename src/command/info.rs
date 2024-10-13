use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::Context;

impl Context {
    #[tracing::instrument(skip(self), err)]
    pub(super) async fn _info(&self) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        use matrix_sdk::ruma::api::federation::discovery::get_server_version::v1::Request as ServerVersionRequest;
        let client = self.room.client();
        let Some(user_id) = client.user_id() else {
            // Should never happen
            return Ok(None);
        };
        let unknown = "[Unknown]".to_string();
        let request = ServerVersionRequest::new();
        let server_info = client.send(request, None).await;
        let profile = client.account().fetch_user_profile().await;
        let server_info_str = match server_info {
            Ok(resp) => match resp.server {
                Some(server) => format!(
                    "{name} {version}",
                    name = server.name.as_ref().unwrap_or(&unknown),
                    version = server.version.as_ref().unwrap_or(&unknown)
                ),
                None => unknown,
            },
            Err(e) => format!("[Request Error: {e}]"),
        };
        let profile_str = match profile {
            Ok(profile) => profile
                .displayname
                .unwrap_or(format!("{user_id} (No Display Name)")),
            Err(e) => format!("[Request Error: {e}]"),
        };

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(format!(
                "Profile: {profile_str}\nServer: {server_info_str}"
            )),
        )))
    }
}
