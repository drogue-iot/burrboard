use core::future::Future;
use drogue_device::{Actor, Address, Inbox};

use crate::gatt::BurrBoardService;

pub enum BoardButton {
    A,
    B,
}

pub struct Counter {
    presses: u32,
    button: BoardButton,

    board: &'static BurrBoardService,
}

impl Counter {
    pub fn new(button: BoardButton, board: &'static BurrBoardService) -> Self {
        Self {
            presses: 0,
            button,
            board,
        }
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
                            match self.button {
                                BoardButton::A => self.board.button_a_set(self.presses),
                                BoardButton::B => self.board.button_b_set(self.presses),
                            };
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
