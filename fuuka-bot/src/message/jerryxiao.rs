//! Implments various Jerry Xiao like functions.

use std::collections::HashMap;

use crate::RoomMemberExt;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::Mentions;
use time::macros::format_description;
use time::OffsetDateTime;

/// Constructs the [RoomMessageEventContent] result of Jerry Xiao from the given room, two senders and text.
#[tracing::instrument(
    skip(from_member, to_member),
    fields(
        from_sender = %from_member.user_id(),
        to_sender = %to_member.user_id(),
    ),
    err
)]
pub async fn jerryxiao(
    from_member: &RoomMember,
    to_member: &RoomMember,
    text: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let from_pill = from_member.make_pill();
    let to_pill = to_member.make_pill();
    {
        let mut splited = text.split_whitespace();
        if let Some(arg0) = splited.next() {
            // All bytes >= 0x80 are for non ASCII char encoding in UTF-8
            if !arg0.as_bytes().iter().all(|b| *b >= 0x80) {
                return Ok(None);
            }
            if ["把", "拿", "被", "将", "令", "使", "让", "给", "替"]
                .into_iter()
                .any(|p| arg0.starts_with(p))
            {
                let arg1 = splited.next().unwrap_or_default();
                let arg1 = arg1.strip_suffix('了').unwrap_or(arg1);
                let arg2 = splited.next().unwrap_or_default();
                let arg2 = arg2.strip_suffix('了').unwrap_or(arg2);
                Ok(Some(
                    RoomMessageEventContent::text_html(
                        format!(
                            "@{from} {arg0} @{to} {arg1}了{arg2}",
                            from = from_member.name(),
                            to = to_member.name(),
                        ),
                        format!(
                            "{from} {arg0} {to} {arg1}了{arg2}",
                            from = from_pill,
                            to = to_pill,
                        ),
                    )
                    .add_mentions(Mentions::with_user_ids([
                        from_member.user_id().to_owned(),
                        to_member.user_id().to_owned(),
                    ])),
                ))
            } else {
                let arg1 = splited.next();
                let arg1 = arg1
                    .map(|arg1| " 的".to_owned() + arg1.strip_prefix('了').unwrap_or(arg1))
                    .unwrap_or_default();
                let chars: Vec<char> = arg0.chars().collect();
                if (chars.len() == 2 && chars[0] == chars[1])
                    || (chars.len() == 3 && chars[1] == '了' && chars[0] == chars[2])
                {
                    Ok(Some(
                        RoomMessageEventContent::text_html(
                            format!(
                                "@{} {}了{} @{}{arg1}",
                                from_member.name(),
                                chars[0],
                                chars[0],
                                to_member.name(),
                            ),
                            format!("{} {}了{} {}{arg1}", from_pill, chars[0], chars[0], to_pill),
                        )
                        .add_mentions(Mentions::with_user_ids([
                            from_member.user_id().to_owned(),
                            to_member.user_id().to_owned(),
                        ])),
                    ))
                } else {
                    let arg0 = arg0.strip_suffix('了').unwrap_or(arg0);
                    Ok(Some(
                        RoomMessageEventContent::text_html(
                            format!(
                                "@{} {arg0}了 @{}{arg1}",
                                from_member.name(),
                                to_member.name(),
                            ),
                            format!("{} {arg0}了 {}{arg1}", from_pill, to_pill),
                        )
                        .add_mentions(Mentions::with_user_ids([
                            from_member.user_id().to_owned(),
                            to_member.user_id().to_owned(),
                        ])),
                    ))
                }
            }
        } else {
            Ok(None)
        }
    }
}

/// Constructs the [RoomMessageEventContent] result of Jerry Xiao from the given room,
/// two senders and formatting text.
#[tracing::instrument(
    skip(from_member, to_member),
    fields(
        from_sender = %from_member.user_id(),
        to_sender = %to_member.user_id(),
    ),
    err
)]
pub async fn jerryxiao_formatted(
    from_member: &RoomMember,
    to_member: &RoomMember,
    text: &str,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    if text.contains("${from}") && text.contains("${to}") {
        let text = text.trim();
        let mut text_context = HashMap::new();
        text_context.insert("from".to_string(), format!("@{}", from_member.name_or_id()));
        text_context.insert("to".to_string(), format!("@{}", to_member.name_or_id()));
        let mut html_context = HashMap::new();
        html_context.insert("from".to_string(), from_member.make_pill());
        html_context.insert("to".to_string(), to_member.make_pill());
        Ok(Some(
            RoomMessageEventContent::text_html(
                envsubst::substitute(text, &text_context)?,
                envsubst::substitute(text, &html_context)?,
            )
            .add_mentions(Mentions::with_user_ids([
                from_member.user_id().to_owned(),
                to_member.user_id().to_owned(),
            ])),
        ))
    } else {
        Ok(Some(RoomMessageEventContent::text_plain(
            "No format slot ${from} ${to} found!",
        )))
    }
}

/// Constructs the [RoomMessageEventContent] result of randomdraw from the given room, sender and text.
#[tracing::instrument(
    skip(member),
    fields(
        user_id = %member.user_id(),
    ),
    err
)]
pub async fn fortune(
    member: &RoomMember,
    query: &str,
    prob: bool,
) -> anyhow::Result<RoomMessageEventContent> {
    let user_id = member.user_id();
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
    let user_pill = member.make_pill();
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
