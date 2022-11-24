use clap::{Arg, ArgAction, Command};
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

#[derive(Debug)]
enum Response {
    Event(String),
    Notice(String),
    Ok(String),
    EOSE(String),
    Unsupported(String)
}

impl Response {
    fn from_string(s: String) -> Self {
        match s {
            s if s.starts_with("[\"EVENT\"") => Response::Event(s),
            s if s.starts_with("[\"NOTICE\"") => Response::Notice(s),
            s if s.starts_with("[\"OK\"") => Response::Ok(s),
            s if s.starts_with("[\"EOSE\"") => Response::EOSE(s),
            _ => Response::Unsupported(s)
        }
    }
}

pub fn cli() -> Command {
    Command::new("nostcat")
        .about("Websocket client for nostr relay scripting")
        .version("0.3.0")
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

pub fn run(
    tx: &mpsc::Sender<Result<String, String>>,
    url_str: String,
    input: Vec<String>,
    stream: bool,
) {
    // Connect to websocket
    let url = match Url::parse(&url_str) {
        Ok(url) => url,
        Err(err) => {
            tx.send(Err(format!("Unable to parse websocket url: {}", err)))
                .unwrap();
            return;
        }
    };

    // TODO: Need to add connection timeout, as currently it hangs for bad/failed connections
    // https://users.rust-lang.org/t/tls-websocket-how-to-make-tungstenite-works-with-mio-for-poll-and-secure-websocket-wss-via-native-tls-feature-of-tungstenite-crate/72533/4
    let (mut socket, response) = match connect(url.clone()) {
        Ok((socket, response)) => (socket, response),
        Err(err) => {
            tx.send(Err(format!(
                "Unable to connect to websocket server: {}",
                err
            )))
            .unwrap();
            return;
        }
    };

    if !stream {
        let timeout_ms = 1000;
        let timeout = Duration::from_millis(timeout_ms);

        let timeout_res = match socket.get_mut() {
            MaybeTlsStream::NativeTls(ref mut s) => s.get_mut().set_read_timeout(Some(timeout)),
            MaybeTlsStream::Plain(ref s) => s.set_read_timeout(Some(timeout)),

            t => {
                log::warn!("{:?} not handled, not setting read timeout", t);
                Ok(())
            }
        };

        match timeout_res {
            Err(err) => log::error!("error setting timeout: {}", err),
            Ok(_) => log::info!("Setting timeout to {} ms", timeout_ms),
        }
    }

    log::info!("Connected to websocket server -- {}", url_str);
    log::info!("Response HTTP code -- {}: {}", url_str, response.status());

    // Send input (stdin)
    for line in input {
        match socket.write_message(Message::Text(line.to_owned())) {
            Ok(_) => {
                log::info!("Sent data -- {}: {}", url_str, line);
            }
            Err(err) => {
                tx.send(Err(format!("Failed to write to websocket: {}", err)))
                    .unwrap();
                return;
            }
        };
    }

    'run_loop: loop {
        // TODO: Review better error handing for this
        let msg = socket.read_message();

        if let Err(err) = msg.as_ref() {
            let errmsg = format!("read error: {}", err);
            if errmsg.contains("Resource temporarily unavailable") {
                let timeout_msg = format!("{} timed out when waiting for a response", url_str);
                tx.send(Err(timeout_msg)).unwrap();
                return;
            }
            tx.send(Err(errmsg)).unwrap();
            break;
        }

        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                log::info!("Received data -- {}: {}", url_str, data);

                match Response::from_string(data.clone()) {

                    // Handle NIP-15: End of Stored Events Notice
                    Response::EOSE(_) => {
                        if !stream {
                            socket.write_message(Message::Close(None)).unwrap();
                            break 'run_loop;
                        }
                    }

                    // Handle NIP-20: Command Results
                    // TODO: when piping relay to relay, we don't want to use --stream
                    //       but also don't want to close the websocket on OK
                    Response::Ok(data) => {

                        tx.send(Ok(data)).unwrap();

                        if !stream {
                            socket.write_message(Message::Close(None)).unwrap();
                            break 'run_loop;
                        }
                    }

                    // Handle NIP-01: NOTICE
                    Response::Notice(data) => {
                        tx.send(Ok(data)).unwrap();

                        if !stream {
                            socket.write_message(Message::Close(None)).unwrap();
                            break 'run_loop;
                        }
                    }

                    Response::Event(data) => {
                        tx.send(Ok(data)).unwrap();
                    },

                    // Handle unsupported nostr data
                    Response::Unsupported(data) => {
                        tx.send(Err(format!("Received unsupported nostr data: {:?}", data))).unwrap();
                    }
                }
            }

            Message::Ping(id) => {
                log::info!("Replied with Pong -- {}", url_str);
                socket.write_message(Message::Pong(id)).unwrap();
            }

            _ => {
                tx.send(Err(format!("Received non-text websocket data: {:?}", msg)))
                    .unwrap();
                return;
            }
        }
    }
}
