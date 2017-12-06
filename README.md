# Telecord

Telecord is a bot that translates Discord messages to Telegram Messages (and the other way around).

[Documentation](https://docs.rs/telecord)
[Crates.io](https://crates.io/crates/telecord)

### Current Status
_it works_

- Sends text and media from Discord to Telegram
- Sends text and media from Telegram to Discord

### Getting the bot
#### From binary release
Grab a download from the [current release](https://github.com/asonix/telecord/releases). If there is no release for your operating system or architecture, go to the **Crates.io or From Source** sections of this readme.

#### From Crates.io
This project requires Rust Nightly to compile. If you don't have [Rustup](https://www.rustup.rs/), this is where you install it.

The following command will download the latest stable version of Telecord from crates.io, compile it, and install the binary to `~/.cargo/bin/telecord`
```bash
rustup run nightly cargo install telecord
```

#### From source
This project requires Rust Nightly to compile. If you don't have [Rustup](https://www.rustup.rs/), this is where you install it.

```bash
git clone https://github.com/asonix/telecord.git

rustup install nightly
rustup run nightly cargo build --release
```
This will create a binary in `./target/release` that you can run with `./target/release/telecord`, or you can copy it wherever you need.

### Usage
Make sure you have the proper environment variables set. You can do this with `export` or by modifying the `.env` file. A sample `.env` is provided in this crate as `.env.sample`

`DISCORD_BOT_TOKEN` and `TELEGRAM_BOT_TOKEN` are self-explanatory, but `CHAT_MAPPINGS` is a comma separated list of colon-separated tuples. The environment variable `1234:abcd,5678:efgh` maps telegram chat `1234` to discord channel `abcd`, and also maps telegram chat `5678` to discord channel `efgh`.

Once you have your environment variable set, you can use `./path/to/your/telecord/binary` to run the bot. Please note that the bot must be added to and be able to read messages in the Discord Channels and Telegram Chats it is meant to connect.

#### Running the program inline
```bash
TELEGRAM_BOT_TOKEN="your token" \
DISCORD_BOT_TOKEN="your token" \
CHAT_MAPPINGS="your mappings" \
RUST_LOG=telecord=info \
./path/to/your/telecord/binary
```

#### As a SystemD process
Make sure you have a user and group you wish to run the bot as. If you don't, you can run it as your own user, or create a new user and group.
```
[Unit]
Description=A bot to connect Telegram to Discord
After=network.target

[Service]
Type=simple
User=your-telecord-user
Group=your-telecord-group
Environment="TELEGRAM_BOT_TOKEN=YOUR_BOT_TOKEN"
Environment="DISCORD_BOT_TOKEN=YOUR_BOT_TOKEN"
Environment="CHAT_MAPPINGS=CHAT_ONE:CHANNEL_ONE,CHAT_TWO:CHANNEL_TWO"
Environment="RUST_LOG=telecord=info"
ExecStart=/path/to/your/telecord/binary
TimeoutSec=90
Restart=always

[Install]
WantedBy=default.target
```

#### On Windows
##### With Powershell and the -msvc release
In powershell, navigate to the folder that contains the .exe file and run the following commands
```powershell
$env:RUST_LOG = "telecord=info"
$env:TELEGRAM_BOT_TOKEN = "YOUR_TELEGRAM_TOKEN"
$env:DISCORD_BOT_TOKEN = "YOUR_DISCORD_TOKEN"
$env:CHAT_MAPPINGS = "chat_one:channel_one,chat_two:channel_two"
.\telecord.exe
```
##### With MingW and the -gnu release
In bash, navigate to the folder that contains the .exe file and run the following comands
```bash
TELEGRAM_BOT_TOKEN="your token" \
DISCORD_BOT_TOKEN="your token" \
CHAT_MAPPINGS="your mappings" \
RUST_LOG=telecord=info \
./telecord.exe
```

### License

Telecord is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Telecord is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Telecord

You should have received a copy of the GNU General Public License along with Telecord If not, see http://www.gnu.org/licenses/.
