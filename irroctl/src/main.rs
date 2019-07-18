use clap::{App, AppSettings, SubCommand};
use libirroctl::network;
use libirroctl::test;
use log::info;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

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
        .about("Discover Irro using the broadcast UDP packets it is sending");

    let matches = App::new("irro-cli")
        .version(irro_version!())
        .long_version(irro_long_version!())
        .author("Martin Indra <martin.indra@mgn.cz>")
        .about("Client CLI to Irro. See https://irro.cz/")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(test_cmd)
        .subcommand(discover_cmd)
        .get_matches();

    match matches.subcommand() {
        ("test", _) => test::integration(),
        ("discover", _) => discover_irro(),
        _ => panic!("Unrecognized command"),
    }
}

fn discover_irro() {
    let irro_ip = network::discover_irro().unwrap();
    print!("{}", irro_ip); // print to STDOUT for machine readability
    info!("Irro's IP: {}", irro_ip);
}
