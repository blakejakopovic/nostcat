use std::io::{self};
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
    println!("{}", msg);
    process::exit(1)
}

pub fn run(url: Url) {
    // TODO: Need to add connection timeout, as currently hangs for failed connections
    let (mut socket, response) = connect(url).unwrap_or_else(|err| {
        exit_with_error(&format!("Unable to connect to websocket server: {}", err))
    });

    log::info!("Connected to websocket server");
    log::info!("Response HTTP code: {}", response.status());

    // Handle input from pipe or stdin prompt
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_else(|err| {
        exit_with_error(&format!("Failed to read from stdin: {}", err))
    });

    input = input.trim().to_string();

    socket.write_message(Message::Text(input.to_owned())).unwrap_or_else(|err| {
        exit_with_error(&format!("Failed to write to websocket: {}", err))
    });

    log::info!("Sent data: {}", input);

    loop {

        let msg = socket.read_message().unwrap_or_else(|err| {
            exit_with_error(&format!("Unable to read websocket messages: {}", err))
        });

        match msg {

            Message::Text(data) => {

              log::info!("Received data: {}", data);

                match data {

                    // Handle NIP-15: End of Stored Events Notice
                    data if data.starts_with(&r#"["EOSE""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        return
                    },

                    // Handle NIP-20: Command Results
                    data if data.starts_with(&r#"["OK""#) => {
                        println!("{}", data);
                        socket.write_message(Message::Close(None)).unwrap();
                        return
                    },

                    // Handle NIP-01: NOTICE
                    data if data.starts_with(&r#"["NOTICE""#) => {
                        println!("{}", data);
                        socket.write_message(Message::Close(None)).unwrap();
                        return
                    },

                    // Handle all other text data
                    _ => {
                        println!("{}", data);
                    }
                }
            },

            _ => {
                exit_with_error(&format!("Received unsupported websocket data type (non-text): {}", msg))
            }
        }
    }
}
