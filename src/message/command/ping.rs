use crate::message::Injected;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use time::Duration;

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let _ = injected;

    let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
    let MilliSecondsSinceUnixEpoch(event_ts) = ev.origin_server_ts;
    let now = Duration::milliseconds(now.into());
    let event_ts = Duration::milliseconds(event_ts.into());
    let delta = now - event_ts;
    let delta_ms = delta.whole_milliseconds();
    let body = if delta_ms >= 2000 {
        format!("Pong after {delta:.3}")
    } else {
        format!("Pong after {delta_ms}ms")
    };

    room.send(RoomMessageEventContent::text_plain(body).make_reply_to(
        ev,
        ForwardThread::No,
        AddMentions::Yes,
    ))
    .await?;

    Ok(())
}
