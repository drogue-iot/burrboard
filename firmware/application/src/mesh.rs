use core::future::Future;
use drogue_device::drivers::ble::mesh::composition::{
    CompanyIdentifier, Composition, ElementDescriptor, ElementsHandler, Features, Location,
    ProductIdentifier, VersionIdentifier,
};
use drogue_device::drivers::ble::mesh::driver::elements::ElementContext;
use drogue_device::drivers::ble::mesh::driver::DeviceError;
use drogue_device::drivers::ble::mesh::model::generic::GENERIC_BATTERY_SERVER;
use drogue_device::drivers::ble::mesh::model::generic::GENERIC_ONOFF_SERVER;
use drogue_device::drivers::ble::mesh::pdu::access::AccessMessage;
use drogue_device::drivers::ble::mesh::provisioning::{
    Algorithms, Capabilities, InputOOBActions, OOBSize, OutputOOBActions, PublicKeyType,
    StaticOOBType,
};
use drogue_device::drivers::ble::mesh::storage::FlashStorage;
use drogue_device::ActorContext;
use drogue_device::{actors::ble::mesh::MeshNode, drivers::ble::mesh::model::Message};
use drogue_device::{actors::flash::SharedFlashHandle, drivers::ble::mesh::InsufficientBuffer};
use drogue_device::{
    drivers::ble::mesh::bearer::nrf52::{SoftdeviceAdvertisingBearer, SoftdeviceRng},
    Actor, Address, Inbox,
};
use embassy::executor::Spawner;
use embassy::time::{Duration, Ticker};
use futures::future::{select, Either};
use futures::{pin_mut, StreamExt};
use nrf_softdevice::{Flash, Softdevice};

use crate::{
    accel::{AccelValues, Read as AccelRead},
    analog::Read as AnalogRead,
    board::*,
    counter::*,
};

const COMPANY_IDENTIFIER: CompanyIdentifier = CompanyIdentifier(0x0003);
const PRODUCT_IDENTIFIER: ProductIdentifier = ProductIdentifier(0x0001);
const VERSION_IDENTIFIER: VersionIdentifier = VersionIdentifier(0x0001);
const FEATURES: Features = Features {
    relay: true,
    proxy: false,
    friend: false,
    low_power: false,
};

#[allow(unused)]
pub struct BurrBoardElementsHandler {
    composition: Composition,
    leds: Leds,
}

impl BurrBoardElementsHandler {
    pub fn new(leds: Leds) -> Self {
        let mut composition = Composition::new(
            COMPANY_IDENTIFIER,
            PRODUCT_IDENTIFIER,
            VERSION_IDENTIFIER,
            FEATURES,
        );
        composition
            .add_element(ElementDescriptor::new(Location(0x0001)).add_model(GENERIC_ONOFF_SERVER))
            .ok();
        composition
            .add_element(ElementDescriptor::new(Location(0x0002)).add_model(GENERIC_ONOFF_SERVER))
            .ok();
        composition
            .add_element(ElementDescriptor::new(Location(0x0003)).add_model(GENERIC_ONOFF_SERVER))
            .ok();
        composition
            .add_element(ElementDescriptor::new(Location(0x0004)).add_model(GENERIC_ONOFF_SERVER))
            .ok();
        composition
            .add_element(ElementDescriptor::new(Location(0x0005)).add_model(GENERIC_BATTERY_SERVER))
            .ok();
        composition
            .add_element(
                ElementDescriptor::new(Location(0x0006)).add_model(model::BURRBOARD_CLIENT),
            )
            .ok();
        Self { leds, composition }
    }
}

impl ElementsHandler for BurrBoardElementsHandler {
    fn composition(&self) -> &Composition {
        &self.composition
    }

    fn connect<C: ElementContext>(&self, _ctx: &C) {
        info!("CONNECT");
    }

    type DispatchFuture<'m>
    where
        Self: 'm,
    = impl Future<Output = Result<(), DeviceError>> + 'm;

    fn dispatch(&self, element: u8, _message: AccessMessage) -> Self::DispatchFuture<'_> {
        async move {
            if element == 0x0001 {
                info!("Element 1");
            } else if element == 0x0002 {
                info!("Element 2");
            } else if element == 0x0003 {
                info!("Element 3");
            } else if element == 0x0004 {
                info!("Element 4");
            } else if element == 0x0005 {
            }
            Ok(())
        }
    }
}

pub struct MeshApp {
    node: ActorContext<
        MeshNode<
            BurrBoardElementsHandler,
            SoftdeviceAdvertisingBearer,
            FlashStorage<SharedFlashHandle<Flash>>,
            SoftdeviceRng,
        >,
    >,
    publisher: ActorContext<BoardStatePublisher>,
}

impl MeshApp {
    pub fn enable() -> Self {
        Self {
            node: ActorContext::new(),
            publisher: ActorContext::new(),
        }
    }

    pub fn mount(&'static self, s: Spawner, sd: &'static Softdevice, p: &BoardPeripherals) {
        extern "C" {
            static __storage: u8;
        }

        let storage: FlashStorage<SharedFlashHandle<Flash>> =
            FlashStorage::new(unsafe { &__storage as *const u8 as usize }, p.flash.into());

        let bearer = SoftdeviceAdvertisingBearer::new(sd);
        let rng = SoftdeviceRng::new(sd);

        let capabilities = Capabilities {
            number_of_elements: 6,
            algorithms: Algorithms::default(),
            public_key_type: PublicKeyType::default(),
            static_oob_type: StaticOOBType::default(),
            output_oob_size: OOBSize::MaximumSize(6),
            output_oob_action: OutputOOBActions::default(),
            input_oob_size: OOBSize::MaximumSize(6),
            input_oob_action: InputOOBActions::default(),
        };

        let elements = BurrBoardElementsHandler::new(p.leds.clone());

        self.node.mount(
            s,
            MeshNode::new(elements, capabilities, bearer, storage, rng),
        );

        self.publisher.mount(
            s,
            BoardStatePublisher::new(Duration::from_secs(1), p.clone()),
        );
        /*
        let mesh_node =
        //let mesh_node = MeshNode::new(capabilities, bearer, storage, rng).force_reset();
        */
    }
}

