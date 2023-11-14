use crate::utils::make_pill;
use crate::FuukaBotError;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::{events::room::message::RoomMessageEventContent, UserId};
use time::macros::format_description;
use time::OffsetDateTime;

pub async fn make_jerryxiao_event_content(
    room: &Room,
    from_sender: &UserId,
    to_sender: &UserId,
    text: &str,
    reversed: bool,
) -> anyhow::Result<RoomMessageEventContent> {
    let from_member = room
        .get_member(if reversed { to_sender } else { from_sender })
        .await?
        .ok_or(FuukaBotError::ShouldAvaliable)?;
    let to_member = room
        .get_member(if reversed { from_sender } else { to_sender })
        .await?
        .ok_or(FuukaBotError::ShouldAvaliable)?;

    let from_pill = make_pill(&from_member);
    let to_pill = make_pill(&to_member);

    let chars: Vec<char> = text.chars().collect();

    if chars.len() == 2 && chars[0] == chars[1] {
        Ok(RoomMessageEventContent::text_html(
            format!(
                "@{} {}了{} @{}",
                from_member.name(),
                chars[0],
                chars[1],
                to_member.name(),
            ),
            format!("{} {}了{} {}", from_pill, chars[0], chars[1], to_pill),
        ))
    } else if let Some(remaining) = text.strip_prefix('把') {
        Ok(RoomMessageEventContent::text_html(
            format!(
                "@{} {} @{} {}",
                from_member.name(),
                '把',
                to_member.name(),
                remaining,
            ),
            format!("{} {} {} {}", from_pill, '把', to_pill, remaining),
        ))
    } else if let Some(remaining) = text.strip_prefix('被') {
        Ok(RoomMessageEventContent::text_html(
            format!(
                "@{} {} @{} {}",
                from_member.name(),
                '被',
                to_member.name(),
                remaining,
            ),
            format!("{} {} {} {}", from_pill, '被', to_pill, remaining),
        ))
    } else if chars.len() == 3 && chars[1] == '一' {
        Ok(RoomMessageEventContent::text_html(
            format!(
                "@{} {}了{} @{}",
                from_member.name(),
                chars[0],
                String::from_iter(&chars[1..]),
                to_member.name(),
            ),
            format!(
                "{} {}了{} {}",
                from_pill,
                chars[0],
                String::from_iter(&chars[1..]),
                to_pill,
            ),
        ))
    } else if let Some(remaining) = text.strip_prefix("发动") {
        Ok(RoomMessageEventContent::text_html(
            format!(
                "@{} 向 @{} 发动了{}",
                from_member.name(),
                to_member.name(),
                remaining,
            ),
            format!("{} 向 {} 发动了{}", from_pill, to_pill, remaining),
        ))
    } else {
        let splited: Vec<&str> = text.split_whitespace().collect();
        if splited.len() == 2 {
            Ok(RoomMessageEventContent::text_html(
                format!(
                    "@{} {}了 @{} 的{}",
                    from_member.name(),
                    splited[0],
                    to_member.name(),
                    splited[1],
                ),
                format!(
                    "{} {}了 {} 的{}",
                    from_pill, splited[0], to_pill, splited[1]
                ),
            ))
        } else {
            Ok(RoomMessageEventContent::text_html(
                format!("@{} {}了 @{}", from_member.name(), text, to_member.name()),
                format!("{} {}了 {}", from_pill, text, to_pill),
            ))
        }
    }
}

pub async fn make_randomdraw_event_content(
    room: &Room,
    user_id: &UserId,
    query: &str,
    prob: bool,
) -> anyhow::Result<RoomMessageEventContent> {
    let member = room
        .get_member(user_id)
        .await?
        .ok_or(FuukaBotError::ShouldAvaliable)?;
    let hash = crc32fast::hash(user_id.as_bytes());
    let date = OffsetDateTime::now_utc();
    let format = format_description!("[year][month][day]");
    let seed: u64 = if query.is_empty() {
        let formatted = format!("{}{}", date.format(&format)?, hash);
        formatted.parse()?
    } else {
        let query_hash = crc32fast::hash(query.as_bytes());
        let right_hash = crc32fast::hash(format!("{}{}", date.format(&format)?, hash).as_bytes());
        // https://stackoverflow.com/a/67041964
        let result = {
            let l_bytes = query_hash.to_ne_bytes();
            let r_bytes = right_hash.to_ne_bytes();
            let mut result: [u8; 8] = [0; 8];
            let (left, right) = result.split_at_mut(l_bytes.len());
            left.copy_from_slice(&l_bytes);
            right.copy_from_slice(&r_bytes);
            result
        };
        u64::from_ne_bytes(result)
    };
    let mut rng = fastrand::Rng::with_seed(seed);
    let draw_result = rng.u32(0..=10000) as f32 / 10000.0;
    let result_type = rng.bool();
    let user_pill = make_pill(&member);
    let result = if prob {
        let result = if result_type {
            draw_result
        } else {
            1.0 - draw_result
        };
        format!("{:.2}%", result * 100.0)
    } else {
        const CHOICE: [&str; 7] = ["大凶", "凶", "小凶", "尚可", "小吉", "吉", "大吉"];
        const MAXIDX: usize = CHOICE.len() - 1;
        let mut resultidx = (draw_result * (CHOICE.len() as f32)) as usize;
        resultidx = if resultidx > MAXIDX {
            MAXIDX
        } else {
            resultidx
        };
        CHOICE[resultidx].to_string()
    };

    let content = if query.is_empty() {
        if prob {
            let luck_string = if result_type {
                "行大运"
            } else {
                "倒大霉"
            };
            RoomMessageEventContent::text_html(
                format!(
                    "你好, @{}\n汝今天{}概率是 {}",
                    member.name(),
                    luck_string,
                    result
                ),
                format!(
                    "你好, {}<br/>汝今天{}概率是 {}",
                    user_pill, luck_string, result
                ),
            )
        } else {
            RoomMessageEventContent::text_html(
                format!("你好, @{}\n汝的今日运势: {}", member.name(), result),
                format!("你好, {}<br/>汝的今日运势: {}", user_pill, result),
            )
        }
    } else if prob {
        let happen_or_not_string = if result_type { "发生" } else { "不发生" };
        RoomMessageEventContent::text_html(
            format!(
                "你好, @{}\n所求事项: {}\n结果: 此事有 {} 的概率{}",
                member.name(),
                query,
                result,
                happen_or_not_string
            ),
            format!(
                "你好, {}<br/>所求事项: {}<br/>结果: 此事有 {} 的概率{}",
                user_pill, query, result, happen_or_not_string
            ),
        )
    } else {
        RoomMessageEventContent::text_html(
            format!(
                "你好, @{}\n所求事项: {}\n结果: {}",
                member.name(),
                query,
                result
            ),
            format!(
                "你好, {}<br/>所求事项: {}<br/>结果: {}",
                user_pill, query, result
            ),
        )
    };
    Ok(content)
}
