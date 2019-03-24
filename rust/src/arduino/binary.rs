//! This module implements asynchronous binary communication with Irro's
//! Arduino over a serial port. The communication is handled in its own
//! thread.
//!
//! See [protocol documentation](http://irro.mgn.cz/serial_protocol.html).

use serial::prelude::*;
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub const ARDUINO_BUFFER_SIZE: usize = 64;
// see: https://www.arduino.cc/en/Serial/Begin
const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud115200,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

/// This struct represent an individual command which could be send to Arduino.
pub struct Message {
    command: u16,
    payload: Vec<u8>,
    /// Arduino response (possibly an empty Vec) will be send via this Sender.
    sender: Sender<Vec<u8>>,
}

impl Message {
    /// Construct a new message from command number and command payload. The
    /// method returns a tuple with the newly constructed message and a
    /// `Receiver` via which a command response will be delivered.
    ///
    /// Command response is send via the channel to the `Receiver` once it is
    /// obtained from the Arduino. No other data is ever send via the channel.
    ///
    /// # Arguments
    ///
    /// * `command` - identifier of the command to be send.
    ///
    /// * `payload` - command payload. Note that the total message size cannot
    ///    be larger than `ARDUINO_BUFFER_SIZE`. Note that message size
    ///    includes 2 bytes for command number and 2 bytes payload length.
    ///
    /// # Panics
    ///
    /// The method panics if the `payload` wouldn't fit the Arduino buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use irro::arduino::binary::Message;
    /// // Message which turns on LED 0.
    /// let (message, receiver) = Message::new(0, vec![128]);
    /// ```
    pub fn new(command: u16, payload: Vec<u8>) -> (Self, Receiver<Vec<u8>>) {
        // This size includes the two bytes for command and two bytes for
        // payload length.
        let bytes_len = 4 + payload.len();
        if bytes_len > ARDUINO_BUFFER_SIZE {
            panic!(
                "Overall message size of {} bytes is larger than Arduino buffer.",
                bytes_len
            );
        }

        let (sender, receiver) = mpsc::channel();
        let message = Message {
            command,
            payload,
            sender,
        };
        (message, receiver)
    }

    /// Return message size including headers.
    fn len(&self) -> usize {
        self.payload.len() + 4
    }

    fn destructure(self) -> (u16, Vec<u8>, Sender<Vec<u8>>) {
        (self.command, self.payload, self.sender)
    }
}

/// This struct corresponds to a command sent to Arduino for which a reply
/// hasn't been processed yet
struct InAir {
    /// Size of the data send to Arduino with this command. This is used to
    /// avoid Arduino buffer overflow.
    len: usize,
    /// This sender should be used to deliver command response.
    sender: Sender<Vec<u8>>,
}

impl InAir {
    fn new(len: usize, sender: Sender<Vec<u8>>) -> Self {
        InAir { len, sender }
    }

    fn len(&self) -> usize {
        self.len
    }

    /// Send response of the command to the client.
    fn respond(self, data: Vec<u8>) {
        // Errors aren't handled because data receiving isn't enforced.
        self.sender.send(data).unwrap_or(());
    }
}

/// An asynchronous connecting to the Arduino.
pub struct Connection {
    /// Receiver used to get commands to be send to the Arduino.
    receiver: Receiver<Message>,
    /// Serial port writer.
    port: serial::SystemPort,
    /// A queue of not yet responded messages. New messages are appended to
    /// front and resolved messages are popped from back.
    in_air: VecDeque<InAir>,
    /// This is not-None in case that a message couldn't be sent right away due
    /// to full Arduino buffer.
    waiting_message: Option<Message>,
}

impl Connection {
    /// Initiate an asynchronous "connection" to the Arduino. This methods
    /// creates a new thread and returns `Sender` through which messages can be
    /// send to the Arduino.
    ///
    /// It is supposed that there is at most one running Connection at any
    /// given moment and that no other program interact with the Arduino.
    ///
    /// # Arguments
    ///
    /// * `device` - serial port device, for example ```"/dev/ttyACM1"```.
    pub fn initiate(device: &str) -> Result<Sender<Message>, serial::Error> {
        let (sender, receiver) = mpsc::channel();

        let mut port = serial::open(device)?;
        port.configure(&SETTINGS).unwrap();
        port.set_timeout(Duration::from_millis(1000)).unwrap();

        thread::spawn(move || {
            let connection = Connection {
                receiver,
                port,
                in_air: VecDeque::new(),
                waiting_message: None,
            };
            connection.start();
        });
        Ok(sender)
    }

    /// Start the communication loop which sends messages to Arduino and
    /// retrieve and delivers response. This method never returns.
    fn start(mut self) -> ! {
        loop {
            self.process_responses();
            self = self.process_messages();
        }
    }

    fn process_messages(mut self) -> Self {
        loop {
            let message: Option<Message> = if self.waiting_message.is_some() {
                let message = self.waiting_message;
                self.waiting_message = None;
                message
            } else {
                match self.receiver.recv_timeout(Duration::from_secs(1)) {
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => {
                        panic!("All data producers have disconnected.");
                    }
                    Ok(message) => Some(message),
                }
            };

            if message.is_none() {
                break self;
            }

            let message = message.unwrap();
            assert!(message.len() <= ARDUINO_BUFFER_SIZE);

            let bytes_in_air: usize = self.in_air.iter().map(|ia| ia.len()).sum();
            let remaining = ARDUINO_BUFFER_SIZE - bytes_in_air;

            if remaining < message.len() {
                self.waiting_message = Some(message);
                break self;
            }

            let (command, payload, sender) = message.destructure();
            let payload_len = payload.len();
            assert!(payload_len < 256 * 256);
            let header: [u8; 4] = [
                (command >> 8) as u8,
                (command & 0xff) as u8,
                (payload_len >> 8) as u8,
                (payload_len & 0xff) as u8,
            ];
            // TODO nicer errors
            self.port.write_all(&header).unwrap();
            self.port.write_all(&payload[..]).unwrap();
            self.in_air.push_front(InAir::new(payload_len + 4, sender));
        }
    }

    /// Read and process all available responses from the Arduino. Responses
    /// are immediately send to clients via each message channel. The function
    /// returns number of processed responses.
    fn process_responses(&mut self) {
        let mut buf = Vec::new();
        if let Err(err) = self.port.read_to_end(&mut buf) {
            if err.kind() != ErrorKind::TimedOut {
                panic!("Error while reading data from Arduino: {}", err);
            }
        }

        let mut offset = 0;
        while offset < buf.len() {
            let payload_len: usize = ((buf[offset] as usize) << 8) | (buf[offset + 1] as usize);
            let in_air = self
                .in_air
                .pop_back()
                .expect("Got an unexpected response from Arduino.");
            offset += 2;
            in_air.respond(buf[offset..(offset + payload_len)].to_vec());
            offset += payload_len;
        }
    }
}
