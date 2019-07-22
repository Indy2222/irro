//! This module implements client to Irro's onboard server REST API.
//! See API documentation at https://irro.cz/api.html

use reqwest::{self, Error};
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::net::IpAddr;
use std::path::Path;

pub struct Client {
    host: String,
    port: u16,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct MotorPowerRatio {
    left: f32,
    right: f32,
}

impl Client {
    /// Store Irro's IP to a file for later user. See `from_file()`.
    ///
    /// # Errors
    ///
    /// An error is returned if file couldn't be successfully written to.
    pub fn store_to_file(path: &Path, ip: IpAddr) -> Result<(), String> {
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(err) => return Err(format!("Failed to open {}: {}", path.display(), err)),
        };

        match writeln!(file, "{}", ip) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Failed to write to {}: {}", path.display(), err)),
        }
    }

    /// Parse server info (IP) from a file and return new Client.
    ///
    /// # Arguments
    ///
    /// * `path` - path to the file with client info.
    ///
    /// # Errors
    ///
    /// An error is returned if file couldn't be successfully read or if
    /// its contents can't be parsed.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(content) => Self::from_str_ip(content.trim()),
            Err(err) => Err(format!("Error while reading {}: {}", path.display(), err)),
        }
    }

    /// # Arguments
    ///
    /// * `ip` - string with IP address, e.g. "127.0.0.1"
    pub fn from_str_ip(ip: &str) -> Result<Self, String> {
        match ip.parse::<IpAddr>() {
            Ok(ip) => Ok(Self::from_ip(ip)),
            Err(err) => Err(format!("Error while parsing IP address: {}", err)),
        }
    }

    pub fn from_ip(ip: IpAddr) -> Self {
        Self::from_ip_and_port(ip, 8080)
    }

    fn from_ip_and_port(ip: IpAddr, port: u16) -> Self {
        Client {
            host: ip.to_string(),
            port,
            client: reqwest::Client::new(),
        }
    }

    /// Retrieve current LED on/off configuration from Irro.
    pub fn get_led(&self) -> Result<Vec<bool>, Error> {
        let url = self.url("/low/led");
        self.client.get(&url).send()?.json()
    }

    pub fn set_led(&self, led_id: u8, value: bool) -> Result<(), Error> {
        let url = self.url(&format!("/low/led/{}", led_id));
        self.client.put(&url).json(&value).send().map(|_| ())
    }

    /// Set power ratio to left and right motors.
    pub fn set_motor_power_ratio(&self, left: f32, right: f32) -> Result<(), Error> {
        if !left.is_finite() || !right.is_finite() || left.abs() > 1.0 || right.abs() > 1.0 {
            // Don't use is_infinite() as it doesn't include NaNs
            panic!("Motor power ratio must be a number between -1 and 1.");
        }

        let url = self.url("/low/motor/power/ratio");
        let payload = MotorPowerRatio { left, right };
        self.client
            .post(&url)
            .json(&payload)
            .send()?
            .error_for_status()
            .map(|_| ())
    }

    fn url(&self, endpoint: &str) -> String {
        format!("http://{}:{}{}", &self.host, self.port, endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use mockito::{mock, server_address};

    #[test]
    fn test_get_led() {
        let _m = mock("GET", "/low/led")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[true, false, true, false, false, false, false, false]")
            .create();

        let address = server_address();
        let client = Client::from_ip_and_port(address.ip(), address.port());

        let leds = client.get_led().unwrap();
        assert_eq!(
            leds,
            vec![true, false, true, false, false, false, false, false]
        );
    }

    #[test]
    fn test_set_led() {
        let mock = mock("PUT", "/low/led/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("null")
            .match_body("true")
            .create();

        let address = server_address();
        let client = Client::from_ip_and_port(address.ip(), address.port());
        client.set_led(1, true).unwrap();

        mock.assert();
    }
}
