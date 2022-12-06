extern crate log;

use nostcat::{cli, run, read_input};
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

    for server in servers {
        let tx2 = tx.clone();
        let input = input.clone();
        let cli_matches = cli_matches.clone();

        log::info!("Spawning thread for -- {}", server);

        tokio::spawn(async move {
            run(tx2, &server, input, cli_matches).await
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
                log::info!("All websockets channels now closed");
                break 'recv_loop;
            },

            Some(Err(err)) => { eprintln!("{}", err) },

            Some(Ok(line)) => {

                if cli_matches.get_flag("unique") {

                    if seen.contains(&line) {
                        continue;
                    }

                    seen.push(line.clone());
                }

                println!("{}", line);
            }
        }
    }

    Ok(())
}
