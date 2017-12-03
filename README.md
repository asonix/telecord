# Telecord

Telecord is a bot that translates Discord messages to Telegram Messages (and the other way around).

[Documentation](https://docs.rs/telecord)

### Current Status
__it works__

- Sends text and mediate from Discord to Telegram
- Sends text and media from Telegram to Discord

### Usage

Make sure you have the proper environment variables set. You can do this with `export` or by modifying the `.env` file. A sample `.env` is provided in this crate as `.env.sample`

`DISCORD_BOT_TOKEN` and `TELEGRAM_BOT_TOKEN` are self-explanatory, but `CHAT_MAPPINGS` is a comma separated list of colon-separated tuples. The environment variable `1234:abcd,5678,efgh` maps telegram chat `1234` to discord channel `abcd`, and also maps telegram chat `5678` to discord channel `efgh`.

Once you have your environment variable set, you can use `cargo run` to run the bot.

### License

Telecord is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Telecord is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Telecord

You should have received a copy of the GNU General Public License along with Telecord If not, see http://www.gnu.org/licenses/.
