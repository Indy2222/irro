use irro::arduino::binary::Connection;
use irro::{api, logging::IrroLogger, network};
use log::{error, info};
use std::panic;

fn main() {
    log::set_logger(&IrroLogger).expect("Could not initialize logger.");
    log::set_max_level(log::LevelFilter::Trace);

    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
        println!("{}", panic_info);
    }));

    info!("Starting Irro...");

    match network::start_broadcasting() {
        Ok(socket) => socket,
        Err(error) => panic!("Error while starting broadcast loop: {}", error),
    }

    let sender = match Connection::init_from_device("/dev/ttyACM0") {
        Ok(sender) => sender,
        Err(error) => panic!("Error while connecting to Arduino: {}", error),
    };

    if let Err(error) = api::run_http_server(sender) {
        panic!("Error while starting HTTP server: {}", error);
    }
}
