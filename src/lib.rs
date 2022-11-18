use tungstenite::{connect, Message};
use url::Url;

pub fn run(url_str: &String, input: &String) -> Result<Vec<String>, String> {

    let urx = &url_str;

    let url = match Url::parse(urx) {
      Ok(url) => url,
      Err(err) => return Err(format!("Unable to parse websocket url: {}", err))
    };

    // TODO: Need to add connection timeout, as currently it hangs for bad/failed connections
    let (mut socket, response) = match connect(url) {
        Ok((socket, response)) => (socket, response),
        Err(err) => return Err(format!("Unable to connect to websocket server: {}", err))
    };

    log::info!("Connected to websocket server -- {}", url_str);
    log::info!("Response HTTP code -- {}: {}", url_str, response.status());

    match socket.write_message(Message::Text(input.to_owned())) {
        Ok(_) => (),
        Err(err) => return Err(format!("Failed to write to websocket: {}", err))
    };

    log::info!("Sent data -- {}: {}", url_str, input);

    let mut result: Vec<String> = Vec::new();

    'run_loop: loop {

        // TODO: Review better error handing for this
        let msg = socket.read_message().unwrap();

        match msg {

            Message::Text(data) => {

              log::info!("Received data -- {}: {}", url_str, data);

                match data {

                    // Handle NIP-15: End of Stored Events Notice
                    data if data.starts_with(&r#"["EOSE""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        // Skip pushing EOSE output to results
                        break 'run_loop
                    },

                    // Handle NIP-20: Command Results
                    data if data.starts_with(&r#"["OK""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        result.push(data);
                        break 'run_loop
                    },

                    // Handle NIP-01: NOTICE
                    data if data.starts_with(&r#"["NOTICE""#) => {
                        socket.write_message(Message::Close(None)).unwrap();
                        result.push(data);
                        break 'run_loop
                    },

                    // Handle all other text data
                    _ => {
                        result.push(data)
                    }
                }
            },

            _ => {
                return Err(format!("Received non-text websocket data: {}", msg));
            }
        }
    }

    return Ok(result)
}
