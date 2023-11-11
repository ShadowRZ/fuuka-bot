use anyhow::Result;
use matrix_sdk::reqwest::Url;
use matrix_sdk::room::Joined;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::ruma::{MxcUri, OwnedUserId};

pub async fn get_reply_target(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Joined,
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

pub async fn get_reply_target_fallback(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Joined,
) -> anyhow::Result<OwnedUserId> {
    if let Some(user_id) = get_reply_target(ev, room).await? {
        Ok(user_id)
    } else {
        Ok(ev.sender.clone())
    }
}

pub fn make_pill(member: &RoomMember) -> String {
    let user_id = member.user_id().as_str();
    let name = member.name();
    format!("<a href=\"https://matrix.to/#/{}\">@{}</a>", user_id, name)
}

pub fn member_name_or_id(member: &RoomMember) -> &str {
    let user_id = member.user_id().as_str();
    member.display_name().unwrap_or(user_id)
}

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
