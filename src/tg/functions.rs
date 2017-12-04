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

//! This module defines functions associated with Telegram and Telebot. Specifically, it defines
//! the functionality for handling an incoming intermediate Message construct and translating that
//! into a legitimate telegram message.

use telebot::bot::RcBot;
use telebot::file::File;
use telebot::functions::{FunctionMessage, FunctionSendAudio, FunctionSendDocument,
                         FunctionSendPhoto, FunctionSendVideo};
use telebot::objects::Integer;
use futures::Future;
use std::io::Cursor;
use std::convert::TryInto;

use super::{FileKind, FileMessage, Message, MessageContent};

/// Given an intermediate Message type, send a legitimate message to Telegram.
pub fn handle_forward(bot: &RcBot, message: Message) {
    let user = message.from;
    let chat_id = message.chat_id;

    match message.content {
        MessageContent::Text(content) => {
            send_text(bot, &user, chat_id, &content);
        }
        MessageContent::File(file) => {
            send_file(bot, &user, chat_id, file);
        }
    }
}

// Sends text to Telegram
fn send_text(bot: &RcBot, user: &str, chat_id: Integer, content: &str) {
    bot.inner.handle.spawn(
        bot.message(chat_id, {
            // Escape content that could be mistaken for HTML tags.
            let escaped_content = content.replace("&", "&amp;").replace(">", "&gt;").replace(
                "<",
                "&lt;",
            );
            let output = format!("<b>{}</b>: {}", user, escaped_content);
            debug!("{}", output);
            output
        }).parse_mode("HTML")
            .send()
            .map(|_| ())
            .map_err(|err| {
                error!("Error: {:?}", err);
            }),
    );
}

// Determines what kind of file is being sent, and dispatches to one of the other send functions
// such as send_image, send_audio, send_video, and send_document
fn send_file(bot: &RcBot, user: &str, chat_id: Integer, file_msg: FileMessage) {
    let FileMessage {
        caption,
        filename,
        contents,
        kind,
    } = file_msg;

    debug!("filename: {}", filename);

    let caption = if let Some(caption) = caption {
        format!("{}: {}", user, caption)
    } else {
        format!("{} {}", user, "sent a file")
    };

    debug!("File len: {}", contents.len());

    let file = (filename.as_ref(), Cursor::new(contents));

    match kind {
        FileKind::Image => {
            send_image(bot, chat_id, file, &caption);
        }
        FileKind::Audio => {
            send_audio(bot, chat_id, file, &caption);
        }
        FileKind::Video => {
            send_video(bot, chat_id, file, &caption);
        }
        FileKind::Unknown => {
            send_document(bot, chat_id, file, &caption);
        }
    }
}

// Sends an Image to Telegram
fn send_image<T>(bot: &RcBot, chat_id: Integer, file: T, caption: &str)
where
    T: TryInto<File>,
{
    bot.inner.handle.spawn({
        bot.photo(chat_id)
            .file(file)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                error!("Error sending file: {:?}", err);
            })
    });
}

// Sends Audio to Telegram
fn send_audio<T>(bot: &RcBot, chat_id: Integer, file: T, caption: &str)
where
    T: TryInto<File>,
{
    bot.inner.handle.spawn({
        bot.audio(chat_id)
            .file(file)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                error!("Error sending file: {:?}", err);
            })
    });
}

// Sends a Video to Telegram
fn send_video<T>(bot: &RcBot, chat_id: Integer, file: T, caption: &str)
where
    T: TryInto<File>,
{
    bot.inner.handle.spawn({
        bot.video(chat_id)
            .file(file)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                error!("Error sending file: {:?}", err);
            })
    });
}

// Sends a Document to Telegram
fn send_document<T>(bot: &RcBot, chat_id: Integer, file: T, caption: &str)
where
    T: TryInto<File>,
{
    bot.inner.handle.spawn({
        bot.document(chat_id)
            .file(file)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                error!("Error sending file: {:?}", err);
            })
    });
}
