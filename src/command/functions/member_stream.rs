use async_stream::stream;
use futures_util::Stream;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::OriginalSyncRoomMemberEvent;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::events::AnySyncStateEvent;
use matrix_sdk::ruma::events::AnySyncTimelineEvent;
use matrix_sdk::ruma::events::SyncStateEvent;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::{EventId, OwnedEventId};

/// Creates a new [Stream] that outputs a series of [OriginalSyncRoomMemberEvent] starting from the given [SyncRoomMemberEvent].
pub fn member_state_stream(
    room: &Room,
    ev: SyncRoomMemberEvent,
) -> impl Stream<Item = OriginalSyncRoomMemberEvent> + '_ {
    stream! {
        if let SyncStateEvent::Original(ev) = ev {
            let mut changes = MemberChanges::new(room, &ev);
            while let Some(member) = changes.next().await {
                yield member
            }
        }
    }
}

/// Represents a member changes internal state.
struct MemberChanges {
    replaces_state: Option<OwnedEventId>,
    room: Room,
}

impl MemberChanges {
    fn new(room: &Room, ev: &OriginalSyncRoomMemberEvent) -> MemberChanges {
        MemberChanges {
            room: room.clone(),
            replaces_state: Some(ev.event_id.clone()),
        }
    }

    async fn next(&mut self) -> Option<OriginalSyncRoomMemberEvent> {
        match &self.replaces_state {
            Some(replaces_state) => {
                if let Ok(timeline) = self.room.event(replaces_state, None).await {
                    use matrix_sdk::deserialized_responses::TimelineEventKind;
                    let TimelineEventKind::PlainText { event } = timeline.kind else {
                        // Member Event currently unencrypted
                        return None;
                    };
                    self.replaces_state = Self::get_replaces_state(&event).await;
                    Self::get_member_event(&event).await
                } else {
                    None
                }
            }
            None => None,
        }
    }

    async fn get_replaces_state(raw: &Raw<AnySyncTimelineEvent>) -> Option<OwnedEventId> {
        raw.get_field::<String>("replaces_state")
            .ok()
            .flatten()
            .and_then(|e| EventId::parse(e).ok())
    }

    async fn get_member_event(
        raw: &Raw<AnySyncTimelineEvent>,
    ) -> Option<OriginalSyncRoomMemberEvent> {
        if let Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(
            SyncStateEvent::Original(orig),
        ))) = raw.deserialize()
        {
            Some(orig)
        } else {
            None
        }
    }
}
