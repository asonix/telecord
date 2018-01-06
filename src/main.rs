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

// Main.rs
//
// Telecord relies on the Serenity library to interface with Discord, and the Telebot library to
// interface with Telegram. These libraries have fundementally different architectures, so mapping
// from one to the other requires a few processes.
//
// Serentiy relies on a threadpool to handle blocking work across multiple cores, while Telebot
// relies on Tokio's event-loop to handle nonblocking work on a single thread. Main.rs reflects
// this in its flow.
//
// First, we generate our Config struct, and create senders and receivers for the messages passed
// between Serenity and Telebot, then we create a thread to handle messages forwarded to discord.
// Next, we create a thread to enclose the Tokio event-loop, which handles both receiving messages
// from Telegram and sending them to the Discord thread, and receiving messages from Serenity and
// sending them to Telegram. Finally, we start Serenity's threadpool from the current thread.

extern crate openssl_probe;
extern crate telebot;
extern crate tokio_core;
extern crate futures;

#[macro_use]
extern crate serenity;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate telecord;

use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;
use tokio_core::reactor::Core;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Stream};
use telebot::bot;
use telebot::functions::*;
use std::thread;
use std::sync;

use telecord::{Config, tg, dc};

fn init_bot(bot: &bot::RcBot) {
    bot.inner.handle.spawn(
        bot.get_me()
            .send()
            .map_err(|e| println!("Error: {:?}", e))
            .and_then(|(bot, user)| {
                let pairs = bot.inner
                    .handlers
                    .borrow()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<Vec<_>>();

                let username = if let Some(username) = user.username {
                    username
                } else {
                    return Err(());
                };

                for (key, value) in pairs {
                    bot.inner.handlers.borrow_mut().insert(
                        format!(
                            "{}@{}",
                            key,
                            username
                        ),
                        value,
                    );
                }

                Ok(())
            }),
    );
}

fn tg_supervisor(
    config: Config,
    dc_sender: sync::mpsc::Sender<dc::Message>,
    tg_receiver: Receiver<tg::Message>,
) {
    let closure_config = config.clone();

    let mut lp = Core::new().unwrap();
    let telegram_bot = bot::RcBot::new(lp.handle(), config.telegram()).update_interval(200);
    init_bot(&telegram_bot);
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

    loop {
        // forwards Telegram messages to Discord
        let stream = telegram_bot
            .get_stream()
            .filter_map(|(bot, update)| if let Some(msg) = update.message {
                tg::discord::handle_message(
                    &closure_bot2.clone(),
                    &closure_config.clone(),
                    msg,
                    dc_sender.clone(),
                );
                None
            } else {
                Some((bot, update))
            })
            .map_err(|e| error!("Error: {}", e))
            .for_each(|_| Ok(()));

        // Starts handling messages from Telegram
        let res = lp.run(stream);

        if let Err(e) = res {
            error!("Error in event loop: {:?}", e);
        }
    }
}

fn dc_bot_supervisor(config: Config, tg_sender: Sender<tg::Message>) {
    let mut discord_bot = Client::new(
        config.discord(),
        dc::Handler::new(config.clone(), tg_sender),
    );

    discord_bot.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("/"))
            .on("channel_id", channel_id)
            .on("ping", dc_ping),
    );

    // Starts handling messages from Discord
    discord_bot.start().unwrap();
    info!("discord_bot ended!");
}

fn dc_sender_supervisor(dc_receiver: sync::mpsc::Receiver<dc::Message>) {
    // Sends forwared messages to Discord
    dc::forward_iter(dc_receiver);
    info!("forward_iter ended!");
}

fn main() {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init().unwrap();
    info!("Starting up!");
    let config = Config::new();

    let (tg_sender, tg_receiver) = channel::<tg::Message>(100);
    let (dc_sender, dc_receiver) = sync::mpsc::channel::<dc::Message>();

    let tg_config = config.clone();
    let tg_supervisor = thread::spawn(move || tg_supervisor(tg_config, dc_sender, tg_receiver));
    let dc_sender_supervisor = thread::spawn(move || dc_sender_supervisor(dc_receiver));
    let dc_bot_supervisor = thread::spawn(move || dc_bot_supervisor(config, tg_sender));

    tg_supervisor.join().unwrap();
    dc_sender_supervisor.join().unwrap();
    dc_bot_supervisor.join().unwrap();
}

command!(dc_ping(_context, message) {
    let _ = message.reply("pong");
});

command!(channel_id(_context, message) {
    let _ = message.reply(&format!("{}", message.channel_id.0));
});
