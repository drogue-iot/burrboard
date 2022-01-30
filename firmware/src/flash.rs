use crate::logger;
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};
use nrf_softdevice::{Flash, FlashError, Softdevice};

pub struct SharedFlash {
    flash: Flash,
}

impl SharedFlash {
    pub fn new(sd: &'static Softdevice) -> Self {
        let flash = Flash::take(sd);
        Self { flash }
    }
}

pub enum FlashOp<'m> {
    Write(u32, &'m [u8]),
    Erase(u32, u32),
    Read(u32, &'m mut [u8]),
}

impl Actor for SharedFlash {
    type Message<'m> = FlashOp<'m>;
    type Response = Option<Result<(), FlashError>>;

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
        Self: 'm,
    {
        async move {
            info!("Starting shared flash manager");
            loop {
                if let Some(mut m) = inbox.next().await {
                    let response = match m.message() {
                        FlashOp::Write(offset, buf) => self.flash.write(*offset, buf).await,
                        FlashOp::Erase(from, to) => self.flash.erase(*from, *to).await,
                        FlashOp::Read(offset, buf) => self.flash.read(*offset as usize, buf).await,
                    };
                    m.set_response(Some(response));
                }
            }
        }
    }
}

pub struct SharedFlashHandle(pub Address<SharedFlash>);

impl AsyncReadNorFlash for SharedFlashHandle {
    const READ_SIZE: usize = Flash::READ_SIZE;
    type Error = FlashError;

    type ReadFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn read<'a>(&'a mut self, address: usize, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            self.0
                .request(FlashOp::Read(address as u32, data))
                .unwrap()
                .await
                .unwrap()
        }
    }

    fn capacity(&self) -> usize {
        // TODO: Create message for it?
        256 * 4096
    }
}

impl AsyncNorFlash for SharedFlashHandle {
    const WRITE_SIZE: usize = Flash::WRITE_SIZE;
    const ERASE_SIZE: usize = Flash::ERASE_SIZE;

    type WriteFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            self.0
                .request(FlashOp::Write(offset, data))
                .unwrap()
                .await
                .unwrap()
        }
    }

    type EraseFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
        async move {
            self.0
                .request(FlashOp::Erase(from, to))
                .unwrap()
                .await
                .unwrap()
        }
    }
}
