//! This module implements client to Irro's onboard server REST API.
//! See API documentation at https://irro.cz/api.html

use reqwest::{self, Error};
use std::net::IpAddr;

pub struct Client {
    host: String,
    port: u16,
    client: reqwest::Client,
}

impl Client {
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
