use irro::arduino::binary::Connection;
use irro::{api, network};
use std::io;

fn main() -> io::Result<()> {
    network::start_broadcasting()?;
    let sender = Connection::init_from_device("/dev/ttyACM1").unwrap();
    api::run_http_server(sender)
}
