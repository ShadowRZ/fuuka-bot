use crate::message::Injected;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let admin = {
        let config = injected.config.borrow();
        let admin = &config.admin_user;

        *admin == Some(ev.sender.clone())
    };

    if !admin {
        return Ok(());
    }

    if !room.is_direct().await? {
        room.send(
            RoomMessageEventContent::text_plain("This command is only avaliable in a DM!")
                .make_reply_to(ev, ForwardThread::No, AddMentions::Yes),
        )
        .await?;
        return Ok(());
    }

    let client = room.client();
    let joined = client.joined_rooms();
    let mut body = "Joined rooms: \n".to_string();
    let mut html_body = "Joined rooms: <br>".to_string();
    for room in joined {
        let name = room.display_name().await?;
        body.push_str(&format!(
            "- {name} ({room_id}){dm}\n",
            room_id = room.room_id(),
            dm = if room.is_direct().await? { " (DM)" } else { "" }
        ));
        html_body.push_str(&format!(
            "- {name} ({room_id}){dm}<br>",
            room_id = room.room_id(),
            dm = if room.is_direct().await? { " (DM)" } else { "" }
        ));
    }

    room.send(
        RoomMessageEventContent::text_html(body, html_body)
            .make_reply_to(ev, ForwardThread::No, AddMentions::Yes),
    )
    .await?;

    Ok(())
}
