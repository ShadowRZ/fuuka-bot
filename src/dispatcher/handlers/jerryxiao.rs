use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};

use super::RequestBody;

#[derive(Clone, Debug)]
enum ResponseType {
    Normal { text: Remaining, reverse: bool },
    Formatted { text: Remaining },
    Fortune { text: Remaining, prob: bool },
}

#[derive(Clone, Debug)]
struct Remaining(String);

pub fn event_handler() -> super::EventHandler {
    dptree::filter_map(|body: RequestBody| {
        let RequestBody(body) = body;

        if let Some(remaining) = body.strip_prefix("//") {
            Some(ResponseType::Formatted {
                text: Remaining(remaining.to_string()),
            })
        } else if let Some(remaining) = body.strip_prefix('/') {
            Some(ResponseType::Normal {
                text: Remaining(remaining.to_string()),
                reverse: false,
            })
        } else if let Some(remaining) = body.strip_prefix("!!") {
            Some(ResponseType::Normal {
                text: Remaining(remaining.to_string()),
                reverse: false,
            })
        } else if let Some(remaining) = body.strip_prefix('\\') {
            Some(ResponseType::Normal {
                text: Remaining(remaining.to_string()),
                reverse: true,
            })
        } else if let Some(remaining) = body.strip_prefix("¡¡") {
            Some(ResponseType::Normal {
                text: Remaining(remaining.to_string()),
                reverse: true,
            })
        } else if let Some(remaining) = body.strip_prefix("@@") {
            Some(ResponseType::Fortune {
                text: Remaining(remaining.to_string()),
                prob: false,
            })
        } else {
            body.strip_prefix("@%")
                .map(|remaining| ResponseType::Fortune {
                    text: Remaining(remaining.to_string()),
                    prob: true,
                })
        }
    })
    .filter(
        |request: super::IncomingRequest, injected: super::Injected| {
            injected
                .config
                .room_jerryxiao_enabled(request.room.room_id())
        },
    )
    .branch(
        dptree::case![ResponseType::Normal { text, reverse }].endpoint(
            |(text, reverse): (Remaining, bool), request: super::IncomingRequest| async move {
                let super::IncomingRequest { ev, room } = request;

                use crate::RoomExt as _;

                let from_sender = &ev.sender;
                let Some(to_sender) = room.in_reply_to_target(&ev).await? else {
                    return Ok(());
                };
                let Some(from_member) = room
                    .get_member(if reverse { &to_sender } else { from_sender })
                    .await?
                else {
                    return Ok(());
                };
                let Some(to_member) = room
                    .get_member(if reverse { from_sender } else { &to_sender })
                    .await?
                else {
                    return Ok(());
                };

                if let Some(content) =
                    crate::message::jerryxiao::jerryxiao(&from_member, &to_member, &text.0).await?
                {
                    room.send(content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes))
                        .await?;
                }

                Ok(())
            },
        ),
    )
    .branch(dptree::case![ResponseType::Formatted { text }].endpoint(
        |(text,): (Remaining,), request: super::IncomingRequest| async move {
            let super::IncomingRequest { ev, room } = request;

            use crate::RoomExt as _;

            let from_sender = &ev.sender;
            let Some(to_sender) = room.in_reply_to_target(&ev).await? else {
                return Ok(());
            };
            let Some(from_member) = room.get_member(from_sender).await? else {
                return Ok(());
            };
            let Some(to_member) = room.get_member(&to_sender).await? else {
                return Ok(());
            };

            if let Some(content) =
                crate::message::jerryxiao::jerryxiao_formatted(&from_member, &to_member, &text.0)
                    .await?
            {
                room.send(content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes))
                    .await?;
            }

            Ok(())
        },
    ))
    .branch(
        dptree::case![ResponseType::Fortune { text, prob }].endpoint(
            |(text, prob): (Remaining, bool), request: super::IncomingRequest| async move {
                let super::IncomingRequest { ev, room } = request;

                let Some(member) = room.get_member(&ev.sender).await? else {
                    return Ok(());
                };

                let content = crate::message::jerryxiao::fortune(&member, &text.0, prob).await?;

                room.send(content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes))
                    .await?;

                Ok(())
            },
        ),
    )
}
