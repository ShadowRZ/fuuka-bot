use crate::utils::make_pill;
use crate::FuukaBotError;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::{events::room::message::RoomMessageEventContent, UserId};

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
