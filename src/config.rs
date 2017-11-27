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

use std::env;
use dotenv::dotenv;
use telebot::objects::Integer;

#[derive(Debug, Clone)]
pub struct Config {
    discord_token: String,
    telegram_token: String,
    telegram_chat_id: Integer,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        Config {
            discord_token: env::var("DISCORD_BOT_TOKEN").expect(
                "Please set the DISCORD_BOT_TOKEN environment variable",
            ),
            telegram_token: env::var("TELEGRAM_BOT_TOKEN").expect(
                "Please set the TELEGRAM_BOT_TOKEN environment variable",
            ),
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID")
                .expect("Please set the TELEGRAM_CHAT_ID environment variable")
                .parse()
                .expect("Invalid telegram chat ID"),
        }
    }

    pub fn discord(&self) -> &str {
        &self.discord_token
    }

    pub fn telegram(&self) -> &str {
        &self.telegram_token
    }

    pub fn telegram_chat_id(&self) -> Integer {
        self.telegram_chat_id
    }
}
