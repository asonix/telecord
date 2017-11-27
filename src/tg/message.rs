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

use mime;

pub struct Message {
    pub from: String,
    pub content: MessageContent,
}

impl Message {
    pub fn text(user: String, content: String) -> Self {
        Message {
            from: user,
            content: MessageContent::Text(content),
        }
    }

    pub fn file(
        user: String,
        caption: Option<String>,
        filename: String,
        contents: Vec<u8>,
        kind: FileKind,
    ) -> Self {
        Message {
            from: user,
            content: MessageContent::File(FileMessage {
                caption,
                filename,
                contents,
                kind,
            }),
        }
    }
}

pub enum MessageContent {
    Text(String),
    File(FileMessage),
}

pub struct FileMessage {
    pub caption: Option<String>,
    pub filename: String,
    pub contents: Vec<u8>,
    pub kind: FileKind,
}

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
