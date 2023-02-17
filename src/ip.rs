use crate::State;
use std::net::Ipv4Addr;
use std::str::FromStr;
use telegram_bot2::log::error;
use telegram_bot2::models::{ChatId, SendMessageBuilder};
use telegram_bot2::{command, daemon, Bot, BotState, Builder};

async fn notify_listeners(state: &State, new_ip: Ipv4Addr, bot: &Bot) {
    let ids = Vec::clone(&state.ip_listener.lock().unwrap());
    for id in ids {
        bot.send_message(SendMessageBuilder::new(id, format!("New IPv4: {}", new_ip)).build())
            .await
            .unwrap();
    }

    *state.ip.lock().unwrap() = Some(new_ip);
    state.save();
}

pub enum IpSubcommand {
    Listen,
    Unlisten,
}

impl FromStr for IpSubcommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match String::from(s).to_ascii_lowercase().as_str() {
            "listen" => Ok(IpSubcommand::Listen),
            "unlisten" => Ok(IpSubcommand::Unlisten),
            _ => Err(()),
        }
    }
}

#[command("/ip <cmd>")]
pub async fn ip(
    state: &BotState<State>,
    bot: &Bot,
    id: ChatId,
    cmd: Option<IpSubcommand>,
) -> Result<(), ()> {
    match cmd {
        None => {
            let Some(ip) = public_ip::addr_v4().await else {
                bot.send_message(SendMessageBuilder::new(id, "Could not get public IPv4".to_string()).build()).await.unwrap();
                return Err(());
            };

            if state.ip.lock().unwrap().is_none() {
                *state.ip.lock().unwrap() = Some(ip);
            }

            let current_ip = state.ip.lock().unwrap().unwrap();

            if ip != current_ip {
                notify_listeners(state, ip, bot).await;
            } else {
                bot.send_message(
                    SendMessageBuilder::new(id, format!("Current IPv4: {}", ip)).build(),
                )
                .await
                .unwrap();
            }
        }
        Some(IpSubcommand::Listen) => {
            let msg = {
                let mut set = state.ip_listener.lock().unwrap();

                if set.contains(&id) {
                    SendMessageBuilder::new(id, "You were already listening".to_string()).build()
                } else {
                    set.push(id.clone());
                    SendMessageBuilder::new(id, "You are now listening".to_string()).build()
                }
            };
            state.save();

            bot.send_message(msg).await.unwrap();
        }
        Some(IpSubcommand::Unlisten) => {
            let msg = {
                let mut set = state.ip_listener.lock().unwrap();

                if let Some(idx) = set.iter().position(|i| *i == id) {
                    set.remove(idx);
                    SendMessageBuilder::new(id, "You are not listening anymore".to_string()).build()
                } else {
                    SendMessageBuilder::new(id, "You were not listening".to_string()).build()
                }
            };
            state.save();

            bot.send_message(msg).await.unwrap();
        }
    }

    Ok(())
}

#[daemon(interval = 60)]
pub async fn ip_daemon(state: &BotState<State>, bot: &Bot) {
    let Some(current_ip) = public_ip::addr_v4().await else {
        error!( "Could not get public IPv4");
        return ;
    };

    let Some(stored_ip) = *state.ip.lock().unwrap() else {
        return ;
    };

    if stored_ip != current_ip {
        notify_listeners(state, current_ip, bot).await;
    }
}
