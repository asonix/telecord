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

//! This module defines the intermediate representation of a Discord message.

use serenity::model::ChannelId;

/// The outermost layer of the Discord Message representation
pub struct Message {
    /// `from` indicates the user that originally sent the message
    pub from: String,
    /// `channel_id` determines which channel the message should be sent to
    pub channel_id: ChannelId,
    /// `content` defines the contents of the message
    pub content: MessageContent,
}

impl Message {
    /// Create a new text message representation
    pub fn text(user: String, channel_id: ChannelId, content: String) -> Self {
        Message {
            from: user,
            channel_id: channel_id,
            content: MessageContent::Text(content),
        }
    }

    /// Create a new file message representation
    pub fn file(
        user: String,
        channel_id: ChannelId,
        caption: Option<String>,
        filename: String,
        contents: Vec<u8>,
    ) -> Self {
        Message {
            from: user,
            channel_id: channel_id,
            content: MessageContent::File(FileMessage {
                caption,
                filename,
                contents,
            }),
        }
    }
}

/// Enumerate the kinds of messages that can be sent
pub enum MessageContent {
    Text(String),
    File(FileMessage),
}

/// Define the required information for sending files.
///
/// Unlike Telegram, Discord does not care what kind of file is being uploaded. The server decides
/// how to represent the file to the clients.
pub struct FileMessage {
    /// `caption` is the optional text associated with the file
    pub caption: Option<String>,
    /// `filename` is the name of the file
    pub filename: String,
    /// `contents` is the contents of a file represented in-memory as a `Vec<u8>`
    pub contents: Vec<u8>,
}
