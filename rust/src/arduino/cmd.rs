//! Implementation of individual commands for Arduino. Please see
//! [documentation of the commands](http://irro.mgn.cz/serial_protocol.html#serial-commands)

pub mod led {
    //! Implementation of [LED](http://irro.mgn.cz/hw.html#hw-leds) commands.

    use super::super::binary::Message;
    use log::debug;
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
            debug!("Going to send LED command to Arduino: {}", self.0);
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
