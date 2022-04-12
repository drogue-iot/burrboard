use drogue_device::{actors::led::LedMessage, Address};

pub struct StatefulLed {
    led: Address<LedMessage>,
    state: bool,
}

impl StatefulLed {
    pub fn new(led: Address<LedMessage>, state: bool) -> Self {
        Self { led, state }
    }

    pub fn on(&mut self) -> Result<(), ()> {
        let _ = self.led.try_notify(LedMessage::On);
        self.state = true;
        Ok(())
    }

    pub fn off(&mut self) -> Result<(), ()> {
        let _ = self.led.try_notify(LedMessage::Off);
        self.state = false;
        Ok(())
    }

    pub fn is_on(&mut self) -> bool {
        self.state
    }
}

impl Clone for StatefulLed {
    fn clone(&self) -> Self {
        Self {
            led: self.led.clone(),
            state: self.state,
        }
    }
}
