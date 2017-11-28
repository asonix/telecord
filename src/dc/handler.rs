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

use serenity::model;
use serenity::prelude::*;
use telebot::objects::Integer;
use futures::sync::mpsc::Sender;
use futures::{Future, Sink};
use mime_sniffer::MimeTypeSniffer;
use mime;

use tg;
use config::Config;

pub struct Handler {
    config: Config,
    tg_sender: Sender<tg::Message>,
}

impl Handler {
    pub fn new(config: Config, tg_sender: Sender<tg::Message>) -> Self {
        Handler { config, tg_sender }
    }

    fn no_attachments(&self, chat_id: Integer, message: model::Message) {
        if let Err(e) = self.tg_sender
            .clone()
            .send(tg::Message::text(
                message.author.name.clone(),
                chat_id,
                message.content.clone(),
            ))
            .wait()
        {
            println!("Failed to send text because {}", e);
        }
    }

    fn has_attachments(&self, chat_id: Integer, message: model::Message) {
        let content = if message.content.is_empty() {
            None
        } else {
            Some(message.content.clone())
        };

        for attachment in message.attachments {
            if let Ok(bytes) = attachment.download() {
                let mtype_opt: Option<String> = bytes.sniff_mime_type().map(|s| String::from(s));

                if let Some(mtype_str) = mtype_opt {
                    if let Ok(mtype) = mtype_str.parse::<mime::Mime>() {
                        if let Err(e) = self.tg_sender
                            .clone()
                            .send(tg::Message::file(
                                message.author.name.clone(),
                                chat_id,
                                content.clone(),
                                attachment.filename,
                                bytes,
                                mtype.into(),
                            ))
                            .wait()
                        {
                            println!("Failed to send because {}", e);
                        }
                    }
                }
            }
        }
    }

    fn regular_message(&self, _: Context, message: model::Message) {
        println!(
            "content: {},\nchannel: {}",
            message.content,
            message.channel_id
        );

        let chat_id = if let Some(chat_id) = self.config.telegram_chat_id(&message.channel_id) {
            chat_id
        } else {
            return;
        };

        if message.author.bot {
            return;
        }

        if message.attachments.is_empty() {
            self.no_attachments(chat_id, message);
        } else {
            self.has_attachments(chat_id, message);
        }
    }

    fn join_message(&self, _: Context, message: model::Message) {
        println!("{} joined!", message.content);
    }
}

impl EventHandler for Handler {
    fn on_message(&self, ctx: Context, message: model::Message) {
        match message.kind {
            model::MessageType::Regular => {
                self.regular_message(ctx, message);
            }
            model::MessageType::MemberJoin => {
                self.join_message(ctx, message);
            }
            _ => {}
        }
    }
}
