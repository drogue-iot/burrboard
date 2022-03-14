use burrboard_dfu::*;
use core::future::Future;
use core::str::FromStr;
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
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};

pub struct SerialUpdater<'a> {
    bus: &'a UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
    tx: &'a mut [u8],
    rx: &'a mut [u8],
    version: &'a str,
    dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
}

impl<'a> SerialUpdater<'a> {
    pub fn new(
        bus: &'a mut UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
        tx: &'a mut [u8],
        rx: &'a mut [u8],
        dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    ) -> Self {
        Self {
            bus,
            tx,
            rx,
            dfu,
            version: crate::FIRMWARE_REVISION.unwrap_or(crate::FIRMWARE_VERSION),
        }
    }
}

impl<'a> Actor for SerialUpdater<'a> {
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where
        Self: 'm,
        M: 'm + Inbox<Self>;

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

            info!("Starting serial updater");
            loop {
                info!("Awaiting next frame");
                let response: Option<Frame> = if let Ok(frame) = Frame::decode(&mut reader).await {
                    info!("Received frame: {:?}", frame);
                    match frame {
                        Frame::Data(data) => Some(Frame::Response(
                            if let Ok(f) = self.dfu.request(DfuCommand::WriteBlock(&data)) {
                                if let DfuResponse::Ok = f.await {
                                    Response::Ok
                                } else {
                                    Response::Err(Error::Flash)
                                }
                            } else {
                                Response::Err(Error::Actor)
                            },
                        )),
                        Frame::Command(command) => Some(Frame::Response(match command {
                            Command::Start => {
                                if let Ok(f) = self.dfu.request(DfuCommand::Start) {
                                    if let DfuResponse::Ok = f.await {
                                        Response::Ok
                                    } else {
                                        Response::Err(Error::Flash)
                                    }
                                } else {
                                    Response::Err(Error::Actor)
                                }
                            }
                            Command::Finish => {
                                if let Ok(_) = self.dfu.notify(DfuCommand::Finish) {
                                    Response::Ok
                                } else {
                                    Response::Err(Error::Flash)
                                }
                            }
                            Command::Booted => {
                                if let Ok(_) = self.dfu.notify(DfuCommand::Booted) {
                                    Response::Ok
                                } else {
                                    Response::Err(Error::Flash)
                                }
                            }
                            Command::Version => {
                                info!("Command version response: {}", self.version);
                                loop {}
                                Response::OkVersion(
                                    heapless::String::from_str(self.version).unwrap(),
                                )
                            }
                        })),
                        Frame::Response(_) => None,
                    }
                } else {
                    info!("Error receiving frame");
                    None
                };

                if let Some(f) = response {
                    info!("Sending response {:?}", f);
                    if let Err(_) = f.encode(&mut writer).await {
                        warn!("Error sending command response");
                    }
                }
            }
        }
    }
}
