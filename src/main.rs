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

/// # Telecord: A bot to link Discord channels to Telegram Channels
///
/// Telecord currently sends text messages between Telegram and Discord, and media messages from
/// Telegram to Discord. Media messages from Discord to Telegram are in the works, but are
/// currently blocked on `issue#11` for the Telebot crate
///
/// In order to run this crate, a few environment variables must be set
/// - `DISCORD_BOT_TOKEN` must contain the token for your discord bot
/// - `TELEGRAM_BOT_TOKEN` must contain the token for your telegram bot
/// - `CHAT_MAPPINGS` must be a comma-separated list of colon-separated tuples. What this means is
///     for each discord channel and telegram chat you wish to connect, you must specify the
///     telegram chat's ID and the discord channel's ID in the following format:
///     `telegram_chat_id:discord_channel_id`. You can have as many connected chats and channels as
///     you would like by adding more to the mapping: `tg_id:dc_id,tg_id2:dc_id2,tg_id3,dc_id3`.
///
/// Once your environment variables are set, you can run the crate with `cargo run`
///
/// ### Main.rs
///
/// Telecord relies on the Serenity library to interface with Discord, and the Telebot library to
/// interface with Telegram. These libraries have fundementally different architectures, so mapping
/// from one to the other requires a few processes.
///
/// Serentiy relies on a threadpool to handle blocking work across multiple cores, while Telebot
/// relies on Tokio's event-loop to handle nonblocking work on a single thread. Main.rs reflects
/// this in its flow.
///
/// First, we generate our Config struct, and create senders and receivers for the messages passed
/// between Serenity and Telebot, then we create a thread to handle messages forwarded to discord.
/// Next, we create a thread to enclose the Tokio event-loop, which handles both receiving messages
/// from Telegram and sending them to the Discord thread, and receiving messages from Serenity and
/// sending them to Telegram. Finally, we start Serenity's threadpool from the current thread.

extern crate serenity;
extern crate telebot;
extern crate tokio_core;
extern crate futures;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate telecord;

use serenity::prelude::*;
use tokio_core::reactor::Core;
use futures::sync::mpsc::channel;
use futures::{Future, IntoFuture, Stream};
use telebot::bot;
use telebot::functions::*;
use std::thread;
use std::sync;

use telecord::{Config, tg, dc};

fn main() {
    env_logger::init().unwrap();
    info!("Starting up!");
    let config = Config::new();

    let (tg_sender, tg_receiver) = channel::<tg::Message>(100);
    let (dc_sender, dc_receiver) = sync::mpsc::channel::<dc::Message>();

    let mut discord_bot = Client::new(
        config.discord(),
        dc::Handler::new(config.clone(), tg_sender),
    );

    let closure_config = config.clone();

    let dc_thread = thread::spawn(move || {
        // Sends forwared messages to Discord
        dc::forward_iter(dc_receiver)
    });

    let tg_thread = thread::spawn(move || {
        let mut lp = Core::new().unwrap();
        let telegram_bot = bot::RcBot::new(lp.handle(), config.telegram()).update_interval(200);
        let closure_bot = telegram_bot.clone();
        let closure_bot2 = telegram_bot.clone();

        // Sends forwarded messages to Telegram
        telegram_bot.inner.handle.spawn(tg_receiver.for_each(
            move |tg_message| {
                tg::handle_forward(&closure_bot.clone(), tg_message);

                Ok(())
            },
        ));

        // useful for testing if the bot is running
        telegram_bot.register(telegram_bot.new_cmd("/ping").and_then(|(bot, msg)| {
            bot.message(msg.chat.id, "pong".into()).send()
        }));

        // useful for getting chat_ids from group chats
        telegram_bot.register(telegram_bot.new_cmd("/chat_id").and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format!("{}", msg.chat.id)).send()
        }));

        // forwards Telegram messages to Discord
        let stream = telegram_bot.get_stream().filter_map(|(bot, update)| {
            if let Some(msg) = update.message {
                tg::discord::handle_message(
                    &closure_bot2.clone(),
                    &closure_config.clone(),
                    msg,
                    dc_sender.clone(),
                );
                None
            } else {
                Some((bot, update))
            }
        });

        // Starts handling messages from Telegram
        let res: Result<(), ()> = lp.run(
            stream
                .for_each(|_| Ok(()))
                .or_else(|e| {
                    error!("Error: {:?}", e);
                    Ok(())
                })
                .into_future(),
        );

        res.unwrap();
    });

    // Starts handling messages from Discord
    discord_bot.start().unwrap();

    tg_thread.join().unwrap();
    dc_thread.join().unwrap();
}
