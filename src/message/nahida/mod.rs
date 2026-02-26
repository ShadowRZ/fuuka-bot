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

use anyhow::Context as _;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use tracing::Instrument;
use url::Url;

use crate::Context;

use self::link_type::{CrateLinkType, LinkType, PixivLinkType};

#[tracing::instrument(name = "nahida", skip(ev, room, context))]
pub(super) async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    url: Url,
) -> anyhow::Result<()> {
    if let Some(content) = crate::message::nahida::dispatch(url.clone(), ev, room, context)
        .await
        .context(format!("Failed to process {url}"))?
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
    context: &crate::Context,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    use crate::Context;

    let Context {
        http: client,
        features,
        crates,
        ..
    } = context;

    match url.try_into()? {
        LinkType::Crates(CrateLinkType::CrateInfo { name, version }) => {
            self::extractors::crates::crates_crate(name, version, crates).await
        }
        LinkType::Pixiv(PixivLinkType::Artwork(artwork_id)) => match &context.pixiv {
            Some((pixiv, context)) => {
                let send_r18 = context.r18 && features.room_pixiv_r18_enabled(room.room_id());
                self::extractors::pixiv::pixiv_illust(
                    ev, room, pixiv, client, artwork_id, context, send_r18,
                )
                .instrument(tracing::info_span!("pixiv")) // TODO
                .await
            }
            None => Ok(None),
        },
        LinkType::Generic(url) => self::extractors::generic::extract(client, url).await,
        LinkType::CannotBeABase => {
            anyhow::bail!("URL is a cannot-be-a-base!")
        }
    }
}
