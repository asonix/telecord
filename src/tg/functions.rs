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

use telebot::bot::RcBot;
use telebot::file::File;
use telebot::functions::{FunctionMessage, FunctionSendAudio, FunctionSendDocument,
                         FunctionSendPhoto, FunctionSendVideo};
use telebot::objects::Integer;
use futures::Future;
use std::io::Cursor;

use super::{FileKind, FileMessage, Message, MessageContent};

fn send_image(bot: RcBot, chat_id: Integer, image: File, caption: String) {
    bot.inner.handle.spawn({
        bot.photo(chat_id)
            .file(image)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                println!("Error sending file: {:?}", err);
            })
    });
}

fn send_audio(bot: RcBot, chat_id: Integer, audio: File, caption: String) {
    bot.inner.handle.spawn({
        bot.audio(chat_id)
            .file(audio)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                println!("Error sending file: {:?}", err);
            })
    });
}

fn send_video(bot: RcBot, chat_id: Integer, video: File, caption: String) {
    bot.inner.handle.spawn({
        bot.video(chat_id)
            .file(video)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                println!("Error sending file: {:?}", err);
            })
    });
}

fn send_document(bot: RcBot, chat_id: Integer, document: File, caption: String) {
    bot.inner.handle.spawn({
        bot.document(chat_id)
            .file(document)
            .caption(caption)
            .send()
            .map(|_| ())
            .map_err(|err| {
                println!("Error sending file: {:?}", err);
            })
    });
}

fn send_text(bot: RcBot, user: String, chat_id: Integer, content: String) {
    bot.inner.handle.spawn(
        bot.message(chat_id, {
            let escaped_content = content.replace("&", "&amp;").replace(">", "&gt;").replace(
                "<",
                "&lt;",
            );
            let output = format!("<b>{}</b>: {}", user, escaped_content);
            println!("{}", output);
            output
        }).parse_mode("HTML")
            .send()
            .map(|_| ())
            .map_err(|err| {
                println!("Error: {:?}", err);
            }),
    );
}

fn send_file(bot: RcBot, user: String, chat_id: Integer, file_msg: FileMessage) {
    let FileMessage {
        caption,
        filename,
        contents,
        kind,
    } = file_msg;

    println!("filename: {}", filename);
    let tup: (&str, Cursor<Vec<u8>>) = (&filename, Cursor::new(contents));
    let file: File = File::from(tup);

    let caption = if let Some(caption) = caption {
        format!("*{}*: {}", user, caption)
    } else {
        format!("*{}*: {}", user, "__sent a file__")
    };

    match kind {
        FileKind::Image => {
            send_image(bot, chat_id, file, caption);
        }
        FileKind::Audio => {
            send_audio(bot, chat_id, file, caption);
        }
        FileKind::Video => {
            send_video(bot, chat_id, file, caption);
        }
        FileKind::Unknown => {
            send_document(bot, chat_id, file, caption);
        }
    }
}

pub fn handle_message(bot: RcBot, message: Message) {
    let user = message.from;
    let chat_id = message.chat_id;

    match message.content {
        MessageContent::Text(content) => {
            send_text(bot, user, chat_id, content);
        }
        MessageContent::File(file) => {
            send_file(bot, user, chat_id, file);
        }
    }
}
