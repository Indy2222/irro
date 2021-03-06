use clap::{App, AppSettings, Arg, SubCommand};
use irro::arduino::binary::Connection;
use irro::{api, logging::IrroLogger, network, update};
use log::{error, info};
use std::panic;
use std::path::Path;

macro_rules! irro_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

macro_rules! irro_long_version {
    () => {
        concat!("v", irro_version!(), " commit: ", env!("IRRO_COMMIT"))
    };
}

fn main() {
    log::set_logger(&IrroLogger).expect("Could not initialize logger.");
    log::set_max_level(log::LevelFilter::Trace);

    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
    }));

    let start_cmd = SubCommand::with_name("start")
        .about("Starts Irro server.")
        .arg(
            Arg::with_name("device")
                .long("device")
                .help("Arduino serial port device, for example /dev/ttyACM0")
                .takes_value(true)
                .required(true),
        );

    let update_cmd = SubCommand::with_name("update")
        .about("Updates this program")
        .long_about(
            "This sub-command downloads and atomically replaces irro-cli (this \
             program). Newest version of the program is downloaded.",
        )
        .arg(
            Arg::with_name("path")
                .long("path")
                .help("Target location, i.e. where the program will be [re]-placed.")
                .takes_value(true)
                .required(true),
        );

    let matches = App::new("irro-cli")
        .version(irro_version!())
        .long_version(irro_long_version!())
        .author("Martin Indra <martin.indra@mgn.cz>")
        .about("CLI & server for Irro onboard computer. See https://irro.cz/")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(start_cmd)
        .subcommand(update_cmd)
        .get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            let device = matches.value_of("device").unwrap();
            start_server(device);
        }
        ("update", Some(matches)) => {
            let path_str = matches.value_of("path").unwrap();
            let path = Path::new(path_str);
            update::update(path);
        }
        _ => panic!("Unrecognized command"),
    }
}

fn start_server(device: &str) {
    info!("Starting Irro {}...", irro_long_version!());

    match network::start_broadcasting() {
        Ok(socket) => socket,
        Err(error) => panic!("Error while starting broadcast loop: {}", error),
    }

    let sender = match Connection::init_from_device(device) {
        Ok(sender) => sender,
        Err(error) => panic!("Error while connecting to Arduino: {}", error),
    };

    if let Err(error) = api::run_http_server(sender) {
        panic!("Error while starting HTTP server: {}", error);
    }
}
