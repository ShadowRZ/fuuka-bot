use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

pub fn format(resp: hitokoto_api::Response) -> RoomMessageEventContent {
    let from_who = resp.from_who.unwrap_or_default();

    RoomMessageEventContent::text_html(
        format!(
            "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
        format!(
            "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
    )
}
