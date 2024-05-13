use matrix_sdk::{
    event_handler::EventHandlerHandle,
    room::RoomMember,
    ruma::{
        events::{
            room::message::{
                AddMentions, ForwardThread, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
            },
            AnyMessageLikeEventContent, Mentions,
        },
        OwnedUserId,
    },
    Client, Room,
};

use crate::{Context, RoomMemberExt};

impl Context {
    #[tracing::instrument(
        skip(self, sender),
        fields(
            sender = %sender.user_id(),
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _remind(
        &self,
        target: OwnedUserId,
        sender: RoomMember,
        content: Option<String>,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        self.room.add_event_handler(
            |ev: OriginalSyncRoomMessageEvent,
             client: Client,
             room: Room,
             handle: EventHandlerHandle| async move {
                let ev = ev.into_full_event(room.room_id().into());
                if ev.sender == target {
                    let pill = sender.make_pill();
                    let reminder = content.unwrap_or("You can ask now.".to_string());
                    let content = RoomMessageEventContent::text_html(
                        format!("Cc {} {}", sender.name_or_id(), &reminder),
                        format!("Cc {} {}", pill, &reminder),
                    )
                    .make_reply_to(&ev, ForwardThread::No, AddMentions::Yes)
                    .add_mentions(Mentions::with_user_ids([target]));
                    match room.send(content).await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("Unexpected error happened: {e:#}"),
                    }
                    client.remove_event_handler(handle);
                };
            },
        );

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain("You'll be reminded when the target speaks."),
        )))
    }
}
