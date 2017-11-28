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

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use serenity::http::AttachmentType;
use serenity::model::ChannelId;
use telebot::objects;
use telebot::RcBot;
use telebot::functions::FunctionGetFile;
use futures::Future;
use Config;
use tokio_curl::Session;
use curl::easy::Easy;

use super::{Message, MessageContent, FileMessage};

pub fn handle_tg_message(
    bot: RcBot,
    config: Config,
    message: objects::Message,
    sender: Sender<Message>,
) {
    println!("handle_tg_message");
    let user = if let Some(ref user) = message.from {
        get_user_name(user)
    } else {
        return;
    };

    println!("user: {}", user);
    println!("chat id: {}", &message.chat.id);

    let channel_id = config.discord_channel_id(&message.chat.id);
    let channel_id = if let Some(channel_id) = channel_id {
        channel_id
    } else {
        return;
    };

    println!("channel_id: {}", channel_id);

    let caption = message.caption.clone();
    let text = message.text.clone();
    let sticker = message.sticker.is_some();

    let file_id = get_file_id(message);

    if let Some(file_id) = file_id {
        send_tg_file(bot, sender, channel_id, user, file_id, caption, sticker);
    } else if let Some(text) = text {
        send_tg_text(sender, channel_id, user, text);
    } else {
        println!("Not sending message");
    }
}

fn get_user_name(user: &objects::User) -> String {
    if let Some(ref username) = user.username {
        format!("{}", username)
    } else if let Some(ref last_name) = user.last_name {
        format!("{} {}", user.first_name, last_name)
    } else {
        user.first_name.clone()
    }
}

fn get_file_id(message: objects::Message) -> Option<String> {
    if let Some(audio) = message.audio {
        Some(audio.file_id)
    } else if let Some(document) = message.document {
        Some(document.file_id)
    } else if let Some(photos) = message.photo {
        photos.iter().max_by_key(|photo| photo.width).map(|photo| {
            photo.file_id.clone()
        })
    } else if let Some(sticker) = message.sticker {
        Some(sticker.file_id)
    } else if let Some(voice) = message.voice {
        Some(voice.file_id)
    } else {
        None
    }
}

fn send_tg_file(
    bot: RcBot,
    sender: Sender<Message>,
    channel_id: ChannelId,
    user: String,
    file_id: String,
    caption: Option<String>,
    sticker: bool,
) {
    println!("send_tg_file");
    bot.inner.handle.spawn(
        bot.get_file(file_id)
            .send()
            .map_err(|e| println!("Failed: {:?}", e))
            .and_then(move |(bot, file)| if let Some(file_size) = file.file_size {
                println!("file_size: {}", file_size);

                if file_size < 8 * 1000 * 1000 {
                    Ok((bot, file))
                } else {
                    Err(())
                }
            } else {
                Err(())
            })
            .and_then(move |(bot, file)| if let Some(path) = file.file_path {
                let path = format!(
                    "https://api.telegram.org/file/bot{}/{}",
                    bot.inner.key,
                    path
                );
                let url = Path::new(&path);
                let filename = url.file_name();

                if let Some(filename) = filename {
                    if let Some(filename) = filename.to_str() {
                        Ok((bot, path.clone(), String::from(filename)))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            } else {
                Err(())
            })
            .and_then(move |(bot, path, filename)| {
                download_file(bot.clone(), &path).and_then(move |response| {
                    let filename = if sticker {
                        format!("{}.webp", filename)
                    } else {
                        filename
                    };

                    let res =
                        sender.send(Message::file(user, channel_id, caption, filename, response));

                    match res {
                        Ok(_) => (),
                        Err(e) => println!("Failed to send file: {}", e),
                    }
                    Ok(())
                })
            }),
    );
}

fn download_file(bot: RcBot, url: &str) -> impl Future<Item = Vec<u8>, Error = ()> {
    let session = Session::new(bot.inner.handle.clone());
    let mut req = Easy::new();
    req.get(true).unwrap();
    println!("url: {}", url);
    req.url(url).unwrap();
    let result = Arc::new(Mutex::new(Vec::new()));

    let r2 = result.clone();
    req.write_function(move |data| {
        r2.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }).unwrap();

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

fn send_tg_text(sender: Sender<Message>, channel_id: ChannelId, user: String, text: String) {
    println!("send_tg_text");
    let res = sender.send(Message::text(user, channel_id, text));

    match res {
        Ok(_) => (),
        Err(e) => println!("Failed to send text: {}", e),
    }
}

pub fn message_iter(rx: Receiver<Message>) {
    for message in rx {
        let user = message.from;
        let channel_id = message.channel_id;

        match message.content {
            MessageContent::Text(content) => send_text(channel_id, user, content),
            MessageContent::File(file) => send_file(channel_id, user, file),
        }
    }
}

fn send_text(channel_id: ChannelId, user: String, content: String) {
    let msg = format!("**{}**: {}", user, content);
    match channel_id.say(msg) {
        Ok(_) => (),
        Err(e) => println!("Failed to send message: {}", e),
    }
}

fn send_file(channel_id: ChannelId, user: String, file_msg: FileMessage) {
    let FileMessage {
        caption,
        filename,
        contents,
    } = file_msg;

    let tup: (&[u8], &str) = (contents.as_ref(), &filename);
    let attachment_type: AttachmentType = AttachmentType::from(tup);
    let attchmnts = vec![attachment_type];

    let res = channel_id.send_files(attchmnts, |create_message| {
        let msg = if let Some(caption) = caption {
            format!("**{}**: {}", user, caption)
        } else {
            format!("**{}**: {}", user, "*sent a file*")
        };

        create_message.content(msg)
    });

    match res {
        Ok(_) => (),
        Err(e) => println!("Failed to send files: {}", e),
    }
}
