use futures_util::{pin_mut, StreamExt};
use matrix_sdk::{
    event_handler::Ctx,
    ruma::events::{room::message::RoomMessageEventContent, AnyMessageLikeEventContent},
};
use pixrs::{RankingContent, RankingMode};

use crate::{handler::PixivCommand, Context, IllustTagsInfoExt};

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
                let specials_str = if resp.tags.has_any_tag(&[
                    "調教",
                    "束縛",
                    "機械姦",
                    "緊縛",
                    "縛り",
                    "鼻フック",
                    "監禁",
                    "口枷",
                ]) {
                    "\n#空指针诱捕器"
                } else if resp.tags.has_any_tag(&["ショタ", "触手", "獣人", "人外"]) {
                    "\n#草方块诱捕器"
                } else {
                    ""
                };
                let specials_str_html = if resp.tags.has_any_tag(&[
                    "調教",
                    "束縛",
                    "機械姦",
                    "緊縛",
                    "縛り",
                    "鼻フック",
                    "監禁",
                    "口枷",
                ]) {
                    "<br/><b><font color='#d72b6d'>#空指针诱捕器</font></b>"
                } else if resp.tags.has_any_tag(&["ショタ", "触手", "獣人", "人外"]) {
                    "<br/><b><font color='#d72b6d'>#草方块诱捕器</font></b>"
                } else {
                    ""
                };
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
