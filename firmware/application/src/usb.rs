use core::future::Future;
use drogue_device::{
    actors::{
        dfu::{DfuCommand, DfuResponse, FirmwareManager},
        flash::SharedFlashHandle,
    },
    Actor, Address, Inbox,
};
use embassy::interrupt::InterruptExt;
use embassy::io::AsyncBufReadExt;
use embassy::io::AsyncWriteExt;
use embassy_nrf::{
    interrupt,
    peripherals::USBD,
    usb::{ClassSet1, ClassSet2, State, Usb, UsbBus, UsbSerial},
};
use futures::pin_mut;
use nrf_softdevice::Flash;
use nrf_usbd::Usbd;
use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Command {
    Start,
    Write,
    Finish,
    Booted,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Error {
    Protocol,
    Actor,
    Flash,
}

pub struct SerialUpdater<'a> {
    bus: &'a UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
    tx: &'a mut [u8],
    rx: &'a mut [u8],
    dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
}

impl<'a> SerialUpdater<'a> {
    pub fn new(
        bus: &'a mut UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
        tx: &'a mut [u8],
        rx: &'a mut [u8],
        dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    ) -> Self {
        Self { bus, tx, rx, dfu }
    }
}

impl<'a> Actor for SerialUpdater<'a> {
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
            let serial = UsbSerial::new(self.bus, self.rx, self.tx);
            let device = UsbDeviceBuilder::new(self.bus, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Red Hat")
                .product("Serial port")
                .serial_number("dr0gue")
                .device_class(0x02)
                .build();

            let irq = interrupt::take!(USBD);
            irq.set_priority(interrupt::Priority::P3);

            let mut state = State::new();
            let usb = unsafe { Usb::new(&mut state, device, serial, irq) };

            pin_mut!(usb);

            let mut buf: [u8; 128] = [0; 128];
            let (mut reader, mut writer) = usb.as_ref().take_serial_0();

            loop {
                let c = reader.read_byte().await.unwrap();
                let command: Command = from_bytes(&[c]).unwrap();
                let response = match command {
                    Command::Start => {
                        if let Ok(f) = self.dfu.request(DfuCommand::Start) {
                            if let DfuResponse::Ok = f.await {
                                Ok(())
                            } else {
                                Err(Error::Flash)
                            }
                        } else {
                            Err(Error::Actor)
                        }
                    }
                    Command::Write => {
                        if let Ok(_) = reader.read_exact(&mut buf[..4]).await {
                            let mut length: usize =
                                u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

                            let mut result = Ok(());
                            while length > 0 {
                                let to_copy: usize = core::cmp::min(length as usize, 128);

                                if let Ok(_) = reader.read_exact(&mut buf[..to_copy]).await {
                                    if let Ok(f) =
                                        self.dfu.request(DfuCommand::WriteBlock(&buf[..to_copy]))
                                    {
                                        if let DfuResponse::Ok = f.await {
                                        } else {
                                            result = Err(Error::Flash);
                                            break;
                                        }
                                    } else {
                                        result = Err(Error::Actor);
                                        break;
                                    }
                                } else {
                                    result = Err(Error::Protocol);
                                    break;
                                }
                                // Write to flash
                                length -= to_copy;
                            }
                            result
                        } else {
                            Err(Error::Protocol)
                        }
                    }
                    Command::Finish => {
                        if let Ok(_) = self.dfu.notify(DfuCommand::Finish) {
                            Ok(())
                        } else {
                            Err(Error::Flash)
                        }
                    }
                    Command::Booted => {
                        if let Ok(_) = self.dfu.notify(DfuCommand::Booted) {
                            Ok(())
                        } else {
                            Err(Error::Flash)
                        }
                    }
                };

                if let Ok(data) = to_slice(&response, &mut buf) {
                    if let Err(_) = writer.write_all(&data).await {
                        warn!("Error sending command response");
                    }
                } else {
                    warn!("Error serializing command response");
                }
            }
        }
    }
}
/*


    info!("usb initialized!");
*/
