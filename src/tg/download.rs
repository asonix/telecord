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
use futures::future::{Future, result};
use tokio_curl::PerformError;
use curl::Error as CurlError;
use tokio_curl::Session;
use curl::easy::Easy;

use std::sync::{Arc, Mutex};
use std::fmt;

/// This error type contains wrappers around Curl's and Tokio-Curl's errors, and provides a few
/// unit types.
#[derive(Debug)]
pub enum DownloadError {
    /// Wrap errors created by the curl library, such as when using get(), url(), etc.
    Curl(CurlError),
    /// Wrap errors created by the Tokio-Curl libray in session.perform()
    TokioCurl(PerformError),
    /// If the response code from Telegram is not 2xx, this error will occur
    Not2XX(u32),
    /// If the file requested is too large, this error will occur
    FileTooLarge(i64),
    /// If the file requested has no size metadata, this error will occur
    FileSizeUnknown,
    /// If the filename cannot be determined, this error will occur
    FileName,
    /// If the mutex for accessing the result Vec<u8> is currently locked (it shouldn't be), this
    /// error will occur.
    Lock,
    /// If the response code from Telegram does not exist, this error will occur
    NoCode,
}

impl From<CurlError> for DownloadError {
    fn from(err: CurlError) -> Self {
        DownloadError::Curl(err)
    }
}

impl From<PerformError> for DownloadError {
    fn from(err: PerformError) -> Self {
        DownloadError::TokioCurl(err)
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &DownloadError::Curl(ref err) => write!(f, "Curl: {}", err),
            &DownloadError::TokioCurl(ref err) => write!(f, "TokioCurl: {}", err),
            &DownloadError::Lock => write!(f, "Could not get lock on result vec"),
            &DownloadError::Not2XX(code) => write!(f, "Response not 2xx: {}", code),
            &DownloadError::NoCode => write!(f, "Response did not include a response code"),
            &DownloadError::FileTooLarge(size) => {
                write!(f, "Requested file is too large: {} bytes", size)
            }
            &DownloadError::FileSizeUnknown => write!(f, "Could not determine filesize"),
            &DownloadError::FileName => write!(f, "Could not determine filename"),
        }
    }
}

/// Download a file given an RcBot and a telebot::objects::File object
pub fn download_file(
    bot: RcBot,
    file: File,
) -> impl Future<Item = (Vec<u8>, String), Error = DownloadError> {
    result(Ok((bot, file)))
        .and_then(move |(bot, file)| {
            // If the file_size exists and is less than 8 * 10^6 bytes, continue, else take the
            // error path.
            if let Some(file_size) = file.file_size {
                println!("file_size: {}", file_size);

                if file_size < 8 * 1000 * 1000 {
                    Ok((bot, file))
                } else {
                    Err(DownloadError::FileTooLarge(file_size))
                }
            } else {
                Err(DownloadError::FileSizeUnknown)
            }
        })
        .and_then(move |(bot, file)| {
            // If the file_path exists, get the filename from it and continue, else take the
            // error path.
            if let Some(path) = file.file_path {
                let path = format!(
                    "https://api.telegram.org/file/bot{}/{}",
                    bot.inner.key,
                    path
                );
                let url = Path::new(&path);
                let filename = url.file_name();

                if let Some(filename) = filename {
                    if let Some(filename) = filename.to_str() {
                        Ok((bot, path.clone(), String::from(filename)))
                    } else {
                        Err(DownloadError::FileName)
                    }
                } else {
                    Err(DownloadError::FileName)
                }
            } else {
                Err(DownloadError::FileName)
            }
        })
        .and_then(move |(bot, path, filename)| {
            // Download the file and send the result as an intermediate message representation
            // to the Discord Bot.
            download(bot.clone(), path).map(|response| (response, filename))
        })
}

/// Download a file given a URL. This is designed to work on the Tokio threadpool used by Telebot.
/// This function will return a failed future if the response code is not in the range 200 to 299
/// inclusive.
pub fn download(bot: RcBot, url: String) -> impl Future<Item = Vec<u8>, Error = DownloadError> {
    // Create a tokio-curl session on the bot's event loop
    let session = Session::new(bot.inner.handle.clone());
    let response = Arc::new(Mutex::new(Vec::new()));
    let r2 = response.clone();

    // Create a new request
    result(Ok(Easy::new())).and_then(move |mut req| {
        req.get(true).map(|_| req).map_err(|e| e.into())
    }).and_then(move |mut req| {
        println!("url: {}", url);
        req.url(url.as_ref()).map(|_| req).map_err(|e| e.into())
    }).and_then(move |mut req| {
        // Define the callback function for the curl request. Here we use an Arc<Mutex<Vec<u8>>> in
        // order to share this vector between the curl callback and the response callback. This is the
        // same logic used by the Telebot crate to fetch data from Telegram (11/28/17)..
        req.write_function(move |data| {
            match r2.lock() {
                Ok(ref mut vec) => {
                    vec.extend_from_slice(data);
                    Ok(data.len())
                },
                Err(_) => Ok(0),
            }
        }).map(|_| req).map_err(|e| e.into())
    }).and_then(move |req| {
        // Create a future perform the request and then take the error path if the response code is not
        // between 200 and 299 inclusive
        session.perform(req).map_err(|e| e.into()).and_then(
            move |mut res| {
                if let Ok(code) = res.response_code() {
                    if 200 <= code && code < 300 {
                        match response.lock() {
                            Ok(ref response) => Ok(response.to_vec()),
                            Err(_) => Err(DownloadError::Lock),
                        }
                    } else {
                        Err(DownloadError::Not2XX(code))
                    }
                } else {
                    Err(DownloadError::NoCode)
                }
            },
        )
    })
}
