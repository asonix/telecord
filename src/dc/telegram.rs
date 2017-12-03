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

//! This module defines how the Discord bot creates intermediate Message Representations and sends
//! them to the Telegram bot.

use serenity::model;
use telebot::objects::Integer;
use futures::sync::mpsc::Sender;
use futures::{Future, Sink};
use mime_sniffer::MimeTypeSniffer;
use mime;

use tg;
use config::Config;

/// Regular Messages include any message sent from a discord user. They can contain files or just
/// text.
///
/// This function first checks if there is an associated Telegram Chat ID for the Discord Channel
/// the message came from, and then offloads to `no_attachments` and `has_attachments` depending on
/// whether files are attached.
pub fn regular_message(config: &Config, sender: Sender<tg::Message>, message: model::Message) {
    debug!(
        "content: {}, channel: {}",
        message.content,
        message.channel_id
    );

    let chat_id = config.telegram_chat_id(&message.channel_id);
    let chat_id = if let Some(chat_id) = chat_id {
        chat_id
    } else {
        return;
    };

    if message.author.bot {
        return;
    }

    if message.attachments.is_empty() {
        no_attachments(sender, chat_id, message);
    } else {
        has_attachments(&sender, chat_id, message);
    }
}

/// Join Messages occur when a user joins a Discord channel. This function is currently a stub.
pub fn join_message(message: &model::Message) {
    debug!("{} joined!", message.content);
}

// Builds a text message representation and sends it to the Telegram bot.
fn no_attachments(sender: Sender<tg::Message>, chat_id: Integer, message: model::Message) {
    if let Err(e) = sender
        .send(tg::Message::text(
            message.author.name,
            chat_id,
            message.content,
        ))
        .wait()
    {
        error!("Failed to send text because {}", e);
    }
}

// For each attachment, send a file message representation to the Telegram bot.
fn has_attachments(sender: &Sender<tg::Message>, chat_id: Integer, message: model::Message) {
    let content = if message.content.is_empty() {
        None
    } else {
        Some(message.content.clone())
    };

    for attachment in message.attachments {
        // Download each attachment and send it to the Telegram Bot as a new intermediate message
        if let Ok(bytes) = attachment.download() {
            let mtype_opt: Option<String> = bytes.sniff_mime_type().map(|s| s.into());

            let mtype = if let Some(mtype) = mtype_opt {
                mtype
            } else {
                mime::APPLICATION_OCTET_STREAM.as_ref().into()
            };

            // This unwrap is safe since the mtype is ensured to have come from a Mimetype to begin
            // with. The reason for this Mime -> String -> Mime conversion is the mime-sniffer
            // package uses and outdated mime package and telecord depends on a current mime
            // package.
            let mtype = mtype.parse::<mime::Mime>().unwrap();
            // If the mime type sniffed from the downloaded file exists (it should), send
            // the message
            if let Err(e) = sender
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
                error!("Failed to send because {}", e);
            }
        }
    }
}
