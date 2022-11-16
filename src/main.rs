use std::io::{self};
use tungstenite::{connect, Message};
use url::Url;

fn main() {

  // First argument is server url
  let url = std::env::args().nth(1).expect("Missing websocket url");

  // Connect to websocket
  let (mut socket, _response) = connect(
      Url::parse(&url).unwrap()
  ).expect("Can't connect to websocket");

  // TODO: handle connection failure

  // Handle input from pipe or stdin prompt
  let mut input = String::new();
  io::stdin()
      .read_line(&mut input)
      .expect("Failed to read from input");
  input = input.trim().to_string();

  // Send input to websocket
  socket.write_message(Message::Text(input)).unwrap();

  loop {
    // Process received messages
    let msg = socket.read_message().expect("Error reading message");

    match msg {

      // Process text only messages for now
      Message::Text(data) => {

        match data {
          // Handle NIP-15: End of Stored Events Notice
          data if data.starts_with(&r#"["EOSE""#) => {
            socket.write_message(Message::Close(None)).unwrap();
            return;
          }

          // Handle NIP-20: Command Results
          data if data.starts_with(&r#"["OK""#) => {
            println!("{}", data);
            socket.write_message(Message::Close(None)).unwrap();
            return;
          }

          // Handle NIP-01: NOTICE
          data if data.starts_with(&r#"["NOTICE"#) => {
            println!("{}", data);
            socket.write_message(Message::Close(None)).unwrap();
            return;
          }

          _ => {
            println!("{}", data);
          }
        }
      }
      _ => {}
    }
  }
}
