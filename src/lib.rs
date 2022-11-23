use clap::{Arg, Command, ArgAction};
use std::sync::mpsc;
use std::io::{self, BufRead};
use tungstenite::{connect, Message};
use url::Url;

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
             .action(ArgAction::SetTrue)
        )
        .arg(
             Arg::new("stream")
             .help("Stream the websocket connection")
             .long("stream")
             .short('s')
             .required(false)
             .num_args(0)
             .action(ArgAction::SetTrue)
        )
        .arg(
             Arg::new("servers")
            .help("Websocket servers")
            .num_args(0..)
            .action(ArgAction::Append)
            .required(true)
            .trailing_var_arg(true)
        )
}

pub fn read_input() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        lines.push(line.unwrap());
    }
    return lines
}

pub fn run(tx: &mpsc::Sender<Result<String, String>>, url_str: String, input: Vec<String>, stream: bool) {

    // Connect to websocket
    let url = match Url::parse(&url_str) {
      Ok(url) => url,
      Err(err) => {
        tx.send(Err(format!("Unable to parse websocket url: {}", err))).unwrap();
        return
      }
    };

    // TODO: Need to add connection timeout, as currently it hangs for bad/failed connections
    // https://users.rust-lang.org/t/tls-websocket-how-to-make-tungstenite-works-with-mio-for-poll-and-secure-websocket-wss-via-native-tls-feature-of-tungstenite-crate/72533/4
    let (mut socket, response) = match connect(url.clone()) {
        Ok((socket, response)) => (socket, response),
        Err(err) => {
          tx.send(Err(format!("Unable to connect to websocket server: {}", err))).unwrap();
          return
        }
    };

    log::info!("Connected to websocket server -- {}", url_str);
    log::info!("Response HTTP code -- {}: {}", url_str, response.status());

    // Send input (stdin)
    for line in input {
      match socket.write_message(Message::Text(line.to_owned())) {
          Ok(_) => {
            log::info!("Sent data -- {}: {}", url_str, line);
          },
          Err(err) => {
            tx.send(Err(format!("Failed to write to websocket: {}", err))).unwrap();
            return
          }
      };
    }

    'run_loop: loop {

        // TODO: Review better error handing for this
        let msg = socket.read_message().unwrap();

        match msg {

            Message::Text(data) => {

                log::info!("Received data -- {}: {}", url_str, data);

                // TODO: Refactor out types / handlers
                match data {

                    // Handle NIP-15: End of Stored Events Notice
                    data if data.starts_with(&r#"["EOSE""#) => {
                        if !stream {
                          socket.write_message(Message::Close(None)).unwrap();
                          break 'run_loop;
                        }
                    },

                    // Handle NIP-20: Command Results
                    // TODO: when piping relay to relay, we don't want to use --stream
                    //       but also don't want to close the websocket on OK
                    data if data.starts_with(&r#"["OK""#) => {

                        tx.send(Ok(data)).unwrap();

                        if !stream {
                          socket.write_message(Message::Close(None)).unwrap();
                          break 'run_loop;
                        }
                    },

                    // Handle NIP-01: NOTICE
                    data if data.starts_with(&r#"["NOTICE""#) => {
                        tx.send(Ok(data)).unwrap();

                        if !stream {
                          socket.write_message(Message::Close(None)).unwrap();
                          break 'run_loop;
                        }
                    },

                    // Handle all other text data
                    _ => {
                        tx.send(Ok(data)).unwrap();
                    }
                }
            },

            Message::Ping(id) => {
              log::info!("Replied with Pong -- {}", url_str);
              socket.write_message(Message::Pong(id)).unwrap();
            },

            _ => {
                 tx.send(Err(format!("Received non-text websocket data: {:?}", msg))).unwrap();
                 return
            }
        }
    }
}
