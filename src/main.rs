extern crate log;

use nostcat::{cli, Item};
use std::io::{self};
use std::process;


#[tokio::main]
async fn main() {

    env_logger::init();

    let cli_matches = cli().get_matches();

    let servers: Vec<String> = cli_matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(err) => {
          eprintln!("Unable to read from stdin: {}", err);
          process::exit(1)
        }
    }

    input = input.trim().to_string();

    // Create items for each websocket server
    let items: Vec<Item> = servers
        .into_iter()
        .map(|url| Item::new(
            url.to_string(),
            input.to_string()))
        .collect();

    // Create tasks for each item
    let tasks: Vec<_> = items
        .into_iter()
        .map(|mut item| {
            tokio::spawn(async {
                item.resolve().await;
                item
            })
        })
        .collect();

    // Await for all tasks to complete
    let mut items = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }

    // Process task results
    let mut results = vec![];
    for item in items.iter() {
        match &item.result.as_ref().unwrap() {
            Ok(output) => {
                results.extend(output);
            },
            Err(error) => {
                eprintln!("Error processing results websocket server: {} {}", item.url, error)
            }
        };
    }

    if cli_matches.get_flag("unique") {
        results.sort();
        results.dedup();
    }

    for result in results {
        println!("{}", result);
    }
}
