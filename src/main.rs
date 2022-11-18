extern crate log;

use url::Url;
use nostcat::exit_with_error;

fn main() {

    env_logger::init();

    let url_str = std::env::args().nth(1).unwrap_or_else(|| {
        exit_with_error("Missing ws:// or wss:// websocket url argument")
    });

    let url = Url::parse(&url_str).unwrap_or_else(|err| {
        exit_with_error(&format!("Unable to parse websocket url: {}", err))
    });

    nostcat::run(url)
}
