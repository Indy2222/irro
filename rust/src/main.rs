use irro::arduino::binary::Connection;
use irro::{api, logging::IrroLogger, network};
use log::{error, info};
use std::io;
use std::panic;

fn main() -> io::Result<()> {
    log::set_logger(&IrroLogger).expect("Could not initialize logger.");
    log::set_max_level(log::LevelFilter::Trace);

    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
        println!("{}", panic_info);
    }));

    info!("Starting Irro...");

    network::start_broadcasting()?;
    let sender = Connection::init_from_device("/dev/ttyACM0").unwrap();
    api::run_http_server(sender)
}
