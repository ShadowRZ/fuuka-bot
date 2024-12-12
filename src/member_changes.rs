use std::pin::Pin;

use futures_util::stream::BoxStream;
use futures_util::{stream::unfold, Stream, StreamExt};
use matrix_sdk::ruma::events::room::member::OriginalSyncRoomMemberEvent;
use matrix_sdk::ruma::events::AnySyncStateEvent;
use matrix_sdk::ruma::events::AnySyncTimelineEvent;
use matrix_sdk::ruma::events::SyncStateEvent;

use pin_project_lite::pin_project;

pin_project! {
    /// A named stream to emit room membership history.
    pub struct MembershipHistory<'a> {
        stream: BoxStream<'a, OriginalSyncRoomMemberEvent>,
    }
}

impl<'a> MembershipHistory<'a> {
    pub(crate) fn new(
        room: &'a matrix_sdk::Room,
        member: &'a matrix_sdk::room::RoomMember,
    ) -> MembershipHistory<'a> {
        let event = member.event();
        let event_id = event.event_id().map(ToOwned::to_owned);
        let stream: BoxStream<'a, OriginalSyncRoomMemberEvent> = match event_id {
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

        Self { stream }
    }
}

impl Stream for MembershipHistory<'_> {
    type Item = OriginalSyncRoomMemberEvent;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.as_mut().poll_next(cx)
    }
}
