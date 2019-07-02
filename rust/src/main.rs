use irro::api;
use irro::arduino::binary::Connection;
use std::io;

fn main() -> io::Result<()> {
    let sender = Connection::init_from_device("/dev/ttyACM1").unwrap();
    api::run_http_server(sender)
}
