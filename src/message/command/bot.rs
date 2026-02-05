use crate::{
    RoomExt,
    message::{Injected, bot::BotCommand},
};
use file_format::FileFormat;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    media::{MediaFormat, MediaRequestParameters},
    ruma::events::{
        AnyMessageLikeEvent, AnyTimelineEvent,
        room::{
            MediaSource,
            message::{MessageType, OriginalRoomMessageEvent, RoomMessageEvent},
        },
    },
};

#[tracing::instrument(name = "help", skip(ev, room, injected), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    command: BotCommand,
) -> anyhow::Result<()> {
    let client = room.client();

    {
        let sender = &ev.sender;
        let config = injected.config.borrow();
        let admin = config.admin_user.as_ref();

        if admin != Some(sender) {
            return Ok(());
        };
    }

    match command {
        BotCommand::SetAvatar => {
            let ev = room
                .in_reply_to_event(ev)
                .await?
                .ok_or(crate::Error::RequiresReply)?;

            if let AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                RoomMessageEvent::Original(ev),
            )) = ev
                && let MessageType::Image(content) = ev.content.msgtype
            {
                match content.source {
                    MediaSource::Plain(owned_mxc_uri) => {
                        client
                            .account()
                            .set_avatar_url(Some(&owned_mxc_uri))
                            .await?;
                    }
                    MediaSource::Encrypted(encrypted_file) => {
                        let media = client.media();
                        let request = MediaRequestParameters {
                            source: MediaSource::Encrypted(encrypted_file),
                            format: MediaFormat::File,
                        };
                        let data = media.get_media_content(&request, true).await?;

                        let format = FileFormat::from_bytes(&data);
                        let mimetype = format.media_type();

                        let response = media.upload(&mimetype.parse()?, data, None).await?;
                        client
                            .account()
                            .set_avatar_url(Some(&response.content_uri))
                            .await?;
                    }
                }
            } else {
                anyhow::bail!("The replied to event is not an image!");
            }
        }
        BotCommand::SetDisplayName { display_name } => {
            client
                .account()
                .set_display_name(Some(&display_name))
                .await?;
        }
    }

    Ok(())
}
