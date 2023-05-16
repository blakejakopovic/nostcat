#[macro_use]
extern crate log;

use clap::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::time::Duration;
use tokio::sync::mpsc;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

#[derive(Clone)]
pub struct Config {
    pub connect_timeout: u64,
    pub stream: bool,
    pub omit_eose: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerResponse {
    pub source_server: String,
    pub response: String,
}

impl ServerResponse {
    fn to_string(self: Self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug)]
pub enum Response {
    Event(String),
    Notice(String),
    Ok(String),
    EOSE(String),
    Count(String),
    Unsupported(String),
}

impl Response {
    pub fn from_string(s: String) -> Self {
        match s {
            s if s.starts_with(r#"["EVENT""#) => Response::Event(s),
            s if s.starts_with(r#"["NOTICE""#) => Response::Notice(s),
            s if s.starts_with(r#"["OK""#) => Response::Ok(s),
            s if s.starts_with(r#"["EOSE""#) => Response::EOSE(s),
            s if s.starts_with(r#"["COUNT""#) => Response::Count(s),
            _ => Response::Unsupported(s),
        }
    }
}

pub fn cli() -> Command {
    Command::new("nostcat")
        .about("Websocket client for nostr relay scripting")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .author("Blake Jakopovic")
        .arg_required_else_help(true)
        .arg(
            Arg::new("unique")
                .help("Sort and unique returned events")
                .long("unique")
                .short('u')
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stream")
                .help("Stream the websocket connection")
                .long("stream")
                .short('s')
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("connect-timeout")
                .help("Websocket connection timeout in milliseconds (non-streaming)")
                .long("connect-timeout")
                .required(false)
                .num_args(1)
                .value_parser(clap::value_parser!(u64))
                .default_value("10000"),
        )
        .arg(
            Arg::new("servers")
                .help("Websocket servers")
                .num_args(0..)
                .action(ArgAction::Append)
                .required(true)
                .trailing_var_arg(true),
        )
}

pub fn read_input() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        lines.push(line.unwrap());
    }
    return lines;
}

pub async fn request(
    tx: mpsc::Sender<Result<String, String>>,
    url_str: &str,
    input: Vec<String>,
    config: Config,
) {
    let url = match Url::parse(&url_str) {
        Ok(url) => url,
        Err(err) => {
            tx.send(Err(format!("Unable to parse websocket url: {}", err)))
                .await
                .unwrap();
            return;
        }
    };

    let (mut socket, response) = match connect(url.clone()) {
        Ok((socket, response)) => (socket, response),
        Err(err) => {
            tx.send(Err(format!(
                "Unable to connect to websocket server: {}",
                err
            )))
            .await
            .unwrap();
            return;
        }
    };

    let stream = config.stream;
    if !stream {
        let timeout_duration = Duration::from_millis(config.connect_timeout);

        let timeout_res = match socket.get_mut() {
            MaybeTlsStream::NativeTls(ref mut s) => {
                s.get_mut().set_read_timeout(Some(timeout_duration))
            }
            MaybeTlsStream::Plain(ref s) => s.set_read_timeout(Some(timeout_duration)),

            t => {
                warn!("{:?} not handled, not setting read timeout", t);
                Ok(())
            }
        };

        match timeout_res {
            Err(err) => error!("Error setting timeout: {}", err),
            Ok(_) => info!("Setting timeout to {} ms", config.connect_timeout),
        }
    }

    info!("Connected to websocket server -- {}", url_str);
    info!("Response HTTP code -- {}: {}", url_str, response.status());

    for line in input {
        match socket.write_message(Message::Text(line.to_owned())) {
            Ok(_) => {
                info!("Sent data -- {}: {}", url_str, line);
            }
            Err(err) => {
                tx.send(Err(format!("Failed to write to websocket: {}", err)))
                    .await
                    .unwrap();
                return;
            }
        };
    }

    'run_loop: loop {
        let msg = socket.read_message();

        if let Err(err) = msg.as_ref() {
            let errmsg = format!("Websocket read message error: {}", err);

            // Connection timeout detected
            if errmsg.contains("Resource temporarily unavailable") {
                let timeout_msg = format!(
                    "Connection timed out after {}ms while waiting for a response -- {}",
                    config.connect_timeout, url_str
                );
                info!("{}", timeout_msg);
                tx.send(Err(timeout_msg)).await.unwrap();
                return;
            }

            tx.send(Err(errmsg)).await.unwrap();
            break;
        }

        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                info!("Received data -- {}: {}", url_str, data);

                match Response::from_string(data.clone()) {
                    // Handle NIP-15: End of Stored Events Notice
                    Response::EOSE(data) => {
                        if !config.omit_eose {
                            let server_response = ServerResponse {
                                source_server: url_str.to_string(),
                                response: data.clone(),
                            };
                            tx.send(Ok(server_response.to_string())).await.unwrap();
                        }

                        if !stream {
                            info!("Closing websocket -- {}: {}", data, url_str);
                            socket.close(None).unwrap();
                            break 'run_loop;
                        }
                    }

                    // Handle NIP-20: Command Results
                    // TODO: when piping relay to relay, we don't want to use --stream
                    //       but also don't want to close the websocket on OK
                    Response::Ok(data) => {
                        let server_response = ServerResponse {
                            source_server: url_str.to_string(),
                            response: data.clone(),
                        };
                        tx.send(Ok(server_response.to_string())).await.unwrap();

                        if !stream {
                            info!("Closing websocket -- {}: {}", data, url_str);
                            socket.close(None).unwrap();
                            break 'run_loop;
                        }
                    }

                    // Handle NIP-01: NOTICE
                    Response::Notice(data) => {
                        let server_response = ServerResponse {
                            source_server: url_str.to_string(),
                            response: data.clone(),
                        };
                        tx.send(Ok(server_response.to_string())).await.unwrap();

                        if !stream {
                            info!("Closing websocket -- {}: {}", data, url_str);
                            socket.close(None).unwrap();
                            break 'run_loop;
                        }
                    }

                    // Handle NIP-45: COUNT
                    Response::Count(data) => {
                        let server_response = ServerResponse {
                            source_server: url_str.to_string(),
                            response: data.clone(),
                        };
                        tx.send(Ok(server_response.to_string())).await.unwrap();

                        if !stream {
                            info!("Closing websocket -- {}: {}", data, url_str);
                            socket.close(None).unwrap();
                            break 'run_loop;
                        }
                    }

                    Response::Event(data) => {
                        let server_response = ServerResponse {
                            source_server: url_str.to_string(),
                            response: data,
                        };
                        tx.send(Ok(server_response.to_string())).await.unwrap();
                    }

                    // Handle unsupported nostr data
                    Response::Unsupported(data) => {
                        tx.send(Err(format!("Received unsupported nostr data: {:?}", data)))
                            .await
                            .unwrap();
                    }
                }
            }

            Message::Ping(id) => {
                info!("Replied with Pong -- {}", url_str);
                socket.write_message(Message::Pong(id)).unwrap();
            }

            _ => {
                tx.send(Err(format!("Received non-text websocket data: {:?}", msg)))
                    .await
                    .unwrap();
                return;
            }
        }
    }
}
