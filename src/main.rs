mod ip;

use serde::{Deserialize, Serialize};

use const_format::concatcp;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::sync::Mutex;
use std::time::Duration;
use telegram_bot2::models::{ChatId, SendMessageBuilder};
use telegram_bot2::{bot, command, commands, daemons, Bot, BotBuilder, Builder};

use ip::*;

const DIR: &str = "/var/bot";
const STATE_FILE: &str = concatcp!("/state.json");

#[derive(Serialize, Deserialize)]
pub struct State {
    ip: Mutex<Option<Ipv4Addr>>,
    ip_listener: Mutex<Vec<ChatId>>,
}

impl State {
    pub fn save(&self) {
        create_dir_all(DIR).unwrap();

        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(STATE_FILE)
            .unwrap()
            .write_all(serde_json::to_string(self).unwrap().as_bytes())
            .unwrap();
    }

    pub fn load() -> Option<Self> {
        let mut string = String::new();
        File::open(STATE_FILE)
            .ok()?
            .read_to_string(&mut string)
            .ok()?;
        serde_json::from_str(string.as_str()).unwrap()
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            ip: Mutex::new(Option::None),
            ip_listener: Mutex::new(Vec::new()),
        }
    }
}

#[command("/help <cmd>")]
async fn help(bot: &Bot, id: ChatId, cmd: Option<String>) -> Result<(), telegram_bot2::Error> {
    bot.send_message(
        SendMessageBuilder::new(
            id,
            match cmd.as_ref().map(String::as_str) {
                Some("ip") => "/ip <cmd>\n/ip: Show current server ip\n/ip listen: Listen to ip changes. You will receive a notification each time the ip is modified\n/ip unlisten: unregister from ip notifications".to_owned(),
                _ => "Available commands are:\n/ip: request ip information".to_owned(),
            },
        )
        .build(),
    )
    .await?;

    Ok(())
}

#[command("/start")]
async fn start(bot: &Bot, id: ChatId) -> Result<(), telegram_bot2::Error> {
    bot.send_message(
        SendMessageBuilder::new(
            id,
            "Hi, to see a full overview of my features, use /help".to_owned(),
        )
        .build(),
    )
    .await?;
    Ok(())
}

#[bot]
fn bot() -> _ {
    BotBuilder::new()
        .interval(Duration::from_secs(0))
        .timeout(300)
        .with_state(State::load().unwrap_or_default())
        .commands(commands![ip, help])
        .daemons(daemons![ip_daemon])
}
