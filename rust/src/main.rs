use clap::{App, AppSettings, Arg, SubCommand};
use irro::arduino::binary::Connection;
use irro::{api, logging::IrroLogger, network, update};
use log::{error, info};
use std::panic;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    log::set_logger(&IrroLogger).expect("Could not initialize logger.");
    log::set_max_level(log::LevelFilter::Trace);

    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
        println!("{}", panic_info);
    }));

    let start_cmd = SubCommand::with_name("start").about("Starts Irro server.");
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
        .version(VERSION)
        .author("Martin Indra <martin.indra@mgn.cz>")
        .about("CLI & server for Irro onboard computer.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(start_cmd)
        .subcommand(update_cmd)
        .get_matches();

    match matches.subcommand() {
        ("start", _) => start_server(),
        ("update", Some(matches)) => {
            let path_str = matches.value_of("path").unwrap();
            let path = Path::new(path_str);
            update::update(path);
        }
        _ => panic!("Unrecognized command"),
    }
}

fn start_server() {
    info!("Starting Irro version {}...", VERSION);

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
