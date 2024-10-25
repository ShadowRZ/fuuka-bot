use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use time::{macros::offset, OffsetDateTime, Weekday};

use crate::Context;

impl Context {
    #[tracing::instrument(
        skip(self),
        fields(
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _crazy_thursday(
        &self,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
        let body = if now.weekday() != Weekday::Thursday {
            let date = now.date().next_occurrence(time::Weekday::Thursday);
            let target = date.with_hms(0, 0, 0)?.assume_offset(offset!(+8));
            let dur = target - now;
            {
                let whole_seconds = dur.whole_seconds().unsigned_abs();
                let seconds = whole_seconds % 60;
                let whole_minutes = dur.whole_minutes().unsigned_abs();
                let minutes = whole_minutes % 60;
                let whole_hours = dur.whole_hours().unsigned_abs();
                let hours = whole_hours % 24;
                let days = dur.whole_days();
                format!("Time until next thursday ({date}): {days} days, {hours:0>2}:{minutes:0>2}:{seconds:0>2}")
            }
        } else {
            "Crazy Thursday!".to_string()
        };

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_plain(body),
        )))
    }
}
