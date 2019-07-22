use clap::{App, AppSettings, Arg, ArgMatches, Error, ErrorKind, SubCommand};
use libirroctl::api::Client;
use libirroctl::network;
use libirroctl::test;
use log::info;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
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

const IP_FILE: &str = "ip.txt";

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Stderr).unwrap();

    let test_cmd = SubCommand::with_name("test")
        .about("Execute integration test suite")
        .long_about(
            "This commands goes through an integration test sequence of Irro. \
             The command communicates with Irro's API. It validates \
             correctness the API responses and prompts the user to manually \
             inspect that Irro behaves according to logged description.",
        );
    let discover_cmd = SubCommand::with_name("discover")
        .about("Discover Irro using the broadcast UDP packets it is sending")
        .arg(
            Arg::with_name("store")
                .long("store")
                .help("Store Irro's IP to a file for a later use."),
        );
    let motor_cmd = SubCommand::with_name("motor")
        .about(
            "Set power ratio of left and right motors, id est values between \
             -1.0 and 1.0. Value -1.0 is full backward power and 1.0 is full \
             forward power.",
        )
        .arg(
            Arg::with_name("left")
                .short("l")
                .long("left")
                .help("Left motors power. A value between -1.0 and 1.0")
                .takes_value(true)
                .allow_hyphen_values(true)
                .required(true),
        )
        .arg(
            Arg::with_name("right")
                .short("r")
                .long("right")
                .help("Right motors power. A value between -1.0 and 1.0")
                .takes_value(true)
                .allow_hyphen_values(true)
                .required(true),
        );

    let matches = App::new("irro-cli")
        .version(irro_version!())
        .long_version(irro_long_version!())
        .author("Martin Indra <martin.indra@mgn.cz>")
        .about("Client CLI to Irro. See https://irro.cz/")
        .long_about(
            "Client CLI to Irro. See https://irro.cz/. \n\n\
             Commands which interact with Irro's API load file with Irro's IP \
             from current directory. The file must exist. See \
             irroctl discover --help",
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(test_cmd)
        .subcommand(discover_cmd)
        .subcommand(motor_cmd)
        .get_matches();

    match matches.subcommand() {
        ("test", _) => test::integration(),
        ("discover", Some(matches)) => {
            let store = matches.is_present("store");
            discover_irro(store);
        }
        ("motor", Some(matches)) => {
            let left = parse_motor_power_ratio(&matches, "left");
            let right = parse_motor_power_ratio(&matches, "right");
            let client = Client::from_file(Path::new(IP_FILE)).unwrap();
            client.set_motor_power_ratio(left, right).unwrap();
        }
        _ => panic!("Unrecognized command"),
    }
}

fn parse_motor_power_ratio(matches: &ArgMatches, arg_name: &str) -> f32 {
    let value: f32 = match matches.value_of(arg_name).unwrap().parse() {
        Ok(value) => value,
        Err(err) => Error::with_description(
            &format!(
                "Motor power ratio must be a number between -1 and 1: {}",
                err
            ),
            ErrorKind::InvalidValue,
        )
        .exit(),
    };
    if !value.is_finite() || value.abs() > 1.0 {
        // Don't use is_infinite() as it doesn't include NaNs
        Error::with_description(
            "Motor power ratio must be a number between -1 and 1.",
            ErrorKind::InvalidValue,
        )
        .exit();
    }
    value
}

fn discover_irro(store: bool) {
    let irro_ip = network::discover_irro().unwrap();
    print!("{}", irro_ip); // print to STDOUT for machine readability
    info!("Irro's IP: {}", irro_ip);

    if store {
        Client::store_to_file(Path::new(IP_FILE), irro_ip).unwrap();
    }
}
