use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use url::Url;

use super::{IncomingRequest, Injected, RequestBody};

pub fn event_handler() -> super::EventHandler {
    dptree::filter_map(|body: RequestBody| {
        let RequestBody(body) = body;

        body.strip_prefix("@Nahida ").map(Url::parse)
    })
    .endpoint(
        |url: Result<Url, url::ParseError>,
         request: super::IncomingRequest,
         injected: super::Injected| async move {
            let IncomingRequest { ev, room } = request;
            let Injected {
                config,
                http,
                pixiv,
                ..
            } = injected;

            let config = {
                let config = config.0.read().expect("RwLock posioned!");
                config.clone()
            };

            if let Some(content) =
                crate::message::nahida::dispatch(url?, &room, &config, &http, pixiv.as_deref())
                    .await?
            {
                room.send(content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes))
                    .await?;
            }

            Ok(())
        },
    )
}
