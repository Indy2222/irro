extern crate serial;

use serial::prelude::*;
use std::io::prelude::*;
use std::io::BufReader;

use std::time::Duration;

// see: https://www.arduino.cc/en/Serial/Begin
const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud115200,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

fn main() {
    let mut port = serial::open("/dev/ttyACM1").unwrap();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(Duration::from_secs(30)).unwrap();
    let reader = BufReader::new(port);

    for line in reader.lines() {
        let line = line.unwrap();
        println!("Line: {}", line);
    }
}
