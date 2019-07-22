//! Implementation of individual commands for Arduino. Please see
//! [documentation of the commands](https://irro.cz/serial_protocol.html#serial-commands)

pub mod led {
    //! Implementation of [LED](https://irro.cz/hw.html#hw-leds) commands.

    use super::super::binary::Message;
    use log::debug;
    use std::sync::mpsc::Sender;
    use std::time::Duration;

    /// Bit mask of which LEDs are turned on/off. LED 0 is mapped to the most
    /// significant bit.
    pub struct LedMask(u8);

    impl LedMask {
        /// Construct the bit mask from a vector of bools.
        ///
        /// # Arguments
        ///
        /// * `leds` - true means that the LED should be turned on. 0th element
        ///   corresponds the LED 0 and so on. Missing elements are interpreted
        ///   as false.
        ///
        /// # Panics
        ///
        /// This method panics if the number of bools is larger than number of
        /// bits in the mask.
        pub fn from_bools(leds: Vec<bool>) -> Self {
            if leds.len() > 8 {
                panic!("Number of bools must be smaller or equal to 8.");
            }

            let mut mask: u8 = 0;
            for (led, shift) in leds.iter().cloned().zip((0..=7).rev()) {
                mask |= (led as u8) << shift;
            }
            LedMask(mask)
        }

        /// Obtain current LED setup from Arduino.
        ///
        /// # Arguments
        ///
        /// * `sender` - message sender channel
        ///
        /// # Panics
        ///
        /// This method panics if command response is not retrieved from
        /// Arduino within 10 seconds or if the retrieved data are incorrect.
        pub fn read(sender: &Sender<Message>) -> Self {
            let (message, receiver) = Message::new(0x0001, vec![]);
            sender.send(message).unwrap();
            let masks = receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("LED mask not received from Arduino");

            if masks.len() != 1 {
                panic!("Expected 1 byte with LED mask, got {} bytes.", masks.len());
            }

            Self(masks[0])
        }

        /// Command Arduino turn on/off LEDs with this mask.
        ///
        /// # Arguments
        ///
        /// * `sender` - sender as returned from
        ///   `super::binary::Connection::new()`.
        pub fn send(&self, sender: &Sender<Message>) {
            debug!("Going to send LED command to Arduino: {}", self.0);
            // There is no interesting response.
            let (message, _) = Message::new(0x0000, vec![self.0]);
            sender.send(message).unwrap();
        }
    }

    /// Transform the bit mask into a Vec<bool>, where the LED 0 is at index 0
    /// and so on.
    impl Into<Vec<bool>> for LedMask {
        fn into(self) -> Vec<bool> {
            (0..8)
                .rev()
                .map(|bit| (self.0 & (1u8 << bit)) > 0)
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_send() {
            use super::super::tests::MessageTestBuilder;

            let test = MessageTestBuilder::new().start();
            let pr = LedMask::from_bools(vec![true, false, true]);
            pr.send(test.sender());
            test.test(0x0000, vec![160]);
        }

        #[test]
        fn test_read() {
            use super::super::tests::MessageTestBuilder;

            let test = MessageTestBuilder::new()
                .response(vec![0b0100_0001])
                .start();
            let leds: Vec<bool> = LedMask::read(test.sender()).into();
            test.test(0x0001, vec![]);

            assert_eq!(
                leds,
                vec![false, true, false, false, false, false, false, true]
            );
        }
    }
}

pub mod motor {
    //! Implementation of motor commands.

    use super::super::binary::Message;
    use std::i16;
    use std::sync::mpsc::Sender;

    const PREFIX: u16 = 0x0100;

    /// Power ratio of Irro's left and right motor.
    pub struct MotorPowerRatio {
        left: i16,
        right: i16,
    }

    impl MotorPowerRatio {
        /// Construct the struct from two floats between -1.0 (max backward
        /// power) and 1.0 (max forward power). Note that left and right motors
        /// are independent.
        ///
        /// # Panics
        ///
        /// This method panic if both numbers aren't between -1.0 and 1.0
        /// (inclusive).
        pub fn from_floats(left: f32, right: f32) -> Self {
            if !left.is_finite() || !right.is_finite() || left.abs() > 1.0 || right.abs() > 1.0 {
                // Don't use is_infinite() as it doesn't include NaNs
                panic!("Motor power ratio must be a number between -1 and 1.");
            }

            let left = Self::float_to_int(left);
            let right = Self::float_to_int(right);
            MotorPowerRatio { left, right }
        }

