
use std::process;
use tungstenite::{connect, Message};
use url::Url;

// TODO: Review https://github.com/snapview/tokio-tungstenite/blob/master/examples/client.rs

// enum CommandResult {
//   0: REQUEST(str, str)
//   1: EVENT(str, str, str)
//   2: OK(str, str)
//   3: NOTICE(str)
// }

pub fn exit_with_error(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1)
}

pub fn run(url_str: &String, input: &String) -> Result<String, String> {

    let urx = &url_str;

    let url = Url::parse(urx).unwrap_or_else(|err| {
        exit_with_error(&format!("Unable to parse websocket url: {}", err))
    });

    // TODO: Need to add connection timeout, as currently hangs for failed connections
    let (mut socket, response) = connect(url).unwrap_or_else(|err| {
        exit_with_error(&format!("Unable to connect to websocket server: {}", err))
    });

    log::info!("Connected to websocket server");
    log::info!("Response HTTP code: {}", response.status());

    socket.write_message(Message::Text(input.to_owned())).unwrap_or_else(|err| {
        exit_with_error(&format!("Failed to write to websocket: {}", err))
    });

    log::info!("Sent data: {}", input);

    'run_loop: loop {

        let msg = socket.read_message().unwrap_or_else(|err| {
            exit_with_error(&format!("Unable to read websocket messages: {}", err))
        });

        let mut result = String::new();
        match msg {

            Message::Text(data) => {

              log::info!("Received data: {}", data);

                match data {

                    // Handle NIP-15: End of Stored Events Notice
                    data if data.starts_with(&r#"["EOSE""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        break 'run_loop Ok(result.to_string());
                    },

                    // Handle NIP-20: Command Results
                    data if data.starts_with(&r#"["OK""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        result.push_str(&data.to_string());
                        break 'run_loop Ok(result.to_string());
                    },

                    // Handle NIP-01: NOTICE
                    data if data.starts_with(&r#"["NOTICE""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        result.push_str(&data.to_string());
                        break 'run_loop Ok(result.to_string());
                    },

                    // Handle all other text data
                    _ => {
                        result.push_str(&data.to_string());
                    }
                }
            },

            _ => {
                break 'run_loop Err(format!("Received unsupported websocket data type (non-text): {}", msg));
            }
        }
    }
}
