use irro::arduino::binary::Connection;
use irro::{api, logging::IrroLogger, network};
use log::info;
use std::io;

fn main() -> io::Result<()> {
    log::set_logger(&IrroLogger).expect("Could not initialize logger.");
    log::set_max_level(log::LevelFilter::Trace);

    info!("Starting Irro...");

    network::start_broadcasting()?;
    let sender = Connection::init_from_device("/dev/ttyACM0").unwrap();
    api::run_http_server(sender)
}
