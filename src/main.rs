mod ip;


use serde::{Deserialize, Serialize};

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::sync::Mutex;
use std::time::Duration;
use telegram_bot2::models::ChatId;
use telegram_bot2::{bot, commands, daemons, BotBuilder};

use ip::*;

const STATE_FILE: &str = "./state.json";

#[derive(Serialize, Deserialize)]
pub struct State {
    ip: Mutex<Option<Ipv4Addr>>,
    ip_listener: Mutex<Vec<ChatId>>,
}

impl State {
    pub fn save(&self) {
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

#[bot]
fn bot() -> _ {
    BotBuilder::new()
        .interval(Duration::from_secs(0))
        .timeout(300)
        .with_state(State::load().unwrap_or_default())
        .commands(commands![ip])
        .daemons(daemons![ip_daemon])
}
