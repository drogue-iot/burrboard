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

impl<F: AsyncNorFlash + AsyncReadNorFlash> Actor for FirmwareManager<F> {
    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(&'m mut self, _: Address<Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            // Mark ourselves as successfully booted
            // TODO: Make a more involved health check?
            self.updater.mark_booted(&mut self.flash).await.unwrap();
            info!("Marked firmware as booted");
        }
    }
}
