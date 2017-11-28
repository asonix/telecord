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
//! into a legitimate telegram message. It also includes logic for downloading files from Telegram,
//! since the Telebot library does not.

use telebot::bot::RcBot;
use telebot::file::File;
use telebot::functions::{FunctionMessage, FunctionSendAudio, FunctionSendDocument,
                         FunctionSendPhoto, FunctionSendVideo};
use telebot::objects::Integer;
use futures::Future;
use tokio_curl::Session;
use curl::easy::Easy;
use std::sync::{Arc, Mutex};
use std::io::Cursor;

use super::{FileKind, FileMessage, Message, MessageContent};

/// Download a file given a URL. This is designed to work on the Tokio threadpool used by Telebot.
/// This function will return a failed future if the response code is not in the range 200 to 299
/// inclusive.
pub fn download_file(bot: RcBot, url: &str) -> impl Future<Item = Vec<u8>, Error = ()> {
    // Create a tokio-curl session on the bot's event loop
    let session = Session::new(bot.inner.handle.clone());

    // Create a new request
    let mut req = Easy::new();
    req.get(true).unwrap();
    println!("url: {}", url);
    req.url(url).unwrap();

    // Define the callback function for the curl request. Here we use an Arc<Mutex<Vec<u8>>> in
    // order to share this vector between the curl callback and the response callback. This is the
    // same logic used by the Telebot crate to fetch data from Telegram (11/28/17)..
    let result = Arc::new(Mutex::new(Vec::new()));
    let r2 = result.clone();
    req.write_function(move |data| {
        r2.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }).unwrap();

    // Create a future perform the request and then take the error path if the response code is not
    // between 200 and 299 inclusive
    session
        .perform(req)
        .map_err(|e| println!("Error getting file: {}", e))
        .and_then(move |mut res| if let Ok(code) = res.response_code() {
            if 200 <= code && code < 300 {
                Ok(result.lock().unwrap().to_vec())
            } else {
                Err(())
            }
        } else {
            Err(())
        })
}

/// Given an intermediate Message type, send a legitimate message to Telegram.
pub fn handle_forward(bot: RcBot, message: Message) {
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

// Sends text to Telegram
fn send_text(bot: RcBot, user: String, chat_id: Integer, content: String) {
    bot.inner.handle.spawn(
        bot.message(chat_id, {
            // Escape content that could be mistaken for HTML tags.
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

// Determines what kind of file is being sent, and dispatches to one of the other send functions
// such as send_image, send_audio, send_video, and send_document
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

// Sends an Image to Telegram
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

// Sends Audio to Telegram
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

// Sends a Video to Telegram
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

// Sends a Document to Telegram
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
