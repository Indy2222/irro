//! Implementation of individual commands for Arduino. Please see documentation
//! of the commands at http://irro.mgn.cz/serial_protocol.html#serial-commands

pub mod led {
    //! Implementation of LED commands.

    use super::super::binary::Message;
    use std::sync::mpsc::Sender;

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

        /// Command Arduino turn on/off LEDs with this mask.
        ///
        /// # Arguments
        ///
        /// * `sender` - sender as returned from
        ///   `super::binary::Connection::new()`.
        pub fn send(&self, sender: &Sender<Message>) {
            // There is no interesting response.
            let (message, _) = Message::new(0x0000, vec![self.0]);
            sender.send(message).unwrap();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_send() {
            use super::super::tests::MessageTest;

            let test = MessageTest::new();
            let pr = LedMask::from_bools(vec![true, false, true]);
            pr.send(test.sender());
            test.test(0x0000, vec![160]);
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
        /// A number between -1.0 and 1.0 (inclusive).
        left: f32,
        /// A number between -1.0 and 1.0 (inclusive).
        right: f32,
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
            MotorPowerRatio { left, right }
        }

        /// Command Arduino to set motor power ratio to this.
        ///
        /// # Arguments
        ///
        /// * `sender` - sender as returned from `super::binary::Connection::new()`.
        pub fn send(&self, sender: &Sender<Message>) {
            let left = Self::float_to_int(self.left);
            let right = Self::float_to_int(self.right);
            let payload = vec![
                (left >> 8) as u8,
                (left & 0xff) as u8,
                (right >> 8) as u8,
                (right & 0xff) as u8,
            ];
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
        use super::*;

        #[test]
        fn test_send() {
            use super::super::tests::MessageTest;

            let test = MessageTest::new();
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

    pub struct MessageTest {
        here_sender: Sender<Message>,
        there_receiver: Receiver<Message>,
    }

    impl MessageTest {
        pub fn new() -> Self {
            let (here_sender, there_receiver) = mpsc::channel();
            MessageTest {
                here_sender,
                there_receiver,
            }
        }

        pub fn sender(&self) -> &Sender<Message> {
            &self.here_sender
        }

        pub fn test(self, expected_cmd: u16, expected_payload: Vec<u8>) {
            let there_receiver = self.there_receiver;
            let (there_sender, here_receiver) = mpsc::channel();

            thread::spawn(move || {
                let message = there_receiver
                    .recv_timeout(Duration::from_millis(100))
                    .unwrap();
                there_sender.send(message).unwrap();
            });

            let message: Message = here_receiver
                .recv_timeout(Duration::from_millis(100))
                .unwrap();

            struct MessageLocal {
                command: u16,
                payload: Vec<u8>,
                _sender: Sender<Vec<u8>>,
            }

            let message_exposed: MessageLocal = unsafe { std::mem::transmute(message) };
            assert_eq!(message_exposed.command, expected_cmd);
            assert_eq!(message_exposed.payload, expected_payload);
        }
    }
}
