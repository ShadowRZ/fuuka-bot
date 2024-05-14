use futures_util::{pin_mut, StreamExt};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::{
    event_handler::Ctx,
    ruma::events::{room::message::RoomMessageEventContent, AnyMessageLikeEventContent},
};
use pixrs::{IllustTagsInfo, RankingContent, RankingMode};

use crate::{config::TrapConfig, handler::PixivCommand, Context, IllustTagsInfoExt};

impl Context {
    #[tracing::instrument(
        skip(self),
        fields(
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _pixiv(
        &self,
        command: PixivCommand,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let Ctx(Some(ref pixiv)) = self.pixiv else {
            return Ok(None);
        };
        match command {
            PixivCommand::Ranking => {
                let resp = pixiv
                    .ranking_stream(RankingMode::Daily, RankingContent::Illust, None)
                    .await
                    .take(5);
                let mut body = String::from("Pixiv Ranking: (Illust/Daily)");
                let mut html_body = String::from("<b>Pixiv Ranking: (Illust/Daily)</b>");
                let mut idx = 1;
                pin_mut!(resp);
                while let Some(illust) = resp.next().await {
                    let illust = illust?;
                    let tag_str = illust
                        .tags
                        .iter()
                        .map(|str| format!("#{str}"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    let tag_html_str = illust
                        .tags
                        .iter()
                        .map(|str| format!("<font color='#3771bb'>#{str}</font>"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    let this_line = format!(
                        "\n#{idx}: {title} https://pixiv.net/i/{illust_id} | {tag_str}",
                        idx = idx,
                        title = illust.title,
                        illust_id = illust.illust_id
                    );
                    let this_line_html = format!(
                        "<br/>#{idx}: <a href='https://pixiv.net/i/{illust_id}'>{title}</a> | {tag_html_str}",
                        idx = idx,
                        title = illust.title,
                        illust_id = illust.illust_id
                    );
                    body.push_str(&this_line);
                    html_body.push_str(&this_line_html);
                    idx += 1;
                }
                Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_html(body, html_body),
                )))
            }
            PixivCommand::IllustInfo(illust_id) => {
                let resp = pixiv.illust_info(illust_id).await?;
                let tag_str = resp
                    .tags
                    .tags
                    .iter()
                    .map(|tag| format!("#{tag}", tag = tag.tag))
                    .collect::<Vec<String>>()
                    .join(" ");
                let tag_html_str = resp
                    .tags
                    .tags
                    .iter()
                    .map(|tag| format!("<font color='#3771bb'>#{tag}</font>", tag = tag.tag))
                    .collect::<Vec<String>>()
                    .join(" ");
                // Specials
                let specials_str = self
                    .config
                    .pixiv
                    .traps
                    .check_for_traps(&resp.tags, self.room.room_id())
                    .map(|s| format!("\n#{s}诱捕器"))
                    .unwrap_or_default();
                let specials_str_html = self
                    .config
                    .pixiv
                    .traps
                    .check_for_traps(&resp.tags, self.room.room_id())
                    .map(|s| format!("<br/><b><font color='#d72b6d'>#{s}诱捕器</font></b>"))
                    .unwrap_or_default();
                let body = format!(
                    "{title} https://pixiv.net/i/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
                    title = resp.title,
                    id = resp.id,
                    author = resp.user_name
                );
                let html_body = format!(
                    "<a href='https://pixiv.net/i/{id}'>{title}</a><br/>{tag_html_str}<br/>Author: {author}{specials_str_html}",
                    title = resp.title,
                    id = resp.id,
                    author = resp.user_name
                );
                Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_html(body, html_body),
                )))
            }
        }
    }
}

impl TrapConfig {
    fn check_for_traps(&self, tags: &IllustTagsInfo, room_id: &RoomId) -> Option<&str> {
        if let Some(infos) = self.room_scoped_config.get(room_id) {
            for item in infos {
                if tags.has_any_tag(
                    &item
                        .required_tags
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>(),
                ) {
                    return Some(&item.target);
                }
            }
        } else {
            for item in &self.global_config {
                if tags.has_any_tag(
                    &item
                        .required_tags
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>(),
                ) {
                    return Some(&item.target);
                }
            }
        }

        None
    }
}
