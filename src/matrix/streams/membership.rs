use std::pin::Pin;

use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt, stream::unfold};
use matrix_sdk::ruma::events::AnySyncStateEvent;
use matrix_sdk::ruma::events::AnySyncTimelineEvent;
use matrix_sdk::ruma::events::SyncStateEvent;
use matrix_sdk::ruma::events::room::member::OriginalSyncRoomMemberEvent;
use pin_project_lite::pin_project;

pin_project! {
    /// A named stream to emit room membership history.
    pub struct MembershipStream<'a> {
        inner: BoxStream<'a, OriginalSyncRoomMemberEvent>,
    }
}

pub(crate) fn history<'a>(
    room: &'a matrix_sdk::Room,
    member: &'a matrix_sdk::room::RoomMember,
) -> MembershipStream<'a> {
    let event = member.event();
    let event_id = event.event_id().map(ToOwned::to_owned);
    let inner: BoxStream<'a, OriginalSyncRoomMemberEvent> = match event_id {
        Some(event_id) => {
            use matrix_sdk::deserialized_responses::TimelineEventKind;
            use matrix_sdk::ruma::EventId;

            unfold(Some(event_id), |event_id| async {
                let event_id = event_id?;

                let event = room
                    .event(&event_id, None)
                    .await
                    .map_err(|e| {
                        tracing::warn!(
                            member_id = %member.user_id(),
                            "Unexpected error happened during iteration of state events: {e:#}"
                        )
                    })
                    .ok()?;

                let TimelineEventKind::PlainText { event } = event.kind else {
                    // Member Event currently unencrypted
                    return None;
                };

                let Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(
                    SyncStateEvent::Original(orig),
                ))) = event.deserialize()
                else {
                    return None;
                };

                let next_event_id = event
                    .get_field::<String>("replaces_state")
                    .ok()
                    .flatten()
                    .and_then(|e| EventId::parse(e).ok());

                Some((orig, next_event_id))
            })
            .boxed()
        }
        None => futures_util::stream::empty().boxed(),
    };

    MembershipStream { inner }
}

impl Stream for MembershipStream<'_> {
    type Item = OriginalSyncRoomMemberEvent;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.as_mut().poll_next(cx)
    }
}
