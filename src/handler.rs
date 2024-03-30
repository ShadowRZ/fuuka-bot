//! Generic Matrix event callback handler.
#![warn(missing_docs)]
use crate::traits::IntoEventContent;
use crate::{Config, HandlerContext};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, OriginalSyncRoomMessageEvent,
};
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use matrix_sdk::{Client as MatrixClient, RoomState};

/// Called when a message is sent.
#[tracing::instrument(skip_all)]
pub async fn on_sync_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    client: MatrixClient,
    config: Ctx<Config>,
    http: Ctx<reqwest::Client>,
) {
    // It should be a joined room.
    if room.state() != RoomState::Joined {
        return;
    }

    // Ignore messages from ourselves.
    if ev.sender == client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        let info = HandlerContext::new(ev, room, client.homeserver());

        let res = if let Some(commands) = &info.body.strip_prefix(&config.command_prefix) {
            crate::command::dispatch(&http, &config, &info, commands).await
        } else {
            crate::message::dispatch(&http, &config, &info).await
        };

        let Err(e) = res else {
            return;
        };

        let content =
            e.event_content()
                .make_reply_to(&info.ev, ForwardThread::Yes, AddMentions::Yes);

        match info.room.send(content).await {
            Ok(_) => (),
            Err(e) => tracing::error!("Unexpected error happened: {e:?}"),
        }
    });
}

/// Called when a member event is from an invited room.
#[tracing::instrument(skip_all)]
pub async fn on_stripped_member(ev: StrippedRoomMemberEvent, room: Room, client: MatrixClient) {
    // Ignore state events not for ourselves.
    if ev.state_key != client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        let room_id = room.room_id();
        tracing::info!("Autojoining room {}", room_id);
        let mut delay = 2;
        while let Err(e) = room.join().await {
            use tokio::time::{sleep, Duration};
            tracing::warn!("Failed to join room {room_id} ({e:?}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:?})");
                break;
            }
        }
    });
}

/// Called when we have a tombstone event.
#[tracing::instrument(skip_all)]
pub async fn on_room_replace(ev: OriginalSyncRoomTombstoneEvent, room: Room, client: MatrixClient) {
    tokio::spawn(async move {
        let room_id = ev.content.replacement_room;
        tracing::info!("Room replaced, Autojoining new room {}", room_id);
        let mut delay = 2;
        while let Err(e) = client.join_room_by_id(&room_id).await {
            use tokio::time::{sleep, Duration};
            tracing::warn!("Failed to join room {room_id} ({e:?}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:?})");
                break;
            }
        }
        if let Some(room) = client.get_room(room.room_id()) {
            tokio::spawn(async move {
                if let Err(e) = room.leave().await {
                    tracing::error!("Can't leave the original room {} ({e:?})", room.room_id());
                }
            });
        }
    });
}
