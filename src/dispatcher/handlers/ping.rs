use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use time::Duration;

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::Ping].endpoint(|request: super::IncomingRequest| async move {
        let super::IncomingRequest { ev, room } = request;

        let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
        let MilliSecondsSinceUnixEpoch(event_ts) = ev.origin_server_ts;
        let now = Duration::milliseconds(now.into());
        let event_ts = Duration::milliseconds(event_ts.into());
        let delta = now - event_ts;
        let delta_ms = delta.whole_milliseconds();
        let body = if delta_ms >= 2000 {
            format!("Pong after {delta:.3}")
        } else {
            format!("Pong after {}ms", delta_ms)
        };

        room.send(RoomMessageEventContent::text_plain(body).make_reply_to(
            &ev,
            ForwardThread::No,
            AddMentions::Yes,
        ))
        .await?;

        Ok(())
    })
}
