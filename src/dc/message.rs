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

use serenity::model::ChannelId;

pub struct Message {
    pub from: String,
    pub channel_id: ChannelId,
    pub content: MessageContent,
}

impl Message {
    pub fn text(user: String, channel_id: ChannelId, content: String) -> Self {
        Message {
            from: user,
            channel_id: channel_id,
            content: MessageContent::Text(content),
        }
    }

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

pub enum MessageContent {
    Text(String),
    File(FileMessage),
}

pub struct FileMessage {
    pub caption: Option<String>,
    pub filename: String,
    pub contents: Vec<u8>,
}
