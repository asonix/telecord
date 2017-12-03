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

//! This is the library for Telebot.
//!
//! It may not be incredibly useful outside the context of the associated main.rs

#![feature(try_from)]
#![feature(conservative_impl_trait)]

extern crate dotenv;
extern crate telebot;
extern crate serenity;
extern crate futures;
extern crate mime;
extern crate mime_sniffer;
extern crate tokio_curl;
extern crate curl;

#[macro_use]
extern crate log;

mod config;
pub mod tg;
pub mod dc;

pub use config::Config;
