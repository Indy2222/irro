use libirroctl::network;
use log::info;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Stderr).unwrap();
    let irro_ip = network::discover_irro().unwrap();
    print!("{}", irro_ip); // print to STDOUT for machine readability
    info!("Irro's IP: {}", irro_ip);
}
