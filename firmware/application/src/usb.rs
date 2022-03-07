use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::interrupt::InterruptExt;
use embassy_nrf::{
    interrupt,
    peripherals::USBD,
    usb::{ClassSet1, ClassSet2, State, Usb, UsbBus, UsbSerial},
};
use futures::pin_mut;
use nrf_usbd::Usbd;
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};

pub enum Command {
    Start,
    Finish,
    Booted,
}

pub enum Response {
    Ok,
    Error(u8),
}

pub enum Data<'m> {
    Command(Command),
    Write(usize, &'m [u8]),
}

pub struct SerialUpdater<'a> {
    bus: &'a UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
    tx: &'a mut [u8],
    rx: &'a mut [u8],
}

impl<'a> SerialUpdater<'a> {
    pub fn new(
        bus: &'a mut UsbBusAllocator<Usbd<UsbBus<'a, USBD>>>,
        tx: &'a mut [u8],
        rx: &'a mut [u8],
    ) -> Self {
        Self { bus, tx, rx }
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

        let (mut reader, mut writer) = usb.as_ref().take_serial_0();
        async move { loop {} }
    }
}
/*


    info!("usb initialized!");
*/
