use irro::arduino::binary::Connection;
use irro::arduino::cmd::led::LedMask;
use std::thread;
use std::time::Duration;

fn main() {
    let sender = Connection::init_from_device("/dev/ttyACM1").unwrap();

    loop {
        LedMask::from_bools(vec![true]).send(&sender);
        thread::sleep(Duration::from_millis(800));

        LedMask::from_bools(vec![false]).send(&sender);
        thread::sleep(Duration::from_millis(800));
    }
}
