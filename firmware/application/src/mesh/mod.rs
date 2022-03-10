use core::future::Future;
use drogue_device::drivers::ble::mesh::composition::{
    CompanyIdentifier, Composition, ElementDescriptor, ElementsHandler, Features, Location,
    ProductIdentifier, VersionIdentifier,
};
use drogue_device::drivers::ble::mesh::driver::elements::ElementContext;
use drogue_device::drivers::ble::mesh::driver::elements::{AppElementContext, AppElementsContext};
use drogue_device::drivers::ble::mesh::driver::DeviceError;
use drogue_device::drivers::ble::mesh::model::generic::battery::{
    GenericBatteryServer, GENERIC_BATTERY_SERVER,
};
use drogue_device::drivers::ble::mesh::model::generic::onoff::{
    GenericOnOffServer, GENERIC_ONOFF_SERVER,
};
use drogue_device::drivers::ble::mesh::model::sensor::{
    PropertyId, RawValue, SensorConfig, SensorData, SensorDescriptor, SensorMessage,
    SensorServer as SensorServerModel, SensorStatus, SENSOR_SERVER,
};
use drogue_device::drivers::ble::mesh::model::{Model, ModelIdentifier};
use drogue_device::drivers::ble::mesh::pdu::{access::AccessMessage, ParseError};
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
use heapless::Vec;
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

type SensorServer = SensorServerModel<BurrBoardSensors, 10, 1>;

#[allow(unused)]
pub struct BurrBoardElementsHandler {
    composition: Composition,
    leds: Leds,
    publisher: Address<BoardSensorPublisher>,
}

impl BurrBoardElementsHandler {
    pub fn new(leds: Leds, publisher: Address<BoardSensorPublisher>) -> Self {
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
            .add_element(ElementDescriptor::new(Location(0x0006)).add_model(SENSOR_SERVER))
            .ok();
        Self {
            leds,
            composition,
            publisher,
        }
    }
}

impl ElementsHandler for BurrBoardElementsHandler {
    fn composition(&self) -> &Composition {
        &self.composition
    }

    fn connect(&self, ctx: AppElementsContext) {
        info!("CONNECT");
        let sensor_ctx = ctx.for_element_model::<SensorServer>(0);
        let _ = self.publisher.notify(PublisherMessage::Connect(sensor_ctx));
    }

    type DispatchFuture<'m>
    where
        Self: 'm,
    = impl Future<Output = Result<(), DeviceError>> + 'm;

    fn dispatch<'m>(
        &'m self,
        element: u8,
        _: &'m ModelIdentifier,
        message: &'m AccessMessage,
    ) -> Self::DispatchFuture<'_> {
        async move {
            if element == 0x0001 {
                info!("Element 1");
                if let Ok(Some(m)) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    info!("LED 1 message: {:?}", m);
                }
            } else if element == 0x0002 {
                info!("Element 2");
                if let Ok(Some(m)) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    info!("LED 2 message: {:?}", m);
                }
            } else if element == 0x0003 {
                info!("Element 3");
                if let Ok(Some(m)) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    info!("LED 3 message: {:?}", m);
                }
            } else if element == 0x0004 {
                info!("Element 4");
                if let Ok(Some(m)) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    info!("LED 4 message: {:?}", m);
                }
            } else if element == 0x0005 {
                info!("Element 5");
                if let Ok(Some(m)) =
                    GenericBatteryServer::parse(message.opcode(), message.parameters())
                {
                    info!("Battery message: {:?}", m);
                }
            } else if element == 0x0006 {
                info!("Element 6");
                if let Ok(Some(m)) = SensorServer::parse(message.opcode(), message.parameters()) {
                    info!("Sensor message: {:?}", m);
                }
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
    publisher: ActorContext<BoardSensorPublisher>,
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

        let publisher = self.publisher.mount(
            s,
            BoardSensorPublisher::new(Duration::from_millis(1000), p.clone()),
        );

        let elements = BurrBoardElementsHandler::new(p.leds.clone(), publisher);

        self.node.mount(
            s,
            MeshNode::new(elements, capabilities, bearer, storage, rng),
        );

        /*
        let mesh_node =
        //let mesh_node = MeshNode::new(capabilities, bearer, storage, rng).force_reset();
        */
    }
}

pub struct BoardSensorPublisher {
    interval: Duration,
    board: BoardPeripherals,
    context: Option<AppElementContext<SensorServer>>,
}

impl BoardSensorPublisher {
    pub fn new(interval: Duration, board: BoardPeripherals) -> Self {
        Self {
            interval,
            board,
            context: None,
        }
    }
}

pub enum PublisherMessage {
    Connect(AppElementContext<SensorServer>),
}

