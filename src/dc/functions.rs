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

//! This module defines functions used to translate an intermediate discord Message type to a
//! legitimate Discord Message.

use std::sync::mpsc::Receiver;
use serenity::http::AttachmentType;
use serenity::model::ChannelId;

use super::{Message, MessageContent, FileMessage};

/// Iterates over the given receiver and translates each Message into legitimate Discord Messages.
/// This function should be called in its own thread, since it will block until all Messages are
/// processed.
pub fn forward_iter(rx: Receiver<Message>) {
    for message in rx {
        let user = message.from;
        let channel_id = message.channel_id;

        match message.content {
            MessageContent::Text(content) => send_text(channel_id, user, content),
            MessageContent::File(file) => send_file(channel_id, user, file),
        }
    }
}

// Sends Text to discord
fn send_text(channel_id: ChannelId, user: String, content: String) {
    let msg = format!("**{}**: {}", user, content);
    match channel_id.say(msg) {
        Ok(_) => (),
        Err(e) => debug!("Failed to send message: {}", e),
    }
}

// Sends a file to discord
fn send_file(channel_id: ChannelId, user: String, file_msg: FileMessage) {
    let FileMessage {
        caption,
        filename,
        contents,
    } = file_msg;

    // These lines put the attachment in the correct format to send to Discord
    let tup: (&[u8], &str) = (contents.as_ref(), &filename);
    let attachment_type: AttachmentType = AttachmentType::from(tup);
    let attchmnts = vec![attachment_type];

    // Create the message and send it to Discord
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
        Err(e) => debug!("Failed to send files: {}", e),
    }
}
