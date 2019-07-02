//! This module implements asynchronous binary communication with Irro's
//! Arduino over a serial port. The communication is handled in its own
//! thread.
//!
//! See [protocol documentation](http://irro.mgn.cz/serial_protocol.html).

use serialport::{self, DataBits, FlowControl, Parity, SerialPort, SerialPortSettings, StopBits};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Size of Arduino serial port buffer. See [Arduino
/// Docs](https://www.arduino.cc/en/Reference/SoftwareSerial).
pub const ARDUINO_BUFFER_SIZE: usize = 64;
// see: https://www.arduino.cc/en/Serial/Begin
const SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: 115_200,
    data_bits: DataBits::Eight,
    parity: Parity::None,
    stop_bits: StopBits::One,
    flow_control: FlowControl::None,
    timeout: Duration::from_millis(1000),
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

/// A de-queue of not yet responded messages.
struct InAirQueue {
    /// A queue of not yet responded messages. New messages are appended to
    /// front and resolved messages are popped from back.
    queue: VecDeque<InAir>,
    /// Total (i.e. sum) size of all not responded messages. This bookkeeping
    /// is necessary to avoid Arduino serial buffer overflow.
    size: usize,
}

impl InAirQueue {
    /// Create an empty queue.
    fn new() -> Self {
        InAirQueue {
            queue: VecDeque::new(),
            size: 0,
        }
    }

    fn push(&mut self, payload_len: usize, sender: Sender<Vec<u8>>) {
        self.queue.push_back(InAir::new(payload_len, sender));
        self.size += payload_len;
    }

    fn respond(&mut self, response: Vec<u8>) {
        let in_air = self
            .queue
            .pop_front()
            .expect("There is no message waiting for a response.");
        self.size -= in_air.len();
        in_air.respond(response);
    }

    fn size(&self) -> usize {
        self.size
    }
}

/// An asynchronous connecting to the Arduino.
pub struct Connection {
    /// Receiver used to get commands to be send to the Arduino.
    receiver: Receiver<Message>,
    /// Serial port writer.
    port: Box<SerialPort>,
    in_air_queue: InAirQueue,
    /// Buffer of messages waiting to be send.
    waiting_messages: VecDeque<Message>,
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
    pub fn init_from_device(device: &str) -> Result<Sender<Message>, serialport::Error> {
        let port = serialport::open_with_settings(device, &SETTINGS)?;
        Ok(Self::initiate(port))
    }

    fn initiate(port: Box<SerialPort>) -> Sender<Message> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let connection = Connection {
                receiver,
                port,
                in_air_queue: InAirQueue::new(),
                waiting_messages: VecDeque::new(),
            };
            connection.start();
        });
        sender
    }

    /// Start the communication loop which sends messages to Arduino and
    /// retrieve and delivers response. This method never returns.
    fn start(mut self) -> ! {
        loop {
            self.process_responses();
            self.process_messages();
        }
    }

    fn process_messages(&mut self) {
        self.waiting_messages.extend(self.receiver.try_iter());

        let mut remaining = ARDUINO_BUFFER_SIZE - self.in_air_queue.size();
        let mut to_send = Vec::new();

        while let Some(message) = self.waiting_messages.pop_front() {
            assert!(message.len() <= ARDUINO_BUFFER_SIZE);

            if remaining < message.len() {
                self.waiting_messages.push_front(message);
                break;
            }

            remaining -= message.len();

            let (command, payload, sender) = message.destructure();
            let payload_len = payload.len();
            assert!(payload_len < 256 * 256);

            self.in_air_queue.push(payload_len, sender);

            to_send.push((command >> 8) as u8);
            to_send.push((command & 0xff) as u8);
            to_send.push((payload_len >> 8) as u8);
            to_send.push((payload_len & 0xff) as u8);
            to_send.extend(payload);
        }

        if !to_send.is_empty() {
            if let Err(err) = self.port.write_all(&to_send[..]) {
                panic!("Error while writing data to Arduino: {}", err);
            }
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
            offset += 2;
            // Unfortunately there is no trivial way how to split the Vec into
            // multiple owned Vec-s. See
            // https://github.com/rust-lang/rust/issues/40708
            self.in_air_queue
                .respond(buf[offset..(offset + payload_len)].to_vec());
            offset += payload_len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_air_queue() {
        let mut queue = InAirQueue::new();
        assert_eq!(queue.size(), 0);

        let (sender, receiver_a) = mpsc::channel();
        queue.push(2, sender);
        assert_eq!(queue.size(), 2);

        let (sender, receiver_b) = mpsc::channel();
        queue.push(3, sender);
        assert_eq!(queue.size(), 5);

        let (sender, receiver_c) = mpsc::channel();
        queue.push(4, sender);
        assert_eq!(queue.size(), 9);

        queue.respond(vec![1, 2]);
        assert_eq!(queue.size(), 7);
        let response = receiver_a.recv().unwrap();
        assert_eq!(response, vec![1, 2]);

        queue.respond(vec![3, 4]);
        assert_eq!(queue.size(), 4);
        let response = receiver_b.recv().unwrap();
        assert_eq!(response, vec![3, 4]);

        queue.respond(vec![5, 6]);
        assert_eq!(queue.size(), 0);
        let response = receiver_c.recv().unwrap();
        assert_eq!(response, vec![5, 6]);
    }

    #[test]
    fn test_connection() {
        use serialport::posix::TTYPort;

        let (mut master, slave) = TTYPort::pair().unwrap();

        let sender = Connection::initiate(Box::new(slave));
        let (message_a, receiver_a) = Message::new(23, vec![6, 2, 1]);
        let (message_b, receiver_b) = Message::new(25, vec![10, 20, 30, 40]);

        sender.send(message_a).unwrap();
        sender.send(message_b).unwrap();

        let mut buf = [0; 7];
        master.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0u8, 23, 0, 3, 6, 2, 1]);

        let mut buf = [0; 8];
        master.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0u8, 25, 0, 4, 10, 20, 30, 40]);

        master.write(&[0u8, 5, 10, 9, 8, 7, 6]).unwrap();
        master.write(&[0u8, 2, 255, 128]).unwrap();

        let recv = receiver_b.recv().unwrap();
        assert_eq!(recv, vec![255, 128]);

        let recv = receiver_a.recv().unwrap();
        assert_eq!(recv, vec![10, 9, 8, 7, 6]);
    }
}