impl Actor for BoardSensorPublisher {
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
                        if let Some(mut m) = m {
                            match m.message() {
                                PublisherMessage::Connect(ctx) => {
                                    info!("connected to mesh {}", ctx.address());
                                    self.context.replace(ctx.clone());
                                }
                            }
                        }
                    }
                    Either::Right((_, _)) => {
                        let accel = self.board.accel.request(AccelRead).unwrap().await.unwrap();
                        let analog = self.board.analog.request(AnalogRead).unwrap().await;
                        let (button_a, counter_a) = self
                            .board
                            .counter_a
                            .request(CounterMessage::Read)
                            .unwrap()
                            .await
                            .unwrap();
                        let (button_b, counter_b) = self
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

                        let data = SensorState {
                            temperature: analog.temperature,
                            brightness: analog.brightness,
                            accel: (accel.x, accel.y, accel.z),
                            battery: analog.battery,
                            button_a,
                            button_b,
                            counter_a,
                            counter_b,
                            red_led,
                            green_led,
                            blue_led,
                            yellow_led,
                        };

                        if let Some(ctx) = &self.context {
                            let message = SensorMessage::Status(SensorStatus::new(data));
                            match ctx.publish(message).await {
                                Ok(_) => {
                                    info!("Sensor data reported successfully");
                                }
                                Err(e) => {
                                    warn!("Error reporting sensor data: {:?}", e);
                                }
                            }
                        } else {
                            info!("Read sensor values: {:?}", data);
                        }
                    }
                }
            }
        }
    }
}

mod prop {
    use super::*;

    pub const RED_LED: PropertyId = PropertyId(1);
    pub const GREEN_LED: PropertyId = PropertyId(2);
    pub const BLUE_LED: PropertyId = PropertyId(3);
    pub const YELLOW_LED: PropertyId = PropertyId(4);
    pub const BUTTON_A: PropertyId = PropertyId(5);
    pub const BUTTON_B: PropertyId = PropertyId(6);
    pub const COUNTER_A: PropertyId = PropertyId(7);
    pub const COUNTER_B: PropertyId = PropertyId(8);
    pub const TEMPERATURE: PropertyId = PropertyId(9);
    pub const BRIGHTNESS: PropertyId = PropertyId(10);
    pub const ACCEL: PropertyId = PropertyId(11);
    pub const BATTERY: PropertyId = PropertyId(12);
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone)]
pub struct BurrBoardSensors;

impl SensorConfig for BurrBoardSensors {
    type Data<'m> = SensorState;

    const DESCRIPTORS: &'static [SensorDescriptor] = &[
        SensorDescriptor::new(prop::RED_LED, 1),
        SensorDescriptor::new(prop::GREEN_LED, 1),
        SensorDescriptor::new(prop::BLUE_LED, 1),
        SensorDescriptor::new(prop::YELLOW_LED, 1),
        SensorDescriptor::new(prop::BUTTON_A, 1),
        SensorDescriptor::new(prop::BUTTON_B, 1),
        SensorDescriptor::new(prop::COUNTER_A, 4),
        SensorDescriptor::new(prop::COUNTER_B, 4),
        SensorDescriptor::new(prop::TEMPERATURE, 2),
        SensorDescriptor::new(prop::BRIGHTNESS, 2),
        SensorDescriptor::new(prop::ACCEL, 12),
        SensorDescriptor::new(prop::BATTERY, 1),
    ];
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorState {
    pub red_led: bool,
    pub green_led: bool,
    pub blue_led: bool,
    pub yellow_led: bool,
    pub button_a: bool,
    pub button_b: bool,
    pub counter_a: u32,
    pub counter_b: u32,
    pub temperature: i16,
    pub brightness: u16,
    pub accel: (f32, f32, f32),
    pub battery: u8,
}

impl SensorData for SensorState {
    fn decode(&mut self, property: PropertyId, data: &[u8]) -> Result<(), ParseError> {
        todo!()
    }

    fn encode<const N: usize>(
        &self,
        property: PropertyId,
        xmit: &mut Vec<u8, N>,
    ) -> Result<(), InsufficientBuffer> {
        match property {
            prop::RED_LED => {
                xmit.push(if self.red_led { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::GREEN_LED => {
                xmit.push(if self.green_led { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::BLUE_LED => {
                xmit.push(if self.blue_led { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::YELLOW_LED => {
                xmit.push(if self.yellow_led { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::BUTTON_A => {
                xmit.push(if self.button_a { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::BUTTON_B => {
                xmit.push(if self.button_a { 1 } else { 0 })
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::COUNTER_A => {
                xmit.extend_from_slice(&self.counter_a.to_le_bytes())
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::COUNTER_B => {
                xmit.extend_from_slice(&self.counter_b.to_le_bytes())
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::TEMPERATURE => xmit
                .extend_from_slice(&self.temperature.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?,
            prop::BRIGHTNESS => xmit
                .extend_from_slice(&self.temperature.to_le_bytes())
                .map_err(|_| InsufficientBuffer)?,
            prop::ACCEL => {
                xmit.extend_from_slice(&self.accel.0.to_le_bytes())
                    .map_err(|_| InsufficientBuffer)?;
                xmit.extend_from_slice(&self.accel.1.to_le_bytes())
                    .map_err(|_| InsufficientBuffer)?;
                xmit.extend_from_slice(&self.accel.2.to_le_bytes())
                    .map_err(|_| InsufficientBuffer)?;
            }
            prop::BATTERY => {
                xmit.push(self.battery).map_err(|_| InsufficientBuffer)?;
            }
            _ => (),
        }
        Ok(())
    }
}
