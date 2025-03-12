use dptree::prelude::*;
use matrix_sdk::ruma::events::room::message::AddMentions;
use matrix_sdk::ruma::events::room::message::ForwardThread;
use std::sync::LazyLock;

use std::sync::Arc;

use crate::MediaProxy;
use crate::ReloadableConfig;
use matrix_sdk::Room;
use matrix_sdk::RoomState;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::message::{
    OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
};

mod handlers;
mod request;

/// A new type for request body.
#[derive(Clone, Debug)]
struct RequestBody(String);

/// Represents a handler of incoming request.
#[derive(Debug, Clone)]
pub struct IncomingRequest {
    ev: OriginalRoomMessageEvent,
    room: Room,
}

/// Injected dependencies.
#[derive(Clone)]
pub struct Injected {
    pub config: ReloadableConfig,
    pub prefix: String,
    pub http: reqwest::Client,
    pub pixiv: Option<Arc<pixrs::PixivClient>>,
    pub media_proxy: Option<Arc<MediaProxy>>,
}

pub type EventHandler = Endpoint<'static, DependencyMap, anyhow::Result<()>>;

pub static DISPATCHER: LazyLock<Handler<'static, DependencyMap, anyhow::Result<()>>> =
    LazyLock::new(|| {
        dptree::map(|request: IncomingRequest| {
            use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;

            let body = remove_plain_reply_fallback(request.ev.content.body()).trim();

            RequestBody(body.to_string())
        })
        .branch(
            dptree::filter_map(|body: RequestBody, injected: Injected| {
                let RequestBody(ref body) = body;

                body.strip_prefix(&injected.prefix)
                    .map(|s| RequestBody(s.to_string()))
            })
            .filter_map_async(|text: RequestBody, request: IncomingRequest| async move {
                let RequestBody(ref s) = text;
                let args = shell_words::split(s);

                match args {
                    Ok(args) => match self::request::from_args(args.into_iter()) {
                        Ok(t) => t,
                        Err(e) => {
                            send_error_content(&request.room, e, &request.ev).await;
                            None
                        }
                    },
                    Err(e) => {
                        send_error_content(&request.room, e.into(), &request.ev).await;
                        None
                    }
                }
            })
            .branch(self::handlers::ping::event_handler())
            .branch(self::handlers::help::event_handler())
            .branch(self::handlers::ignore::event_handler())
            .branch(self::handlers::unignore::event_handler())
            .branch(self::handlers::hitokoto::event_handler())
            .branch(self::handlers::pixiv::event_handler())
            .branch(self::handlers::nixpkgs::event_handler())
            .branch(self::handlers::profile::event_handler())
            .branch(self::handlers::room_id::event_handler())
            .branch(self::handlers::user_id::event_handler()),
        )
        .branch(self::handlers::jerryxiao::event_handler())
        .branch(self::handlers::nahida::event_handler())
    });

/// Called when a message is sent.
#[tracing::instrument(skip_all)]
pub async fn on_sync_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    client: matrix_sdk::Client,
    injected: Ctx<Injected>,
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
    let ev = ev.into_full_event(room_id);

    tokio::spawn(async move {
        let request = self::IncomingRequest { ev, room };
        let Ctx(injected) = injected;

        if let ControlFlow::Break(Err(e)) = DISPATCHER
            .dispatch(dptree::deps![request.clone(), injected])
            .await
        {
            send_error_content(&request.room, e, &request.ev).await
        }
    });
}

async fn send_error_content(room: &Room, e: anyhow::Error, ev: &OriginalRoomMessageEvent) {
    use crate::Error;

    let body = RoomMessageEventContent::text_plain(match e.downcast::<crate::Error>() {
        Ok(Error::RequiresReply) => "Replying to a event is required for this command.".to_string(),
        Ok(Error::InvaildArgument { arg, source }) => {
            format!("Invaild argument for {arg}: {source}")
        }
        Ok(Error::MissingArgument(arg)) => format!("Missing argument: {arg}"),
        Ok(Error::UnknownCommand(command)) => format!("Unknown command {command}"),
        Ok(Error::UnexpectedError(e)) => e.to_string(),
        Ok(Error::GraphQLError { service, error }) => {
            format!("GraphQL Error response from {service}: {error:?}")
        }
        Err(e) => {
            tracing::error!("Unexpected error happened: {e:#}");
            format!("Unexpected error happened: {e:#}")
        }
    })
    .make_reply_to(ev, ForwardThread::No, AddMentions::Yes);

    if let Err(e) = room.send(body).await {
        tracing::error!("Unexpected error happened while sending error content: {e:#}");
    }
}
