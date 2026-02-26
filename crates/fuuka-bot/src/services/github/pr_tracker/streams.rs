use cronchik::CronSchedule;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct CronStream {
    cron: Box<CronSchedule>,
}

impl CronStream {
    pub fn new(cron: Box<CronSchedule>) -> Self {
        Self { cron }
    }

    pub async fn wait_for_next_tick(&self) {
        let now = OffsetDateTime::now_utc();
        let next = self.cron.next_time_from(now);

        let duration = next - now;

        tokio::time::sleep(duration.unsigned_abs()).await;
    }
}
