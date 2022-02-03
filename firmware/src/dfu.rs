use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy_boot_nrf::FirmwareUpdater;
use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};

pub struct FirmwareManager<F: AsyncNorFlash + AsyncReadNorFlash> {
    flash: F,
    updater: FirmwareUpdater,
}

impl<F: AsyncNorFlash + AsyncReadNorFlash> FirmwareManager<F> {
    pub fn new(flash: F) -> Self {
        Self {
            flash,
            updater: FirmwareUpdater::new(),
        }
    }
}

pub enum DfuCommand<'m> {
    Erase,
    Write(u32, &'m [u8]),
    Swap,
}

impl<F: AsyncNorFlash + AsyncReadNorFlash> Actor for FirmwareManager<F> {
    type Message<'m>
    where
        Self: 'm,
    = DfuCommand<'m>;

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
        info!("Starting firmware manager");
        async move {
            // TODO: Mark ourselves as successfully booted
            // TODO: Make a more involved health check?
            //    self.updater.mark_booted(&mut self.flash).await.unwrap();
            loop {
                if let Some(mut m) = inbox.next().await {
                    match m.message() {
                        DfuCommand::Erase => {
                            info!("ERASE");
                        }
                        DfuCommand::Swap => {
                            /*
                            match self.updater.mark_update(&mut self.flash).await {
                                Ok(_) => {
                                    info!("Tagged swap magic");
                                }
                                Err(e) => {
                                    info!("Error marking swap updated");
                                }
                            }
                            self.updater.reset();*/
                            info!("SWAPPED");
                        }
                        DfuCommand::Write(offset, data) => {
                            match self
                                .updater
                                .write_firmware(*offset as usize, data, &mut self.flash)
                                .await
                            {
                                Ok(_) => {
                                    info!("Wrote {} bytes at {}", data.len(), offset);
                                }
                                Err(_) => {
                                    info!("Error writing firmware");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
