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

use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use url::Url;

use self::link_type::{CrateLinkType, LinkType, PixivLinkType};

/// Dispatch prefixed messages that starts with `@Nahida`.
pub async fn dispatch(
    url: Url,
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
                self::extractors::pixiv::pixiv_illust(
                    pixiv,
                    artwork_id,
                    &config.pixiv,
                    room.room_id(),
                )
                .await
            }
            None => Ok(None),
        },
        LinkType::Generic(url) => self::extractors::generic::extract(client, url).await, // TODO
        LinkType::CannotBeABase => {
            Result::Err(crate::Error::UnexpectedError("URL is a cannot-be-a-base!").into())
        }
    }
}
