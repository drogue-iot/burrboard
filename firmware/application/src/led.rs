use drogue_device::{actors::led::LedMessage, Actor, ActorError, Address};

pub struct StatefulLed<L>
where
    L: Actor<Message<'static> = LedMessage> + 'static,
{
    led: Address<L>,
    state: bool,
}

impl<L> StatefulLed<L>
where
    L: Actor<Message<'static> = LedMessage> + 'static,
{
    pub fn new(led: Address<L>, state: bool) -> Self {
        Self { led, state }
    }
    pub fn on(&mut self) -> Result<(), ActorError> {
        self.led.notify(LedMessage::On)?;
        self.state = true;
        Ok(())
    }

    pub fn off(&mut self) -> Result<(), ActorError> {
        self.led.notify(LedMessage::Off)?;
        self.state = false;
        Ok(())
    }

    pub fn is_on(&mut self) -> bool {
        self.state
    }
}

impl<L> Copy for StatefulLed<L> where L: Actor<Message<'static> = LedMessage> + 'static {}

impl<L> Clone for StatefulLed<L>
where
    L: Actor<Message<'static> = LedMessage> + 'static,
{
    fn clone(&self) -> Self {
        Self {
            led: self.led.clone(),
            state: self.state,
        }
    }
}
