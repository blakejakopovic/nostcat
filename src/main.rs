extern crate log;

use std::io::{self};
use nostcat::exit_with_error;


struct Item {
    url: String,
    input: String,
    result: Result<String, String>
}

impl Item {

    // pub fn new(url: String, input: String) -> Self {
    //     Item {
    //         name: name
    //     }
    // }

    async fn resolve(&mut self) {
        println!("Running: {}, {}", self.url, self.input);
        self.result = nostcat::run(&self.url, &self.input)
    }

    pub fn result(&self) -> String {
        self.url.to_string()
    }
}

#[tokio::main]
async fn main() {

    let arguments: Vec<String> = std::env::args().collect();

    // Handle input from pipe or stdin prompt
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_else(|err| {
        exit_with_error(&format!("Failed to read from stdin: {}", err))
    });

    input = input.trim().to_string();

    // Create items for each websocket server
    let items: Vec<Item> = arguments[1..]
        .into_iter()
        .map(|url_str| Item {
            url: url_str.to_owned(),
            input: input.to_string(),
            result: Ok("".to_string())
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
    for item in items.iter() {
        match &item.result {
            Ok(output) => {
                println!("Output: {}", output)
            },

            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
    }
}


// async fn main() {

//     env_logger::init();

    // // Handle input from pipe or stdin prompt
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).unwrap_or_else(|err| {
    //     exit_with_error(&format!("Failed to read from stdin: {}", err))
    // });

    // input = input.trim().to_string();

//     let mut arguments: Vec<String> = std::env::args().collect();

//     for url_str in &mut arguments[1..] {
//       let _x = run(url_str.to_string(), input.to_string()).await;
//     }

//     let url_str = std::env::args().nth(1).unwrap_or_else(|| {
//         exit_with_error("Missing ws:// or wss:// websocket url argument")
//     });

//     let url = Url::parse(&url_str).unwrap_or_else(|err| {
//         exit_with_error(&format!("Unable to parse websocket url: {}", err))
//     });

//     nostcat::run(url)
// }
