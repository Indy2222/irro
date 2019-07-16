use log::info;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

const BROADCAST_ADDR: &str = "255.255.255.255:34254";

/// Start a new thread sending periodic broadcast messages (in IPv4 network).
pub fn start_broadcasting() -> std::io::Result<()> {
    info!("Starting broadcast loop...");

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_write_timeout(Some(Duration::from_secs(10)))?;

    thread::spawn(move || loop {
        // Sleep first, so the server has time to bootstrap.
        thread::sleep(Duration::from_secs(10));

        let result = socket.send_to("Hello, I am Irro!\n".as_bytes(), BROADCAST_ADDR);
        if let Err(error) = result {
            panic!("Error while sending broadcast UDP packet: {}", error);
        }

        info!("Broadcast sent to {}.", BROADCAST_ADDR);
    });

    Ok(())
}
