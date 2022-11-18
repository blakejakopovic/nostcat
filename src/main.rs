extern crate log;

use clap::{Arg, Command, ArgAction};
use std::io::{self};
use std::process;

struct Item {
    url: String,
    input: String,
    result: Result<Vec<String>, String>
}

impl Item {

    // TODO: Create constructor method
    // pub fn new(url: String, input: String) -> Self {
    //     Item {
    //         url: url,
    //         input: input,
    //         result: Result::Ok(vec![])
    //     }
    // }

    async fn resolve(&mut self) {
        log::info!("Connecting to websocket server -- {}", self.url);
        self.result = nostcat::run(&self.url, &self.input)
    }
}

fn cli() -> Command {
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
        .map(|url_str| Item {
            url: url_str.to_owned(),
            input: input.to_string(),
            result: Ok(vec![])
        }).collect();

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
        match &item.result {
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
