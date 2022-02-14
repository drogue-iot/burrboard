use core::future::Future;
use drogue_device::{Actor, Address, Inbox};

pub enum BoardButton {
    A,
    B,
}

pub struct Counter {
    presses: u32,
    button: BoardButton,
}

impl Counter {
    pub fn new(button: BoardButton) -> Self {
        Self { presses: 0, button }
    }
}

#[derive(Clone, Copy)]
pub enum CounterMessage {
    Increment,
    Read,
}

impl Actor for Counter {
    type Message<'m> = CounterMessage;
    type Response = Option<u32>;

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
                        CounterMessage::Increment => {
                            self.presses += 1;
                            info!("Presses: {}", self.presses);
                            None
                        }
                        CounterMessage::Read => Some(self.presses),
                    };
                    m.set_response(response);
                }
            }
        }
    }
}