pub struct BoardStatePublisher {
    interval: Duration,
    board: BoardPeripherals,
    //  ctx: Option<AppElementContext<model::BurrBoardClient>>,
}

impl BoardStatePublisher {
    pub fn new(interval: Duration, board: BoardPeripherals) -> Self {
        Self {
            interval,
            //        ctx: None,
            board,
        }
    }
}

pub enum PublisherMessage {
    //Connect(AppElementContext<model::BurrBoardClient>),
}

impl Actor for BoardStatePublisher {
    type Message<'m> = PublisherMessage;
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
            let mut ticker = Ticker::every(self.interval);
            loop {
                let next = inbox.next();
                let tick = ticker.next();

                pin_mut!(next);
                pin_mut!(tick);

                match select(next, tick).await {
                    Either::Left((m, _)) => {
                        if let Some(m) = m {
                            /*
                                match m.message() {
                                    PublisherMessage::Connect(ctx) => {
                                        info!("connected to mesh {}", ctx.address());
                                        //self.ctx.replace(ctx.clone());
                                    }
                                }
                            */
                        }
                    }
                    Either::Right((_, _)) => {
                        let accel = if let Some(accel) = self.board.accel {
                            accel.request(AccelRead).unwrap().await.unwrap()
                        } else {
                            AccelValues { x: 0, y: 0, z: 0 }
                        };
                        let analog = self.board.analog.request(AnalogRead).unwrap().await;
                        let button_a = self
                            .board
                            .counter_a
                            .request(CounterMessage::Read)
                            .unwrap()
                            .await
                            .unwrap();
                        let button_b = self
                            .board
                            .counter_b
                            .request(CounterMessage::Read)
                            .unwrap()
                            .await
                            .unwrap();

                        let red_led = self.board.leds.red.is_on();
                        let green_led = self.board.leds.green.is_on();
                        let blue_led = self.board.leds.blue.is_on();
                        let yellow_led = self.board.leds.yellow.is_on();

                        let data = model::BurrBoardState {
                            temperature: analog.temperature,
                            brightness: analog.brightness,
                            accel: [accel.x, accel.y, accel.z],
                            battery: analog.battery,
                            button_a,
                            button_b,
                            red_led,
                            green_led,
                            blue_led,
                            yellow_led,
                        };

                        //                        if let Some(ctx) = &self.ctx {
                        //                       } else {
                        info!("Read sensor values: {:?}", data);
                        //                      }
                    }
                }
            }
        }
    }
}

mod model {
    use super::*;
    use drogue_device::drivers::ble::mesh::model::ModelIdentifier;
    use drogue_device::drivers::ble::mesh::pdu::access::Opcode;
    use drogue_device::opcode;
    use heapless::Vec;

    pub const BURRBOARD_SERVER: ModelIdentifier =
        ModelIdentifier::Vendor(COMPANY_IDENTIFIER, 0x0001);
    pub const BURRBOARD_CLIENT: ModelIdentifier =
        ModelIdentifier::Vendor(COMPANY_IDENTIFIER, 0x0002);

    #[derive(defmt::Format, Debug)]
    pub struct BurrBoardState {
        pub temperature: i16,
        pub brightness: u16,
        pub accel: [i16; 3],
        pub battery: u8,
        pub button_a: u32,
        pub button_b: u32,
        pub red_led: bool,
        pub green_led: bool,
        pub blue_led: bool,
        pub yellow_led: bool,
    }

    pub enum BurrBoardMessage {
        State(BurrBoardState),
    }

    pub struct BurrBoardModel;

    impl Message for BurrBoardMessage {
        fn opcode(&self) -> Opcode {
            BURRBOARD_STATE
        }

        fn emit_parameters<const N: usize>(
            &self,
            xmit: &mut heapless::Vec<u8, N>,
        ) -> Result<(), InsufficientBuffer> {
            match self {
                Self::State(state) => state.emit_parameters(xmit),
            }
        }
    }

    opcode!(BURRBOARD_STATE 0x12, 0x34);

    impl BurrBoardState {
        fn emit_parameters<const N: usize>(
            &self,
            xmit: &mut Vec<u8, N>,
        ) -> Result<(), InsufficientBuffer> {
            xmit.extend_from_slice(&self.temperature.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.brightness.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.accel[0].to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.accel[1].to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.accel[2].to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.push(self.battery).map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.button_a.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.extend_from_slice(&self.button_b.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?;
            xmit.push(if self.red_led { 1 } else { 0 })
                .map_err(|_| InsufficientBuffer)?;
            xmit.push(if self.green_led { 1 } else { 0 })
                .map_err(|_| InsufficientBuffer)?;
            xmit.push(if self.blue_led { 1 } else { 0 })
                .map_err(|_| InsufficientBuffer)?;
            xmit.push(if self.yellow_led { 1 } else { 0 })
                .map_err(|_| InsufficientBuffer)?;
            Ok(())
        }
    }
}
