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
    MarkBooted,
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
            let mut word_counter = 0;
            loop {
                if let Some(mut m) = inbox.next().await {
                    match m.message() {
                        DfuCommand::MarkBooted => {
                            self.updater.mark_booted(&mut self.flash).await.unwrap();
                        }
                        DfuCommand::Erase => {
                            info!("ERASE");
                        }
                        DfuCommand::Swap => {
                            info!("SWAP");
                            match self.updater.mark_update(&mut self.flash).await {
                                Ok(_) => {
                                    info!("Tagged swap magic");
                                }
                                Err(e) => {
                                    info!("Error marking swap updated");
                                }
                            }
                            self.updater.reset();
                        }
                        DfuCommand::Write(offset, data) => {
                            word_counter += data.len() / 4;
                            /*
                            for i in (0..data.len()).step_by(4) {
                                info!(
                                    "word: 0x{:02x}{:02x}{:02x}{:02x}",
                                    data[i + 3],
                                    data[i + 2],
                                    data[i + 1],
                                    data[i],
                                );
                                use embassy::time::{Duration, Timer};
                                Timer::after(Duration::from_millis(10)).await;
                            }*/
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

                            info!("We have written {} words", word_counter);
                        }
                    }
                }
            }
        }
    }
}
