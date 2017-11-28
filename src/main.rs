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

extern crate serenity;
extern crate telebot;
extern crate tokio_core;
extern crate futures;

extern crate telecord;

use serenity::prelude::*;
use tokio_core::reactor::Core;
use futures::sync::mpsc::channel;
use futures::{IntoFuture, Stream};
use telebot::bot;
use telebot::functions::*;
use std::thread;
use std::sync;

use telecord::{Config, tg, dc};

fn main() {
    let config = Config::new();

    let (tg_sender, tg_receiver) = channel::<tg::Message>(100);

    let mut discord_bot = Client::new(
        config.discord(),
        dc::Handler::new(config.clone(), tg_sender),
    );

    let (dc_sender, dc_receiver) = sync::mpsc::channel::<dc::Message>();

    let closure_config = config.clone();

    let dc_thread = thread::spawn(move || dc::message_iter(dc_receiver));

    let tg_thread = thread::spawn(move || {
        let mut lp = Core::new().unwrap();
        let telegram_bot = bot::RcBot::new(lp.handle(), config.telegram()).update_interval(200);
        let closure_bot = telegram_bot.clone();
        let closure_bot2 = telegram_bot.clone();

        telegram_bot.inner.handle.spawn(tg_receiver.for_each(
            move |tg_message| {
                tg::handle_message(closure_bot.clone(), tg_message);

                Ok(())
            },
        ));

        telegram_bot.register(telegram_bot.new_cmd("/ping").and_then(|(bot, msg)| {
            bot.message(msg.chat.id, "pong".into()).send()
        }));

        telegram_bot.register(telegram_bot.new_cmd("/chat_id").and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format!("{}", msg.chat.id)).send()
        }));

        let stream = telegram_bot.get_stream().filter_map(|(bot, update)| {
            if let Some(msg) = update.message {
                dc::handle_tg_message(
                    closure_bot2.clone(),
                    closure_config.clone(),
                    msg,
                    dc_sender.clone(),
                );
                None
            } else {
                Some((bot, update))
            }
        });

        lp.run(stream.for_each(|_| Ok(())).into_future()).unwrap();
    });

    let _ = discord_bot.start();

    let _ = tg_thread.join();
    let _ = dc_thread.join();
}
