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

//! This module contains functions related to forwarding messages to Discord from Telegram.
//!
//! It doesn't handle the actual sending of messages, but instead offloads an intermediate message
//! construct through an mpsc::channel to (possibly) another thread.

use std::sync::mpsc::Sender;
use telebot::objects;
use telebot::RcBot;
use telebot::functions::FunctionGetFile;
use serenity::model::ChannelId;
use futures::Future;

use dc;
use super::download::download_file;
use config::Config;

/// As the only public funtion in the module, handle_message dictates the flow for sending messages
/// to Discord. First, get the User's name, preferring username, but falling back on firstname
/// lastname. Then we get the associated Discord Channel ID from the given Telegram Chat ID. If
/// there is some kind of file attached, we get telegram's File ID from it, and procede to send the
/// file, and if there is not a file attached, we send the provided text.
///
/// Sending a file happens in three stages. First the file_id is used to fetch a URL to download
/// the file, second, the file is downloaded from Telegram, and third, the file is re-uploaded to
/// Discord.
pub fn handle_message(
    bot: RcBot,
    config: Config,
    message: objects::Message,
    sender: Sender<dc::Message>,
) {
    let user = if let Some(ref user) = message.from {
        get_user_name(user)
    } else {
        return;
    };

    debug!("user: {}", user);
    debug!("chat id: {}", &message.chat.id);

    let channel_id = config.discord_channel_id(&message.chat.id);
    let channel_id = if let Some(channel_id) = channel_id {
        channel_id
    } else {
        return;
    };

    debug!("channel_id: {}", channel_id);

    let caption = message.caption.clone();
    let text = message.text.clone();
    let sticker = message.sticker.is_some();

    let file_id = get_file_id(message);

    if let Some(file_id) = file_id {
        send_file(bot, sender, channel_id, user, file_id, caption, sticker);
    } else if let Some(text) = text {
        send_text(sender, channel_id, user, text);
    } else {
        debug!("Not sending message");
    }
}

// Prefer a username, but fallback to firstname lastname if not available
fn get_user_name(user: &objects::User) -> String {
    if let Some(ref username) = user.username {
        format!("{}", username)
    } else if let Some(ref last_name) = user.last_name {
        format!("{} {}", user.first_name, last_name)
    } else {
        user.first_name.clone()
    }
}

// Get the file_id from any Telegram file type
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

// Send File if Filesize is below 8 * 10^6 Bytes.
fn send_file(
    bot: RcBot,
    sender: Sender<dc::Message>,
    channel_id: ChannelId,
    user: String,
    file_id: String,
    caption: Option<String>,
    sticker: bool,
) {
    bot.inner.handle.spawn(
        bot.get_file(file_id)
            .send()
            .map_err(|e| debug!("Failed: {:?}", e))
            .and_then(|(bot, file)| {
                download_file(bot, file).map_err(|e| debug!("Error: {}", e))
            })
            .and_then(move |(response, filename)| {
                let filename = if sticker {
                    format!("{}.webp", filename)
                } else {
                    filename
                };

                let res = sender.send(dc::Message::file(
                    user,
                    channel_id,
                    caption,
                    filename,
                    response,
                ));

                match res {
                    Ok(_) => (),
                    Err(e) => debug!("Failed to send file: {}", e),
                }
                Ok(())
            }),
    );
}

// Send text to Telegram
fn send_text(sender: Sender<dc::Message>, channel_id: ChannelId, user: String, text: String) {
    let res = sender.send(dc::Message::text(user, channel_id, text));

    match res {
        Ok(_) => (),
        Err(e) => debug!("Failed to send text: {}", e),
    }
}
