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

//! This module defines the intermediate Message type used to send messages to Telegram.

use mime;
use telebot::objects::Integer;

/// Message is the outermost structure for the intermediate type.
pub struct Message {
    /// `from` represents the user that sent the original message
    pub from: String,
    /// `chat_id` indicates which chat this message should be sent to
    pub chat_id: Integer,
    /// `content` contains the body of the message
    pub content: MessageContent,
}

impl Message {
    /// Create a new intermediate Message representation for sending Text
    pub fn text(user: String, chat_id: Integer, content: String) -> Self {
        Message {
            from: user,
            chat_id: chat_id,
            content: MessageContent::Text(content),
        }
    }

    /// Create a new intermediate Message representation for sending any kind of File
    pub fn file(
        user: String,
        chat_id: Integer,
        caption: Option<String>,
        filename: String,
        contents: Vec<u8>,
        kind: FileKind,
    ) -> Self {
        Message {
            from: user,
            chat_id: chat_id,
            content: MessageContent::File(FileMessage {
                caption,
                filename,
                contents,
                kind,
            }),
        }
    }
}

/// Defines the content of a message
pub enum MessageContent {
    Text(String),
    File(FileMessage),
}

/// Defines the content of a Message with an attached File.
pub struct FileMessage {
    /// An optional caption for the sent file
    pub caption: Option<String>,
    /// The name of the sent file
    pub filename: String,
    /// The contents of the sent file
    pub contents: Vec<u8>,
    /// The kind of the sent file
    pub kind: FileKind,
}

/// All reasonable kinds of files. The `FileKind::Unknown` variant is used for any kind of file
/// that does not fit into the categories of Image, Video, or Audio and will be sent as a Document
/// to telegram.
pub enum FileKind {
    Image,
    Video,
    Audio,
    Unknown,
}

impl From<mime::Mime> for FileKind {
    fn from(mime: mime::Mime) -> Self {
        match mime.type_() {
            mime::IMAGE => {
                println!("IMAGE");
                FileKind::Image
            }
            mime::VIDEO => {
                println!("VIDEO");
                FileKind::Video
            }
            mime::AUDIO => {
                println!("AUDIO");
                FileKind::Audio
            }
            unknown => {
                println!("UNKNOWN TYPE: {}", unknown);
                FileKind::Unknown
            }
        }
    }
}
