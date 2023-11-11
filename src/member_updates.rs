use async_stream::stream;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::OriginalRoomMemberEvent;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::events::AnyTimelineEvent;
use matrix_sdk::ruma::events::SyncStateEvent;
use matrix_sdk::ruma::events::{AnyStateEvent, StateEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::{EventId, OwnedEventId};
use tokio_stream::Stream;

pub struct MemberChanges {
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

    pub fn new_stream(
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

    async fn next(&mut self) -> Option<OriginalRoomMemberEvent> {
        match &self.replaces_state {
            Some(replaces_state) => {
                if let Ok(timeline) = self.room.event(replaces_state).await {
                    let ev = &timeline.event;
                    self.replaces_state = get_replaces_state(ev).await;
                    get_member_event(ev).await
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

async fn get_replaces_state(raw: &Raw<AnyTimelineEvent>) -> Option<OwnedEventId> {
    if let Ok(replaces_state) = raw.get_field::<String>("replaces_state") {
        match replaces_state {
            Some(replaces_state) => {
                if let Ok(ret) = EventId::parse(replaces_state) {
                    Some(ret)
                } else {
                    None
                }
            }
            None => None,
        }
    } else {
        None
    }
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
