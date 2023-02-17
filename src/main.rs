use std::time::Duration;
use telegram_bot2::{BotBuilder, bot};

#[bot]
fn bot() -> _ {
    BotBuilder::new()
        .interval(Duration::from_secs(0))
        .timeout(300)
}