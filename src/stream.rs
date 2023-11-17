//! Various streams for the bot.
#![warn(missing_docs)]
use async_stream::stream;
use futures_util::Stream;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::OriginalRoomMemberEvent;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::events::AnyTimelineEvent;
use matrix_sdk::ruma::events::SyncStateEvent;
use matrix_sdk::ruma::events::{AnyStateEvent, StateEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::{EventId, OwnedEventId};

/// A set of stream factories for the bot.
pub struct StreamFactory {}

impl StreamFactory {
    /// Creates a new [Stream] that outputs a series of [OriginalRoomMemberEvent] starting from the given [SyncRoomMemberEvent].
    pub fn member_state_stream(
        room: &Room,
        ev: SyncRoomMemberEvent,
    ) -> impl Stream<Item = OriginalRoomMemberEvent> + '_ {
        stream! {
            if let SyncStateEvent::Original(ev) = ev {
                let event = ev.into_full_event(room.room_id().into());
                let mut changes = MemberChanges::new(room, &event);
                while let Some(member) = changes.next().await {
                    yield member
                }
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
    fn new(room: &Room, ev: &OriginalRoomMemberEvent) -> MemberChanges {
        MemberChanges {
            room: room.clone(),
            replaces_state: Some(ev.event_id.clone()),
        }
    }

    async fn next(&mut self) -> Option<OriginalRoomMemberEvent> {
        match &self.replaces_state {
            Some(replaces_state) => {
                if let Ok(timeline) = self.room.event(replaces_state).await {
                    let ev = &timeline.event;
                    self.replaces_state = Self::get_replaces_state(ev).await;
                    Self::get_member_event(ev).await
                } else {
                    None
                }
            }
            None => None,
        }
    }

    async fn get_replaces_state(raw: &Raw<AnyTimelineEvent>) -> Option<OwnedEventId> {
        raw.get_field::<String>("replaces_state")
            .ok()
            .flatten()
            .and_then(|e| EventId::parse(e).ok())
    }

    async fn get_member_event(raw: &Raw<AnyTimelineEvent>) -> Option<OriginalRoomMemberEvent> {
        if let Ok(AnyTimelineEvent::State(AnyStateEvent::RoomMember(StateEvent::Original(orig)))) =
            raw.deserialize()
        {
            Some(orig)
        } else {
            None
        }
    }
}
