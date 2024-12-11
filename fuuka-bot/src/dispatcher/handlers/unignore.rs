use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::Unignore(user_id)].endpoint(
        |request: super::IncomingRequest| async move {
            use crate::RoomExt as _;

            let user_id = request
                .room
                .in_reply_to_target(&request.ev)
                .await?
                .ok_or(crate::Error::RequiresReply)?;
            let account = request.room.client().account();
            account.unignore_user(&user_id).await?;

            request
                .room
                .send(RoomMessageEventContent::text_plain("Done.").make_reply_to(
                    &request.ev,
                    ForwardThread::No,
                    AddMentions::Yes,
                ))
                .await?;

            Ok(())
        },
    )
}
