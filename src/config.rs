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

//! The config module exists to expose the Config type

use std::collections::HashMap;
use std::env;
use dotenv::dotenv;
use telebot::objects::Integer;
use serenity::model::ChannelId;

/// The Config type contains four values, the Discord Bot's Token, the Telegram Bot's token, a
/// HashMap for quickly getting a Telegram Chat ID from a Discord Channel ID, and a HashMap for
/// quickly getting a Discord Channel ID from a Telegram Chat ID.
#[derive(Debug, Clone)]
pub struct Config {
    discord_token: String,
    telegram_token: String,
    discord_to_telegram: HashMap<ChannelId, Integer>,
    telegram_to_discord: HashMap<Integer, ChannelId>,
}

impl Config {
    /// The new function is what actually accesses the environment to read in the required values.
    ///
    /// The `CHAT_MAPPINGS` environment variable is read as a comma-separated list of
    /// colon-separated tuples, where the first element of each tuple is a Telegram Chat ID and the
    /// second argument of each tuple is a Discord Channel ID.
    ///
    /// For example, `CHAT_MAPPINGS=1234:abcd,5678:efgh` would map the Telegram Chat 1234 to the
    /// Discord Channel abcd and would also map the Telegram Chat 5678 to the Discord Channel efgh.
    ///
    /// The `DISCORD_BOT_TOKEN` and `TELEGRAM_BOT_TOKEN` environment variables are
    /// self-explanatory.
    ///
    /// If any of the required environment variables are not set, this function will panic. Since
    /// this is intended to be the first function an application runs, this should not cause
    /// issues. Either the config will be correct, or the application will not attempt to deal with
    /// missing configuration options.
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

    /// Returns the Discord Bot Token
    pub fn discord(&self) -> &str {
        &self.discord_token
    }

    /// Returns the Telegram Bot Token
    pub fn telegram(&self) -> &str {
        &self.telegram_token
    }

    /// Retrieves the Telegram Chat ID that corresponds to the given Discord Channel ID.
    pub fn telegram_chat_id(&self, discord_channel_id: &ChannelId) -> Option<Integer> {
        self.discord_to_telegram.get(discord_channel_id).map(|i| {
            i.clone()
        })
    }

    /// Retrieves the Discord Channel ID that corresponds to the given Telegram Chat ID.
    pub fn discord_channel_id(&self, telegram_chat_id: &Integer) -> Option<ChannelId> {
        self.telegram_to_discord.get(telegram_chat_id).map(
            |i| i.clone(),
        )
    }
}
