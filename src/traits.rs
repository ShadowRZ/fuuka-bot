use std::future::Future;

use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::events::AnyTimelineEvent;
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::{room::RoomMember, ruma::MxcUri};
use url::Url;

use crate::MembershipHistory;

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
    /// Returns the HTTP URL of the given [MxcUri], with the specified homeserver.
    fn authed_http_url(&self, homeserver: &Url) -> anyhow::Result<Url>;
}

impl MxcUriExt for MxcUri {
    fn http_url(&self, homeserver: &Url) -> anyhow::Result<Url> {
        let (server_name, media_id) = self.parts()?;
        Ok(homeserver
            .join("/_matrix/media/r0/download/")?
            .join(format!("{server_name}/{media_id}").as_str())?)
    }

    fn authed_http_url(&self, homeserver: &Url) -> anyhow::Result<Url> {
        let (server_name, media_id) = self.parts()?;
        Ok(homeserver
            .join("/_matrix/client/v1/media/download/")?
            .join(format!("{server_name}/{media_id}").as_str())?)
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

pub trait RoomExt {
    fn in_reply_to_event(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> impl Future<Output = anyhow::Result<Option<AnyTimelineEvent>>> + Send;
    fn in_reply_to_target(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> impl Future<Output = anyhow::Result<Option<OwnedUserId>>> + Send;
    fn in_reply_to_target_fallback(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> impl Future<Output = anyhow::Result<OwnedUserId>> + Send;
    fn get_member_membership_changes<'a>(&'a self, member: &'a RoomMember)
    -> MembershipHistory<'a>;
    fn run_with_typing<F>(&self, fut: F) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        F: IntoFuture<Output = anyhow::Result<()>> + Send,
        <F as IntoFuture>::IntoFuture: Send;
}

impl RoomExt for matrix_sdk::Room {
    async fn in_reply_to_event(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> anyhow::Result<Option<AnyTimelineEvent>> {
        match &ev.content.relates_to {
            Some(Relation::Reply { in_reply_to }) => {
                use matrix_sdk::deserialized_responses::TimelineEventKind;
                let event_id = &in_reply_to.event_id;
                let event = match self.event(event_id, None).await?.kind {
                    TimelineEventKind::PlainText { event } => event
                        .deserialize()?
                        .into_full_event(self.room_id().to_owned()),
                    TimelineEventKind::Decrypted(decrypted) => decrypted.event.deserialize()?,
                    TimelineEventKind::UnableToDecrypt { event, utd_info } => {
                        tracing::warn!(
                            ?utd_info,
                            event_id = %ev.event_id,
                            room_id = %self.room_id(),
                            "Unable to decrypt event {event_id}",
                        );
                        event
                            .deserialize()?
                            .into_full_event(self.room_id().to_owned())
                    }
                };
                Ok(Some(event))
            }
            _ => Ok(None),
        }
    }

    async fn in_reply_to_target(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> anyhow::Result<Option<OwnedUserId>> {
        self.in_reply_to_event(ev)
            .await
            .map(|ev| ev.map(|ev| ev.sender().to_owned()))
    }

    async fn in_reply_to_target_fallback(
        &self,
        ev: &OriginalRoomMessageEvent,
    ) -> anyhow::Result<OwnedUserId> {
        Ok(self
            .in_reply_to_target(ev)
            .await?
            .unwrap_or(ev.sender.clone()))
    }

    fn get_member_membership_changes<'a>(
        &'a self,
        member: &'a RoomMember,
    ) -> MembershipHistory<'a> {
        MembershipHistory::new(self, member)
    }

    async fn run_with_typing<F>(&self, fut: F) -> anyhow::Result<()>
    where
        F: IntoFuture<Output = anyhow::Result<()>> + Send,
        <F as IntoFuture>::IntoFuture: Send,
    {
        self.typing_notice(true).await?;
        fut.await?;
        self.typing_notice(false).await?;
        Ok(())
    }
}
