//! This module implements various Irro tests.

use crate::api::Client;
use crate::network::discover_irro;
use log::{info, warn};
use std::thread;
use std::time::Duration;

macro_rules! validate {
    ($arg:tt) => {
        warn!("[VALIDATE] {}", $arg);
    };
}

/// Go through all available integration tests in sequence and prompt user to
/// validate that Irro behaves according to instructions. This is a long
/// running blocking function.
///
/// # Panics
///
/// This function panics if the Irro's API does respond with an error, doesn't
/// respond at all (i.e. on network errors) or if Irro's misbehavior is
/// detected automatically (i.e. when it gives an inconsistent API response).
pub fn integration() {
    info!(
        "Going to execute integration test sequence. Make sure that Irro \
         performs all operations which are logged with [VALIDATE] prefix."
    );

    // Users need time to read the above message and start focusing.
    thread::sleep(Duration::from_secs(10));

    info!("Going to look for Irro on local network...");
    let irro_ip = match discover_irro() {
        Err(error) => panic!("Could not find Irro: {}", error),
        Ok(ip) => ip,
    };
    info!("Irro successfully found at {}.", irro_ip);

    let client = Client::from_ip(irro_ip);

    validate!("Going to turn off all Irro LEDs.");
    client.set_led(0, false).unwrap();

    info!("Going to validate Irro's LED GET endpoint response.");
    let leds = client
        .get_led()
        .expect("Problem when retrieving LED states");
    assert_eq!(leds[0], false);

    thread::sleep(Duration::from_secs(5));
    validate!("Going to turn on onboard LED.");
    client.set_led(0, true).unwrap();

    info!("Going to validate Irro's LED GET endpoint response.");
    let leds = client
        .get_led()
        .expect("Problem when retrieving LED states");
    assert_eq!(leds[0], true);

    thread::sleep(Duration::from_secs(5));
    validate!("Going to turn off onboard LED.");
    client.set_led(0, false).unwrap();

    info!("Integration test suit is finished.");
}
