//! Various helper functions.

use anyhow::Result;
use matrix_sdk::reqwest::Url;
use matrix_sdk::room::Room;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::{MxcUri, OwnedUserId};

/// Given a [OriginalSyncRoomMessageEvent], returns the user ID of the reply target.
pub async fn get_reply_target(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<OwnedUserId>> {
    match &ev.content.relates_to {
        Some(Relation::Reply { in_reply_to }) => {
            let event_id = &in_reply_to.event_id;
            let event = room.event(event_id).await?.event.deserialize()?;
            let ret = event.sender();
            Ok(Some(ret.into()))
        }
        _ => Ok(None),
    }
}

/// Given a [OriginalSyncRoomMessageEvent], returns the user ID of the reply target,
/// it that doesn't exist, returns the user ID of the sender.
pub async fn get_reply_target_fallback(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<OwnedUserId> {
    if let Some(user_id) = get_reply_target(ev, room).await? {
        Ok(user_id)
    } else {
        Ok(ev.sender.clone())
    }
}

/// Constructs a HTML link of the specified [RoomMember], known as the mention "pill".
pub fn make_pill(member: &RoomMember) -> String {
    let user_id = member.user_id();
    let name = member.name();
    format!("<a href=\"{}\">@{}</a>", user_id.matrix_to_uri(), name)
}

/// Returns the display name or the user ID of the specified [RoomMember].
pub fn member_name_or_id(member: &RoomMember) -> &str {
    let user_id = member.user_id().as_str();
    member.display_name().unwrap_or(user_id)
}

/// Returns the HTTP URL of the given [MxcUri], with the specified homeserver
/// using the [Client-Server API](https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3downloadservernamemediaid).
pub fn avatar_http_url(avatar_uri: Option<&MxcUri>, homeserver: &Url) -> Result<Option<Url>> {
    if let Some(avatar_uri) = avatar_uri {
        let (server_name, media_id) = avatar_uri.parts()?;
        let result = homeserver
            .join(format!("/_matrix/media/r0/download/{}/{}", server_name, media_id).as_str())?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

/// Returns the make-up divergence.
pub fn make_divergence(room_hash: u32, event_id_hash: Option<u32>) -> f32 {
    let seed = room_hash + event_id_hash.unwrap_or(0);
    let mut rng = fastrand::Rng::with_seed(seed.into());
    rng.f32() + if rng.bool() { 1.0 } else { 0.0 }
}

/// Given a [nom::error::Error] and the input, returns the [RoomMessageEventContent] to send to the room
pub fn nom_error_message(input: &str, e: nom::error::Error<String>) -> RoomMessageEventContent {
    use nom::Offset;
    let offset = input.offset(&e.input);
    let prefix = &input.as_bytes()[..offset];
    // Count the number of newlines in the first `offset` bytes of input
    let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

    // Find the line that includes the subslice:
    // Find the *last* newline before the substring starts
    let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map(|pos| offset - pos)
        .unwrap_or(0);

    // Find the full line after that newline
    let line = input[line_begin..]
        .lines()
        .next()
        .unwrap_or(&input[line_begin..])
        .trim_end();
    // The (1-indexed) column number is the offset of our substring into that line
    let column_number = line.offset(&e.input) + 1;

    RoomMessageEventContent::text_html(
        format!(
            "Ln {line_number}, Col {column_number}: Expected {expect:?}, Got {got}\n\
             {line}\n{caret:>column_number$}",
            caret = "^",
            expect = e.code,
            got = e
                .input
                .chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or("(EOF)".to_string())
        ),
        format!(
            "Ln {line_number}, Col {column_number}: Expected {expect:?}, Got {got}<br/>\
             <pre><code>{line}\n{caret:>column_number$}</code></pre>",
            caret = "^",
            expect = e.code,
            got = e
                .input
                .chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or("(EOF)".to_string())
        ),
    )
}
