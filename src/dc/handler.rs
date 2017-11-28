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

//! This module defines how to respond to messages from Discord.

use serenity::model;
use serenity::prelude::*;
use futures::sync::mpsc::Sender;

use tg;
use config::Config;
use super::telegram;

/// Defines the Handler type, which is used in the Serenity Library to handle incoming messages
/// from Discord.
pub struct Handler {
    /// `config` is present so the Handler has access to the function translating Discord
    /// Channel IDs to Telegram Chat IDs.
    config: Config,
    /// `tg_sender` is present to send intermediate message representations to the telegram
    /// handler.
    tg_sender: Sender<tg::Message>,
}

impl Handler {
    /// Creates a new Handler struct from a Config and Sender.
    pub fn new(config: Config, tg_sender: Sender<tg::Message>) -> Self {
        Handler { config, tg_sender }
    }

    /// Handles regular chat messages, such as people sending text or files.
    fn regular_message(&self, _: Context, message: model::Message) {
        telegram::regular_message(&self.config, self.tg_sender.clone(), message);
    }

    /// Handles messages indicating a user has Joined the chat.
    fn join_message(&self, _: Context, message: model::Message) {
        telegram::join_message(message);
    }
}

impl EventHandler for Handler {
    fn on_message(&self, ctx: Context, message: model::Message) {
        match message.kind {
            model::MessageType::Regular => {
                self.regular_message(ctx, message);
            }
            model::MessageType::MemberJoin => {
                self.join_message(ctx, message);
            }
            _ => {}
        }
    }
}
