extern crate log;

use nostcat::{cli, run, read_input};
use std::sync::mpsc;
use std::thread;

fn main() {

    env_logger::init();

    let cli_matches = cli().get_matches();

    let servers: Vec<String> = cli_matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let stream = cli_matches.get_flag("stream");

    let input = read_input();

    let (tx, rx) = mpsc::channel();

    let mut v = Vec::<std::thread::JoinHandle<()>>::new();
    for server in servers {
      let tx = tx.clone();
      let input = input.clone();

      log::info!("Spawning thread for -- {}", server);

      let jh = thread::spawn( move || {
        run(&tx, server, input, stream)
      });

      v.push(jh);
    };

    // drop the original tx, as it was never used
    drop(tx);

    let mut seen: Vec<String> = vec![];
    for line in rx {

        match line {
            Err(e) => { eprintln!("{}", e) },
            Ok(line) => {

              if cli_matches.get_flag("unique") {

                  if seen.contains(&line) {
                      continue;
                  }

                  seen.push(line.clone());
              }

              println!("{}", line);
            }
        };
    }

    for jh in v {
        jh.join().unwrap();
    }
}
