use matrix_sdk::{
    room::RoomMember,
    ruma::events::{
        room::message::{ImageMessageEventContent, MessageType, RoomMessageEventContent},
        AnyMessageLikeEventContent,
    },
};

use crate::{Context, RoomMemberExt};

impl Context {
    #[tracing::instrument(
        skip(self, member),
        fields(
            user_id = %member.user_id(),
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _send_avatar(
        &self,
        member: RoomMember,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        match member.avatar_url() {
            Some(avatar_url) => {
                let name = member.name_or_id();
                let info =
                    super::functions::get_image_info(avatar_url, &self.room.client()).await?;
                Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::new(MessageType::Image(
                        ImageMessageEventContent::plain(
                            format!("[Avatar of {name}]"),
                            avatar_url.into(),
                        )
                        .info(Some(Box::new(info))),
                    )),
                )))
            }
            None => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                RoomMessageEventContent::text_plain("The user has no avatar."),
            ))),
        }
    }
}
