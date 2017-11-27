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

// #[macro_use]
// extern crate lazy_static;

// use tokio_core::reactor::{Core, Handle};
use serenity::prelude::*;
use tokio_core::reactor::Core;
use futures::sync::mpsc::channel;
use futures::Stream;
use telebot::bot;
use telebot::functions::*;
use std::thread;

use telecord::{Config, tg, dc};

fn main() {
    let config = Config::new();

    let (tg_sender, tg_receiver) = channel::<tg::Message>(100);

    let mut discord_bot = Client::new(config.discord(), dc::Handler::new(tg_sender));

    let closure_config = config.clone();

    let tg_thread = thread::spawn(move || {
        let mut lp = Core::new().unwrap();
        let telegram_bot = bot::RcBot::new(lp.handle(), config.telegram()).update_interval(200);
        let closure_bot = telegram_bot.clone();

        telegram_bot.inner.handle.spawn(tg_receiver.for_each(
            move |tg_message| {
                let user = tg_message.from;

                match tg_message.content {
                    tg::MessageContent::Text(content) => {
                        tg::send_text(closure_bot.clone(), &closure_config, user, content);
                    }
                    tg::MessageContent::File(file) => {
                        tg::send_file(closure_bot.clone(), &closure_config, user, file);
                    }
                }

                Ok(())
            },
        ));

        let cmd = telegram_bot.new_cmd("/ping").and_then(|(bot, msg)| {
            bot.message(msg.chat.id, "pong".into()).send()
        });

        telegram_bot.register(cmd);

        telegram_bot.run(&mut lp)
    });

    let _ = discord_bot.start();

    let _ = tg_thread.join();
}
