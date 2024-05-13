use file_format::FileFormat;
use matrix_sdk::{
    ruma::events::{
        room::message::{MessageType, RoomMessageEventContent},
        AnyMessageLikeEvent, AnyMessageLikeEventContent, AnyTimelineEvent, MessageLikeEvent,
    },
    Room,
};

use crate::{Context, Error};

impl Context {
    #[tracing::instrument(
        skip(self),
        fields(
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _upload_sticker(
        &self,
        ev: AnyTimelineEvent,
        pack_name: String,
        sticker_room: Room,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        match ev {
            AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                MessageLikeEvent::Original(ev),
            )) => {
                let content = ev.content;
                match content.msgtype {
                    MessageType::File(event_content) => {
                        let name = event_content
                            .filename
                            .clone()
                            .unwrap_or(format!("{}", ev.origin_server_ts.0));
                        let data = self
                            .room
                            .client()
                            .media()
                            .get_file(&event_content, false)
                            .await?
                            .ok_or(Error::UnexpectedError("File has no data!"))?;
                        let format = FileFormat::from_bytes(&data);
                        let mimetype = format.media_type();
                        if mimetype != "application/zip" {
                            return Result::Err(
                                Error::UnexpectedError("File is not a ZIP file!").into(),
                            );
                        }
                        let content = super::functions::prepare_sticker_upload_event_content(
                            &self.room.client(),
                            data,
                            pack_name,
                        )
                        .await?;
                        sticker_room
                            .send_state_event_for_key(&name, content)
                            .await?;
                        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                            RoomMessageEventContent::text_plain("Done."),
                        )))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}
