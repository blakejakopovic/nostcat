use clap::{Arg, Command, ArgAction};
use tungstenite::{connect, Message};
use url::Url;

pub struct Item {
    pub url: String,
    pub input: String,
    pub result: Option<Result<Vec<String>, String>>
}

impl Item {

    // TODO: Create constructor method
    pub fn new(url: String, input: String) -> Self {
        Self {
            url: url,
            input: input,
            result: None
        }
    }

    pub async fn resolve(&mut self) {
        log::info!("Connecting to websocket server -- {}", self.url);
        self.result = Some(run(&self.url, &self.input))
    }
}

pub fn cli() -> Command {
    Command::new("nostcat")
        .about("A fictional versioning CLI")
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
             Arg::new("servers")
            .help("Websocket servers")
            .num_args(0..)
            .action(ArgAction::Append)
            .required(true)
            .trailing_var_arg(true)
        )
}

// fn _parse_url(_url_str: &String) -> Result<url::Url, String> {
//   todo!()
// }

// fn _connect2<T, G>(_url: Url) -> Result<(T, G), String> {
//   todo!()
// }

// fn _write_message(_message: String) -> Result<String, String> {
//   todo!()
// }

// fn is_ESOE(data) -> bool {
//   true
// }

// fn is_NOTICE(data) -> bool {
//   true
// }

// fn is_OK(data) -> bool {
//   true
// }

// fn handle_ESOE(socket: &TypeName) {
//   &socket.write_message(Message::Close(None)).unwrap()
// }

// enum ServerCommand {
//     EOSE(String),
//     NOTICE(String),
//     OK(String),
//     EVENT(String),
//     UNHANDLED(String)
// }

// impl ServerCommand {
//   fn from_data(data: String) -> Self {
//         match data {
//             data if data.starts_with(&r#"["EOSE""#) => { ServerCommand::EOSE(data) }
//             data if data.starts_with(&r#"["NOTICE""#) => { ServerCommand::NOTICE(data) }
//             data if data.starts_with(&r#"["OK""#) => { ServerCommand::OK(data) }
//             data if data.starts_with(&r#"["EVENT""#) => { ServerCommand::EVENT(data) }
//             _ => ServerCommand::UNHANDLED(data)
//         }
//     }
// }


// fn _close_socket<T>(_socket: &T) {

// }

pub fn run(url_str: &String, input: &String) -> Result<Vec<String>, String> {

    let url = match Url::parse(url_str) {
      Ok(url) => url,
      Err(err) => return Err(format!("Unable to parse websocket url: {}", err))
    };

    // TODO: Need to add connection timeout, as currently it hangs for bad/failed connections
    // https://users.rust-lang.org/t/tls-websocket-how-to-make-tungstenite-works-with-mio-for-poll-and-secure-websocket-wss-via-native-tls-feature-of-tungstenite-crate/72533/4
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
