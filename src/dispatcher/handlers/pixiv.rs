use futures_util::{pin_mut, StreamExt};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use pixrs::{RankingContent, RankingMode};

use crate::dispatcher::request::pixiv::PixivCommand;

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::Pixiv(command)].endpoint(|
        command: PixivCommand,
        request: super::IncomingRequest, injected: super::Injected| async move {
            let super::IncomingRequest { ev, room } = request;
            let super::Injected { pixiv, config, .. } = injected;

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
                    let resp = pixiv.illust_info(illust_id).await?;

                    let send_r18 = config.room_pixiv_r18_enabled(room.room_id());
                    let is_r18 = match resp.restriction {
                        pixrs::Restriction::General => false,
                        pixrs::Restriction::R18 => true,
                        pixrs::Restriction::R18G => true,
                        _ => true,
                    };

                    if is_r18 && !send_r18 {
                        return Ok(())
                    };

                    let tag_str = resp
                        .tags
                        .tags
                        .iter()
                        .map(|tag| {
                            format!(
                                "#{tag}{translated}",
                                tag = tag.tag,
                                translated = tag
                                    .translation
                                    .get("en")
                                    .map(|s| format!(" ({s})"))
                                    .unwrap_or_default()
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    let tag_html_str = resp
                        .tags
                        .tags
                        .iter()
                        .map(|tag| {
                            format!(
                                "<font color='#3771bb'>#{tag}</font>{translated}",
                                tag = tag.tag,
                                translated = tag
                                    .translation
                                    .get("en")
                                    .map(|s| format!(" ({s})"))
                                    .unwrap_or_default()
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    // Specials
                    let specials_str = config
                        .pixiv_check_for_traps(&resp.tags, room.room_id())
                        .map(|s| format!("\n#{s}诱捕器"))
                        .unwrap_or_default();
                    let specials_str_html = config
                        .pixiv_check_for_traps(&resp.tags, room.room_id())
                        .map(|s| format!("<br/><b><font color='#d72b6d'>#{s}诱捕器</font></b>"))
                        .unwrap_or_default();
                    let body = format!(
                        "{title} https://www.pixiv.net/artworks/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
                        title = resp.title,
                        id = resp.id,
                        author = resp.user_name
                    );
                    let html_body = format!(
                        "<a href='https://www.pixiv.net/artworks/{id}'>{title}</a><br/>{tag_html_str}<br/>Author: {author}{specials_str_html}",
                        title = resp.title,
                        id = resp.id,
                        author = resp.user_name
                    );

                    RoomMessageEventContent::text_html(body, html_body)
                }
            }.make_reply_to(
                &ev,
                ForwardThread::No,
                AddMentions::Yes,
            );

            room.send(content).await?;

            Ok(())
    })
}
