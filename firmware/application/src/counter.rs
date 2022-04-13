use core::future::Future;
use drogue_device::{Actor, Address, Inbox, Request};
use embassy_nrf::gpio::{AnyPin, Input};
use futures::future::{select, Either};
use futures::pin_mut;

pub struct Counter {
    button: Input<'static, AnyPin>,
    presses: u16,
    pressed: bool,
}

impl Counter {
    pub fn new(button: Input<'static, AnyPin>) -> Self {
        Self {
            button,
            presses: 0,
            pressed: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CounterRead;
pub type CounterResponse = (bool, u16);
pub type CounterRequest = Request<CounterRead, CounterResponse>;

impl Actor for Counter {
    type Message<'m> = CounterRequest;

    type OnMountFuture<'m, M>
    = impl Future<Output = ()> + 'm
    where
        Self: 'm,
        M: 'm + Inbox<Self::Message<'m>>;

    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self::Message<'m>>,
        mut inbox: M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self::Message<'m>> + 'm,
    {
        async move {
            loop {
                let mut check_presses = false;
                {
                    let next = inbox.next();
                    let button = self.button.wait_for_any_edge();
                    pin_mut!(next);
                    pin_mut!(button);
                    match select(next, &mut button).await {
                        Either::Left((r, _)) => {
                            r.reply((self.pressed, self.presses)).await;
                        }
                        Either::Right((_, _)) => {
                            check_presses = true;
                        }
                    };
                }

                if check_presses {
                    if self.button.is_low() {
                        self.pressed = true;
                        self.presses += 1;
                    } else {
                        self.pressed = false;
                    }
                }
            }
        }
    }
}
