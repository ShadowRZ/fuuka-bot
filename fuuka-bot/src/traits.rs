use matrix_sdk::{room::RoomMember, ruma::MxcUri};
use url::Url;

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
    fn http_url(&self, homeserver: &Url) -> anyhow::Result<Url>;
}

impl MxcUriExt for MxcUri {
    #[tracing::instrument(err)]
    fn http_url(&self, homeserver: &Url) -> anyhow::Result<Url> {
        let (server_name, media_id) = self.parts()?;
        Ok(homeserver
            .join("/_matrix/media/r0/download/")?
            .join(format!("{}/{}", server_name, media_id).as_str())?)
    }
}

pub trait IllustTagsInfoExt {
    /// Check if we have the untranslated tag `tag`.
    fn has_tag(&self, tag: &str) -> bool;
    /// Check if we have any of the untranslated tag `tags`.
    fn has_any_tag(&self, tags: &[&str]) -> bool;
}

impl IllustTagsInfoExt for pixrs::IllustTagsInfo {
    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|il| il.tag == tag)
    }

    fn has_any_tag(&self, tags: &[&str]) -> bool {
        self.tags.iter().any(|il| tags.iter().any(|t| il.tag == *t))
    }
}
