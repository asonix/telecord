// This file is part of Telecord

// Telecord is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Telecord is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Telecord  If not, see <http://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::env;
use dotenv::dotenv;
use telebot::objects::Integer;
use serenity::model::ChannelId;

#[derive(Debug, Clone)]
pub struct Config {
    discord_token: String,
    telegram_token: String,
    discord_to_telegram: HashMap<ChannelId, Integer>,
    telegram_to_discord: HashMap<Integer, ChannelId>,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        let mapping_vec = env::var("CHAT_MAPPINGS")
            .expect("Please set the CHAT_MAPPINGS environment variable")
            .split(",")
            .filter_map(|mapping| {
                let mapping = mapping.split(":").collect::<Vec<_>>();

                if mapping.len() == 2 {
                    let telegram = mapping[0].parse::<Integer>().expect(
                        "Failed to parse Telegram Chat ID",
                    );
                    let discord = ChannelId(mapping[1].parse().expect(
                        "Failed to parse Discord Channel ID",
                    ));
                    Some((telegram, discord))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut discord_to_telegram = HashMap::new();
        let mut telegram_to_discord = HashMap::new();

        for (tele, disc) in mapping_vec {
            discord_to_telegram.insert(disc, tele);
            telegram_to_discord.insert(tele, disc);
        }

        let discord_token = env::var("DISCORD_BOT_TOKEN").expect(
            "Please set the DISCORD_BOT_TOKEN environment variable",
        );

        let telegram_token = env::var("TELEGRAM_BOT_TOKEN").expect(
            "Please set the TELEGRAM_BOT_TOKEN environment variable",
        );

        Config {
            discord_token,
            telegram_token,
            discord_to_telegram,
            telegram_to_discord,
        }
    }

    pub fn discord(&self) -> &str {
        &self.discord_token
    }

    pub fn telegram(&self) -> &str {
        &self.telegram_token
    }

    pub fn telegram_chat_id(&self, discord_channel_id: &ChannelId) -> Option<Integer> {
        self.discord_to_telegram.get(discord_channel_id).map(|i| {
            i.clone()
        })
    }

    pub fn discord_channel_id(&self, telegram_chat_id: &Integer) -> Option<ChannelId> {
        self.telegram_to_discord.get(telegram_chat_id).map(
            |i| i.clone(),
        )
    }
}
