use crate::message::{Injected, pixiv::PixivCommand};
use futures_util::{StreamExt, pin_mut};
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use pixrs::{RankingContent, RankingMode};

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    command: PixivCommand,
) -> anyhow::Result<()> {
    let Ctx(Injected { pixiv, config, .. }) = injected;

    let Some(pixiv) = pixiv else {
        return Ok(());
    };

    let content = match command {
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
                            "\n#{idx}: {title} https://www.pixiv.net/artworks/{illust_id} | {tag_str}",
                            idx = idx,
                            title = illust.title,
                            illust_id = illust.illust_id
                        );
                        let this_line_html = format!(
                            "<br/>#{idx}: <a href='https://www.pixiv.net/artworks/{illust_id}'>{title}</a> | {tag_html_str}",
                            idx = idx,
                            title = illust.title,
                            illust_id = illust.illust_id
                        );
                        body.push_str(&this_line);
                        html_body.push_str(&this_line_html);
                        idx += 1;
                    }
                    RoomMessageEventContent::text_html(body, html_body)
                }
                PixivCommand::IllustInfo(illust_id) => {
                    let resp = pixiv.illust_info(illust_id).with_lang("zh").await?;
                    let room_id = room.room_id();
                    let send_r18 = {
                        config.borrow().features.room_pixiv_r18_enabled(room_id)
                    };

                    {
                        let config = config.borrow();
                        let pixiv = &config.pixiv;
                        match crate::services::pixiv::illust::format(
                            resp, pixiv, send_r18, room_id, false,
                        ) {
                            Some(msg) => msg,
                            None => return  Ok(()),
                        }
                    }
                }
            }.make_reply_to(
                ev,
                ForwardThread::No,
                AddMentions::Yes,
            );

    room.send(content).await?;

    Ok(())
}
