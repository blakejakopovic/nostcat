#[macro_use]
extern crate log;

use nostcat::{Config, ServerResponse};
use nostcat::{cli, request, read_input};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let cli_matches = cli().get_matches();

    let servers: Vec<String> = cli_matches
        .get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let input = read_input();

    let (tx, mut rx) = mpsc::channel(100);

    let config: Config = Config{
      connect_timeout: *cli_matches.get_one("connect-timeout").unwrap(),
      stream: cli_matches.get_flag("stream"),
      omit_eose: true,
    };

    for server in servers {
        let tx2 = tx.clone();
        let input = input.clone();
        let config = config.clone();

        info!("Spawning async for -- {}", server);

        tokio::spawn(async move {
            request(tx2, &server, input, config).await
        });
    }

    // drop the original tx, as it was never used
    drop(tx);

    // TODO: Perhaps change to event id, or hash of event (as less data)
    let mut seen: Vec<String> = vec![];

    'recv_loop: loop {

        let receive = rx.recv().await;

        match receive {
            None => {
                info!("All websockets channels now closed");
                break 'recv_loop;
            },

            Some(Err(err)) => { eprintln!("{}", err) },

            Some(Ok(message)) => {

                let server_response: ServerResponse = serde_json::from_str(&message).unwrap();
                let response = server_response.response;

                if cli_matches.get_flag("unique") {

                    if seen.contains(&response) {
                        continue;
                    }

                    seen.push(response.clone());
                }

                println!("{}", response);
            }
        }
    }

    Ok(())
}
