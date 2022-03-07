use core::future::Future;
use drogue_device::{
    actors::button::{ButtonEvent, FromButtonEvent},
    Actor, Address, Inbox,
};

pub struct Counter {
    presses: u32,
    pressed: bool,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            presses: 0,
            pressed: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CounterMessage {
    ButtonPressed,
    ButtonReleased,
    Read,
}

impl FromButtonEvent<CounterMessage> for Counter {
    fn from(event: ButtonEvent) -> Option<CounterMessage> {
        // Buttons are active low, invert logic
        Some(match event {
            ButtonEvent::Pressed => CounterMessage::ButtonReleased,
            ButtonEvent::Released => CounterMessage::ButtonPressed,
        })
    }
}

impl Actor for Counter {
    type Message<'m> = CounterMessage;
    type Response = Option<(bool, u32)>;

    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;

    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
    {
        async move {
            loop {
                if let Some(mut m) = inbox.next().await {
                    let response = match *m.message() {
                        CounterMessage::ButtonPressed => {
                            self.pressed = true;
                            self.presses += 1;
                            info!("Presses: {}", self.presses);
                            None
                        }
                        CounterMessage::ButtonReleased => {
                            self.pressed = false;
                            None
                        }
                        CounterMessage::Read => Some((self.pressed, self.presses)),
                    };
                    m.set_response(response);
                }
            }
        }
    }
}
