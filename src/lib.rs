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

//! # Telecord: A bot to link Discord channels to Telegram Channels
//!
//! Telecord currently sends text messages between Telegram and Discord, and media messages from
//! Telegram to Discord. Media messages from Discord to Telegram are in the works, but are
//! currently blocked on `issue#11` for the Telebot crate
//!
//! In order to run this crate, a few environment variables must be set
//! - `DISCORD_BOT_TOKEN` must contain the token for your discord bot
//! - `TELEGRAM_BOT_TOKEN` must contain the token for your telegram bot
//! - `CHAT_MAPPINGS` must be a comma-separated list of colon-separated tuples. What this means is
//!     for each discord channel and telegram chat you wish to connect, you must specify the
//!     telegram chat's ID and the discord channel's ID in the following format:
//!     `telegram_chat_id:discord_channel_id`. You can have as many connected chats and channels as
//!     you would like by adding more to the mapping: `tg_id:dc_id,tg_id2:dc_id2,tg_id3,dc_id3`.
//!
//! Once your environment variables are set, you can run the crate with `cargo run`
//!
//! ## This is the library for Telebot.
//!
//! It may not be incredibly useful outside the context of the associated main.rs

#![feature(try_from)]
#![feature(conservative_impl_trait)]

extern crate dotenv;
extern crate telebot;
extern crate serenity;
extern crate futures;
extern crate mime;
extern crate mime_sniffer;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;

#[macro_use]
extern crate log;

mod config;
pub mod tg;
pub mod dc;

pub use config::Config;
