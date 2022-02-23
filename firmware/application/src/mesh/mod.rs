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
    PropertyId, RawValue, SensorConfig, SensorData, SensorMessage,
    SensorServer as SensorServerModel, SensorStatus, SENSOR_SERVER,
};
use drogue_device::drivers::ble::mesh::model::Model;
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
    onoff: GenericOnOffServer,
    battery: GenericBatteryServer,
    sensor: SensorServer,
    composition: Composition,
    leds: Leds,
    publisher: Address<BoardSensorPublisher>,
}

mod prop {
    use super::*;

    pub const RED_LED: PropertyId = PropertyId(1);
    pub const GREEN_LED: PropertyId = PropertyId(2);
    pub const BLUE_LED: PropertyId = PropertyId(3);
    pub const YELLOW_LED: PropertyId = PropertyId(4);
    pub const BUTTON_A: PropertyId = PropertyId(5);
    pub const BUTTON_B: PropertyId = PropertyId(6);
    pub const TEMP: PropertyId = PropertyId(7);
    pub const LIGHT: PropertyId = PropertyId(8);
    pub const ACCEL: PropertyId = PropertyId(9);
    pub const BATTERY: PropertyId = PropertyId(10);
}

pub struct BurrBoardSensors;

impl SensorConfig for BurrBoardSensors {
    fn value(id: PropertyId) -> usize {
        match id {
            prop::RED_LED => 1,
            prop::GREEN_LED => 1,
            prop::BLUE_LED => 1,
            prop::YELLOW_LED => 1,
            prop::BUTTON_A => 4,
            prop::BUTTON_B => 4,
            prop::TEMP => 2,
            prop::LIGHT => 2,
            prop::ACCEL => 6,
            prop::BATTERY => 1,
            _ => 0,
        }
    }
    fn x_value(id: PropertyId) -> usize {
        0
    }
    fn column_width(id: PropertyId) -> usize {
        0
    }
    fn y_value(id: PropertyId) -> usize {
        0
    }
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
            battery: GenericBatteryServer,
            onoff: GenericOnOffServer,
            sensor: SensorServer::new(),
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

    fn dispatch(&self, element: u8, message: AccessMessage) -> Self::DispatchFuture<'_> {
        async move {
            if element == 0x0001 {
                info!("Element 1");
                if let Ok(Some(m)) = self.onoff.parse(message.opcode(), message.parameters()) {
                    info!("LED 1 message: {:?}", m);
                }
            } else if element == 0x0002 {
                info!("Element 2");
                if let Ok(Some(m)) = self.onoff.parse(message.opcode(), message.parameters()) {
                    info!("LED 2 message: {:?}", m);
                }
            } else if element == 0x0003 {
                info!("Element 3");
                if let Ok(Some(m)) = self.onoff.parse(message.opcode(), message.parameters()) {
                    info!("LED 3 message: {:?}", m);
                }
            } else if element == 0x0004 {
                info!("Element 4");
                if let Ok(Some(m)) = self.onoff.parse(message.opcode(), message.parameters()) {
                    info!("LED 4 message: {:?}", m);
                }
            } else if element == 0x0005 {
                info!("Element 5");
                if let Ok(Some(m)) = self.battery.parse(message.opcode(), message.parameters()) {
                    info!("Battery message: {:?}", m);
                }
            } else if element == 0x0006 {
                info!("Element 6");
                if let Ok(Some(m)) = self.sensor.parse(message.opcode(), message.parameters()) {
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
            BoardSensorPublisher::new(Duration::from_secs(1), p.clone()),
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
                                    //self.ctx.replace(ctx.clone());
                                }
                            }
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

                        if let Some(ctx) = &self.context {
                            let t = analog.temperature.to_le_bytes();
                            let b = analog.brightness.to_le_bytes();
                            let accel_x = accel.x.to_le_bytes();
                            let accel_y = accel.y.to_le_bytes();
                            let accel_z = accel.z.to_le_bytes();
                            let accel: [u8; 6] = [
                                accel_x[0], accel_x[1], accel_y[0], accel_y[1], accel_z[0],
                                accel_z[1],
                            ];

                            let battery = analog.battery.to_le_bytes();
                            let button_a = button_a.to_le_bytes();
                            let button_b = button_b.to_le_bytes();

                            let red_led = (red_led as u8).to_le_bytes();
                            let green_led = (green_led as u8).to_le_bytes();
                            let blue_led = (blue_led as u8).to_le_bytes();
                            let yellow_led = (yellow_led as u8).to_le_bytes();

                            let mut values = heapless::Vec::new();
                            values.push(SensorData::new(prop::RED_LED, &red_led)).ok();
                            values
                                .push(SensorData::new(prop::GREEN_LED, &green_led))
                                .ok();
                            values.push(SensorData::new(prop::BLUE_LED, &blue_led)).ok();
                            values
                                .push(SensorData::new(prop::YELLOW_LED, &yellow_led))
                                .ok();

                            values.push(SensorData::new(prop::BUTTON_A, &button_a)).ok();
                            values.push(SensorData::new(prop::BUTTON_B, &button_b)).ok();
                            values.push(SensorData::new(prop::TEMP, &t)).ok();
                            values.push(SensorData::new(prop::LIGHT, &b)).ok();
                            values.push(SensorData::new(prop::ACCEL, &accel)).ok();
                            values.push(SensorData::new(prop::BATTERY, &battery)).ok();

                            let message = SensorMessage::Status(SensorStatus::new(values));
                            match ctx.publish(message).await {
                                Ok(_) => {
                                    info!("Sensor data reported successfully");
                                }
                                Err(e) => {
                                    warn!("Error reporting sensor data: {:?}", e);
                                }
                            }
                        } else {
                            let data = SensorState {
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

                            info!("Read sensor values: {:?}", data);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SensorState {
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
