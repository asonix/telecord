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

//! This module contains the function for downloading files from Telegram and the associated Error
//! type.

use std::path::Path;
use telebot::bot::RcBot;
use telebot::objects::File;
use futures::{Future, IntoFuture, Stream};
use hyper::Error as HyperError;
use hyper::error::UriError;
use native_tls::Error as TlsError;

use hyper::{Body, Client, Method, Request};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use std::fmt;

/// This error type contains wrappers around Hyper's errors, and provides a few
/// unit types.
#[derive(Debug)]
pub enum DownloadError {
    /// Wrap errors created by the Hyper library. When making requests
    Hyper(HyperError),
    /// Wrap errors crated by Hyper's URI Parser
    Uri(UriError),
    /// Wrap errors created by Native TLS, used by Hyper Tls
    Tls(TlsError),
    /// If the response code from Telegram is not 2xx, this error will occur
    Not2XX(u16),
    /// If the file requested is too large, this error will occur
    FileTooLarge(i64),
    /// If the file requested has no size metadata, this error will occur
    FileSizeUnknown,
    /// If the filename cannot be determined, this error will occur
    FileName,
}

impl From<HyperError> for DownloadError {
    fn from(err: HyperError) -> Self {
        DownloadError::Hyper(err)
    }
}

impl From<UriError> for DownloadError {
    fn from(err: UriError) -> Self {
        DownloadError::Uri(err)
    }
}

impl From<TlsError> for DownloadError {
    fn from(err: TlsError) -> Self {
        DownloadError::Tls(err)
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            DownloadError::Hyper(ref err) => write!(f, "Hyper: {}", err),
            DownloadError::Uri(ref err) => write!(f, "hyper::Uri: {}", err),
            DownloadError::Tls(ref err) => write!(f, "native_tls: {}", err),
            DownloadError::Not2XX(code) => write!(f, "Response not 2xx: {}", code),
            DownloadError::FileTooLarge(size) => {
                write!(f, "Requested file is too large: {} bytes", size)
            }
            DownloadError::FileSizeUnknown => write!(f, "Could not determine filesize"),
            DownloadError::FileName => write!(f, "Could not determine filename"),
        }
    }
}

fn prepare_request(bot: RcBot, file: File) -> Result<(RcBot, String, String), DownloadError> {
    // If the file_size exists and is less than 8 * 10^6 bytes, continue, else take the
    // error path.
    let file_size = file.file_size.ok_or(DownloadError::FileSizeUnknown)?;
    debug!("file_size: {}", file_size);

    if file_size >= 8 * 1000 * 1000 {
        return Err(DownloadError::FileTooLarge(file_size));
    }

    let path = file.file_path.ok_or(DownloadError::FileName)?;

    let path = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.inner.key, path
    );

    let url = Path::new(&path);
    let filename = url.file_name().ok_or(DownloadError::FileName)?;

    let filename = filename.to_str().ok_or(DownloadError::FileName)?;

    Ok((bot, path.clone(), String::from(filename)))
}

/// Download a file given an `RcBot` and a `telebot::objects::File` object
pub fn download_file(
    bot: RcBot,
    file: File,
) -> impl Future<Item = (Vec<u8>, String), Error = DownloadError> {
    prepare_request(bot, file)
        .into_future()
        .and_then(move |(bot, path, filename)| {
            // Download the file and send the result as an intermediate message representation
            // to the Discord Bot.
            download(&bot, &path).map(|response| (response, filename))
        })
}

fn build_request(
    bot: &RcBot,
    url: &str,
) -> Result<(Client<HttpsConnector<HttpConnector>, Body>, Request<Body>), DownloadError> {
    let client = Client::configure()
        .connector(HttpsConnector::new(2, &bot.inner.handle)?)
        .build(&bot.inner.handle);
    // Create a new request
    let req = Request::new(Method::Get, url.parse()?);
    Ok((client, req))
}

/// Download a file given a URL. This is designed to work on the Tokio threadpool used by Telebot.
/// This function will return a failed future if the response code is not in the range 200 to 299
/// inclusive.
pub fn download(bot: &RcBot, url: &str) -> impl Future<Item = Vec<u8>, Error = DownloadError> {
    build_request(bot, url)
        .into_future()
        .and_then(|(client, req)| client.request(req).map_err(DownloadError::from))
        .and_then(move |res| {
            let code = res.status().as_u16();

            if 200 <= code && code < 300 {
                Ok(res)
            } else {
                Err(DownloadError::Not2XX(code))
            }
        })
        .and_then(|res| {
            res.body()
                .concat2()
                .map(|chunk| chunk.to_vec())
                .map_err(DownloadError::from)
        })
}
