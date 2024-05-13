//! Responses to messages that are not commands.

mod jerryxiao;
mod nahida;

use matrix_sdk::ruma::events::AnyMessageLikeEventContent;

use self::jerryxiao::fortune;
use self::jerryxiao::jerryxiao;
use self::jerryxiao::jerryxiao_formatted;
use crate::handler::Message;
use crate::Context;

impl Context {
    /// Dispatchs a message.
    pub async fn dispatch_message(
        &self,
        message: Message,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        match message {
            Message::Slash { from, to, text } => jerryxiao(&from, &to, &text)
                .await
                .map(|e| e.map(AnyMessageLikeEventContent::RoomMessage)),
            Message::SlashFormatted { from, to, text } => jerryxiao_formatted(&from, &to, &text)
                .await
                .map(|e| e.map(AnyMessageLikeEventContent::RoomMessage)),
            Message::Nahida(url) => self::nahida::dispatch(&url, &self.http)
                .await
                .map(|e| e.map(AnyMessageLikeEventContent::RoomMessage)),
            Message::Fortune { member, text, prob } => fortune(&member, &text, prob)
                .await
                .map(|e| Some(AnyMessageLikeEventContent::RoomMessage(e))),
        }
    }
}
