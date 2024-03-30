//! Helper traits.
#![warn(missing_docs)]
use anyhow::Result;
use matrix_sdk::reqwest::Url;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::MxcUri;

use crate::Error;

/// Extensions to [RoomMember].
pub trait RoomMemberExt {
    /// Returns the display name or the user ID of the specified [RoomMember].
    fn name_or_id(&self) -> &str;
    /// Constructs a HTML link of the specified [RoomMember], known as the mention "pill".
    fn make_pill(&self) -> String;
}

impl RoomMemberExt for RoomMember {
    fn name_or_id(&self) -> &str {
        self.display_name().unwrap_or(self.user_id().as_str())
    }

    fn make_pill(&self) -> String {
        format!(
            "<a href=\"{}\">@{}</a>",
            self.user_id().matrix_to_uri(),
            self.name()
        )
    }
}

/// Extensions to [MxcUri].
pub trait MxcUriExt {
    /// Returns the HTTP URL of the given [MxcUri], with the specified homeserver
    /// using the [Client-Server API](https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3downloadservernamemediaid).
    fn http_url(&self, homeserver: &Url) -> Result<Url>;
}

impl MxcUriExt for MxcUri {
    #[tracing::instrument(err)]
    fn http_url(&self, homeserver: &Url) -> Result<Url> {
        let (server_name, media_id) = self.parts()?;
        Ok(homeserver
            .join("/_matrix/media/r0/download/")?
            .join(format!("{}/{}", server_name, media_id).as_str())?)
    }
}

/// Helper trait to convert types to event content.
pub trait IntoEventContent {
    /// The output of converting.
    type Output;

    /// Given a [Self] and the input, returns the [Self::Output] to send to the room.
    fn event_content(self) -> Self::Output;
}

impl IntoEventContent for anyhow::Error {
    type Output = RoomMessageEventContent;

    fn event_content(self) -> Self::Output {
        match self.downcast_ref::<Error>() {
            Some(Error::MissingParamter(_) | Error::InvaildUrl(_)) => {
                RoomMessageEventContent::text_plain(format!("Invaild input: {self:#}"))
            }
            Some(Error::RequiresBannable | Error::RequiresReply) => {
                RoomMessageEventContent::text_plain(format!(
                    "Command requirement is unsatisfied: {self:#}"
                ))
            }
            Some(Error::UserNotFound) => {
                RoomMessageEventContent::text_plain(format!("Runtime error: {self:#}"))
            }
            Some(Error::NoInfomation) => RoomMessageEventContent::text_plain(format!("{self:#}")),
            Some(Error::ShouldAvaliable) => RoomMessageEventContent::text_plain(format!(
                "⁉️ The bot fired an internal error: {self:#}"
            )),
            Some(Error::MathOverflow | Error::DivByZero) => {
                RoomMessageEventContent::text_plain(format!("Math error happened: {self:#}"))
            }
            None => RoomMessageEventContent::text_plain(format!(
                "⁉️ An unexpected error occoured: {self:#}"
            )),
        }
    }
}
