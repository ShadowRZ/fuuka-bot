//! Handler for prefixed messages that starts with `@Nahida`.
//!
//! > [Nahida](https://genshin-impact.fandom.com/wiki/Nahida) is a character from _Genshin Impact_.
//!
//! ## Usage
//!
//! Send `@Nahida` followed by a supported URL, example:
//!
//! ```text
//! # Outputs infomation for Rust crate syn
//! @Nahida https://crates.io/crates/syn
//! ```

mod extractors;
mod link_type;

use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use tracing::Instrument;
use url::Url;

use self::link_type::{CrateLinkType, LinkType, PixivLinkType};

use super::Injected;

#[tracing::instrument(name = "nahida", skip(ev, room, injected))]
pub(super) async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    url: Url,
) -> anyhow::Result<()> {
    let Ctx(Injected {
        config,
        http,
        pixiv,
        ..
    }) = injected;

    let config = {
        let config = config.borrow();
        config.clone()
    };

    if let Some(content) =
        crate::message::nahida::dispatch(url, ev, room, &config, http, pixiv.as_deref()).await?
    {
        room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
            .await?;
    }

    Ok(())
}

/// Dispatch prefixed messages that starts with `@Nahida`.
async fn dispatch(
    url: Url,
    ev: &OriginalRoomMessageEvent,
    room: &matrix_sdk::Room,
    config: &crate::Config,
    client: &reqwest::Client,
    pixiv: Option<&pixrs::PixivClient>,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    match url.try_into()? {
        LinkType::Crates(CrateLinkType::CrateInfo { name, version }) => {
            self::extractors::crates::crates_crate(name, version, client).await
        }
        LinkType::Pixiv(PixivLinkType::Artwork(artwork_id)) => match pixiv {
            Some(pixiv) => {
                let send_r18 =
                    config.pixiv.r18 && config.features.room_pixiv_r18_enabled(room.room_id());
                self::extractors::pixiv::pixiv_illust(
                    ev,
                    room,
                    pixiv,
                    client,
                    artwork_id,
                    &config.pixiv,
                    send_r18,
                )
                .instrument(tracing::info_span!("pixiv")) // TODO
                .await
            }
            None => Ok(None),
        },
        LinkType::Generic(url) => self::extractors::generic::extract(client, url).await,
        LinkType::CannotBeABase => {
            Result::Err(crate::Error::UnexpectedError("URL is a cannot-be-a-base!").into())
        }
    }
}