        /// Command Arduino to set motor power ratio to this.
        ///
        /// # Arguments
        ///
        /// * `sender` - sender as returned from `super::binary::Connection::new()`.
        pub fn send(&self, sender: &Sender<Message>) {
            let left_bytes = self.left.to_be_bytes();
            let right_bytes = self.right.to_be_bytes();

            let payload = vec![left_bytes[0], left_bytes[1], right_bytes[0], right_bytes[1]];
            // There is no interesting response.
            let (message, _) = Message::new(PREFIX, payload);
            sender.send(message).unwrap();
        }

        /// Convert an f32 value between -1.0 and 1.0 to full range i16.
        fn float_to_int(value: f32) -> i16 {
            if value.is_sign_positive() {
                (value * f32::from(i16::MAX)) as i16
            } else {
                (value.abs() * f32::from(i16::MIN)) as i16
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::tests::MessageTestBuilder;
        use super::MotorPowerRatio;

        #[test]
        fn test_send() {
            let test = MessageTestBuilder::new().start();
            let pr = MotorPowerRatio::from_floats(0.5, 0.25);
            pr.send(test.sender());
            test.test(0x0100, vec![63, 255, 31, 255]);
        }

        #[test]
        fn test_float_to_int() {
            let res: i16 = MotorPowerRatio::float_to_int(-1.0);
            assert_eq!(res, -32_768);
            let res: i16 = MotorPowerRatio::float_to_int(1.0);
            assert_eq!(res, 32_767);
            let res: i16 = MotorPowerRatio::float_to_int(0.0);
            assert_eq!(res, 0);
            let res: i16 = MotorPowerRatio::float_to_int(-0.15);
            assert_eq!(res, -4915);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::super::binary::Message;
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;
    use std::time::Duration;

    struct MessageLocal {
        command: u16,
        payload: Vec<u8>,
        sender: Sender<Vec<u8>>,
    }

    pub struct MessageTestBuilder {
        here_sender: Sender<Message>,
        here_receiver: Receiver<MessageLocal>,
        there_sender: Sender<MessageLocal>,
        there_receiver: Receiver<Message>,
        response: Vec<u8>,
    }

    pub struct MessageTest {
        here_sender: Sender<Message>,
        here_receiver: Receiver<MessageLocal>,
        // Keep this so the channel is not closed until this struct is dropped.
        _there_sender: Sender<MessageLocal>,
    }

    impl MessageTestBuilder {
        pub fn new() -> Self {
            let (here_sender, there_receiver) = mpsc::channel();
            let (there_sender, here_receiver) = mpsc::channel();

            MessageTestBuilder {
                here_sender,
                here_receiver,
                there_sender,
                there_receiver,
                response: Vec::new(),
            }
        }

        pub fn response(mut self, response: Vec<u8>) -> Self {
            self.response = response;
            self
        }

        pub fn start(self) -> MessageTest {
            let there_sender = self.there_sender.clone();
            let there_receiver = self.there_receiver;
            let response = self.response.clone();

            thread::spawn(move || {
                let message = there_receiver
                    .recv_timeout(Duration::from_millis(100))
                    .unwrap();
                let message_exposed: MessageLocal = unsafe { std::mem::transmute(message) };
                // The channel may be closed already if the command doesn't
                // read the response which is completely ok.
                message_exposed.sender.send(response).unwrap_or(());
                there_sender.send(message_exposed).unwrap();
            });

            MessageTest {
                here_sender: self.here_sender,
                here_receiver: self.here_receiver,
                _there_sender: self.there_sender,
            }
        }
    }

    impl MessageTest {
        pub fn sender(&self) -> &Sender<Message> {
            &self.here_sender
        }

        pub fn test(self, expected_cmd: u16, expected_payload: Vec<u8>) {
            let message_exposed: MessageLocal = self
                .here_receiver
                .recv_timeout(Duration::from_millis(100))
                .expect("shit");

            assert_eq!(message_exposed.command, expected_cmd);
            assert_eq!(message_exposed.payload, expected_payload);
        }
    }
}
