use crate::message::{Injected, pixiv::PixivCommand};
use futures_util::{StreamExt, pin_mut};
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};
use pixrs::{PixivClient, RankingContent, RankingMode};

#[tracing::instrument(name = "pixiv", skip_all)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    command: PixivCommand,
) -> anyhow::Result<()> {
    let Ctx(Injected {
        pixiv,
        config,
        http,
        ..
    }) = injected;

    let Some(pixiv) = pixiv else {
        return Ok(());
    };

    let content = match command {
        PixivCommand::Ranking => format_ranking(pixiv).await?,
        PixivCommand::IllustInfo(illust_id) => {
            let config = config.borrow().clone();
            send_illust(ev, room, pixiv, http, &config, illust_id).await?;

            return Ok(());
        }
    }
    .make_reply_to(ev, ForwardThread::No, AddMentions::Yes);

    room.send(content).await?;

    Ok(())
}

#[tracing::instrument(name = "ranking", skip_all, fields(ranking = "daily"), err)]
async fn format_ranking(pixiv: &PixivClient) -> anyhow::Result<RoomMessageEventContent> {
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
    Ok(RoomMessageEventContent::text_html(body, html_body))
}

#[tracing::instrument(name = "illust", skip_all, fields(illust_id = %illust_id), err)]
async fn send_illust(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    pixiv: &pixrs::PixivClient,
    http: &reqwest::Client,
    config: &crate::Config,
    illust_id: i32,
) -> anyhow::Result<()> {
    let room_id = room.room_id();
    let send_r18 = config.pixiv.r18 && config.features.room_pixiv_r18_enabled(room_id);
    crate::services::pixiv::illust::send(ev, room, pixiv, http, &config.pixiv, illust_id, send_r18)
        .await
}
