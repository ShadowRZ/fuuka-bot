use dptree::prelude::*;
use matrix_sdk::ruma::events::room::message::AddMentions;
use matrix_sdk::ruma::events::room::message::ForwardThread;
use matrix_sdk::ruma::events::{AnyStateEventContent, StateEventContent};
use std::sync::LazyLock;

use std::sync::Arc;

use crate::Config;
use crate::MediaProxy;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::message::{
    OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::Room;
use matrix_sdk::RoomState;

mod event;
mod handlers;

pub struct OutgoingResponse {
    room: Arc<Room>,
    content: OutgoingContent,
}

pub enum OutgoingContent {
    Event(AnyMessageLikeEventContent),
    State {
        state: AnyStateEventContent,
        state_key: <AnyStateEventContent as StateEventContent>::StateKey,
        notify: Option<RoomMessageEventContent>,
    },
    None,
}

pub type EventHandler = Endpoint<'static, DependencyMap, anyhow::Result<OutgoingResponse>>;

pub static DISPATCHER: LazyLock<Handler<'static, DependencyMap, anyhow::Result<OutgoingResponse>>> =
    LazyLock::new(|| {
        dptree::entry()
            .branch(self::handlers::ping::event_handler())
            .branch(self::handlers::help::event_handler())
            .branch(self::handlers::send_avatar::event_handler())
            .branch(self::handlers::ignore::event_handler())
            .branch(self::handlers::unignore::event_handler())
            .branch(self::handlers::hitokoto::event_handler())
            .branch(self::handlers::room_id::event_handler())
            .branch(self::handlers::user_id::event_handler())
            .branch(self::handlers::name_changes::event_handler())
            .branch(self::handlers::avatar_changes::event_handler())
            .branch(self::handlers::pixiv::event_handler())
            .branch(self::handlers::crazy_thursday::event_handler())
            .branch(self::handlers::nixpkgs::event_handler())
            .branch(self::handlers::remind::event_handler())
            .branch(self::handlers::upload_sticker::event_handler())
            .branch(self::handlers::jerryxiao::event_handler())
            .branch(self::handlers::nahida::event_handler())
    });

/// Called when a message is sent.
#[tracing::instrument(skip_all)]
pub async fn on_sync_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    client: matrix_sdk::Client,
    config: Ctx<Arc<Config>>,
    http: Ctx<reqwest::Client>,
    pixiv: Ctx<Option<Arc<pixrs::PixivClient>>>,
    media_proxy: Ctx<Option<Arc<MediaProxy>>>,
) {
    // It should be a joined room.
    if room.state() != RoomState::Joined {
        return;
    }

    // Ignore messages from ourselves.
    if ev.sender == client.user_id().unwrap() {
        return;
    }

    let room_id = room.room_id().to_owned();
    let ev = Arc::new(ev.into_full_event(room_id));
    let room = Arc::new(room);

    tokio::spawn(async move {
        let resp = match self::event::event_from_incoming_event(&ev, &room, &config).await {
            Ok(Some(event)) => {
                if let ControlFlow::Break(content) = DISPATCHER
                    .dispatch(dptree::deps![
                        event,
                        ev.clone(),
                        room.clone(),
                        client,
                        config.0,
                        http.0,
                        pixiv.0,
                        media_proxy.0
                    ])
                    .await
                {
                    Some(content)
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(e) => Some(Ok(OutgoingResponse {
                room: room.clone(),
                content: OutgoingContent::from_error(e, &ev),
            })),
        };

        let resp = resp.map(|res| match res {
            Ok(res) => res,
            Err(e) => OutgoingResponse {
                room: room.clone(),
                content: OutgoingContent::from_error(e, &ev),
            },
        });

        if let Some(OutgoingResponse { room, content }) = resp {
            let resp = match content {
                OutgoingContent::Event(content) => room.send(content).await.map(|_| ()),
                OutgoingContent::State {
                    state,
                    state_key,
                    notify,
                } => {
                    match room
                        .send_state_event_for_key(&state_key, state)
                        .await
                        .map(|_| ())
                    {
                        Ok(()) => {
                            if let Some(notify) = notify {
                                room.send(notify).await.map(|_| ())
                            } else {
                                Ok(())
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                OutgoingContent::None => Ok(()),
            };

            if let Err(e) = resp {
                tracing::warn!("Unexpected error occured: {e:#}");
            }
        }
    });
}

impl Default for OutgoingContent {
    fn default() -> Self {
        Self::None
    }
}

impl OutgoingContent {
    fn from_error(e: anyhow::Error, ev: &OriginalRoomMessageEvent) -> Self {
        use crate::Error;

        let body = RoomMessageEventContent::text_plain(match e.downcast::<crate::Error>() {
            Ok(Error::RequiresReply) => {
                "Replying to a event is required for this command.".to_string()
            }
            Ok(Error::InvaildArgument { arg, source }) => {
                format!("Invaild argument for {arg}: {source}")
            }
            Ok(Error::MissingArgument(arg)) => format!("Missing argument: {arg}"),
            Ok(Error::UnknownCommand(command)) => format!("Unknown command {command}"),
            Ok(Error::UnexpectedError(e)) => e.to_string(),
            Ok(Error::GraphQLError { service, .. }) => {
                format!("GraphQL Error response from {service}!")
            }
            Err(e) => {
                tracing::error!("Unexpected error happened: {e:#}");
                format!("Unexpected error happened: {e:#}")
            }
        })
        .make_reply_to(ev, ForwardThread::No, AddMentions::Yes);

        Self::Event(body.into())
    }
}
