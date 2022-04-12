use core::future::Future;
use drogue_device::actors::ble::mesh::{MeshNode, MeshNodeMessage, NodeMutex};
use drogue_device::drivers::ble::mesh::composition::{
    CompanyIdentifier, Composition, ElementDescriptor, ElementsHandler, Features, Location,
    ProductIdentifier, VersionIdentifier,
};
use drogue_device::drivers::ble::mesh::config::ConfigurationModel;
use drogue_device::drivers::ble::mesh::driver::elements::{AppElementContext, AppElementsContext};
use drogue_device::drivers::ble::mesh::driver::DeviceError;
use drogue_device::drivers::ble::mesh::model::generic::battery::{
    GenericBatteryFlags, GenericBatteryFlagsCharging, GenericBatteryFlagsIndicator,
    GenericBatteryFlagsPresence, GenericBatteryMessage, GenericBatteryServer,
    Status as GenericBatteryStatus, GENERIC_BATTERY_SERVER,
};
use drogue_device::drivers::ble::mesh::model::generic::onoff::{
    GenericOnOffMessage, GenericOnOffServer, GENERIC_ONOFF_CLIENT, GENERIC_ONOFF_SERVER,
};
use drogue_device::drivers::ble::mesh::model::sensor::{
    PropertyId, RawValue, SensorConfig, SensorData, SensorDescriptor, SensorMessage,
    SensorServer as SensorServerModel, SensorStatus, SENSOR_SERVER,
};
use drogue_device::drivers::ble::mesh::model::Message;
use drogue_device::drivers::ble::mesh::model::{Model, ModelIdentifier};
use drogue_device::drivers::ble::mesh::pdu::{access::AccessMessage, ParseError};
use drogue_device::drivers::ble::mesh::provisioning::{
    Algorithms, Capabilities, InputOOBActions, OOBSize, OutputOOBActions, PublicKeyType,
    StaticOOBType,
};
use drogue_device::drivers::ble::mesh::storage::FlashStorage;
use drogue_device::ActorContext;
use drogue_device::{
    drivers::ble::mesh::bearer::nrf52::{SoftdeviceAdvertisingBearer, SoftdeviceRng},
    Actor, Address, Inbox,
};
use drogue_device::{drivers::ble::mesh::InsufficientBuffer, flash::*};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy::time::{Duration, Ticker};
use futures::future::{select, Either};
use futures::{pin_mut, StreamExt};
use heapless::Vec;
use nrf_softdevice::{Flash, Softdevice};

use crate::{
    accel::{AccelRead, AccelValues},
    analog::AnalogRead,
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
    publisher: Address<PublisherMessage>,
}

impl BurrBoardElementsHandler {
    pub fn new(leds: Leds, publisher: Address<PublisherMessage>) -> Self {
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
        /*
        composition
            .add_element(ElementDescriptor::new(Location(0x0007)).add_model(GENERIC_ONOFF_SERVER))
            .ok();
        */
        Self {
            leds,
            composition,
            publisher,
        }
    }
}

impl ElementsHandler<'static> for BurrBoardElementsHandler {
    fn composition(&self) -> &Composition {
        &self.composition
    }

    fn connect(&mut self, ctx: AppElementsContext<'static>) {
        let _ = self
            .publisher
            .try_notify(PublisherMessage::Connect(ctx.clone()));
    }

    fn configure(&mut self, config: &ConfigurationModel) {
        if let Some(period) = config.publish_period_duration() {
            let _ = self.publisher.notify(PublisherMessage::SetPeriod(period));
        }
    }

    type DispatchFuture<'m> = impl Future<Output = Result<(), DeviceError>> + 'm
    where
        Self: 'm;

    fn dispatch<'m>(
        &'m mut self,
        element: u8,
        _: &'m ModelIdentifier,
        message: &'m AccessMessage,
    ) -> Self::DispatchFuture<'_> {
        async move {
            if element == 0x0001 {
                if let Ok(Some(GenericOnOffMessage::Set(set))) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    if set.on_off == 0 {
                        let _ = self.leds.red.off();
                    } else {
                        let _ = self.leds.red.on();
                    }
                }
            } else if element == 0x0002 {
                if let Ok(Some(GenericOnOffMessage::Set(set))) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    if set.on_off == 0 {
                        let _ = self.leds.green.off();
                    } else {
                        let _ = self.leds.green.on();
                    }
                }
            } else if element == 0x0003 {
                if let Ok(Some(GenericOnOffMessage::Set(set))) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    if set.on_off == 0 {
                        let _ = self.leds.blue.off();
                    } else {
                        let _ = self.leds.blue.on();
                    }
                }
            } else if element == 0x0004 {
                if let Ok(Some(GenericOnOffMessage::Set(set))) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    if set.on_off == 0 {
                        let _ = self.leds.yellow.off();
                    } else {
                        let _ = self.leds.yellow.on();
                    }
                }
            } else if element == 0x0007 {
                if let Ok(Some(GenericOnOffMessage::Set(set))) =
                    GenericOnOffServer::parse(message.opcode(), message.parameters())
                {
                    if set.on_off == 0 {
                        unsafe {
                            nrf_softdevice::raw::sd_power_gpregret_set(0, 0x1);
                            cortex_m::peripheral::SCB::sys_reset();
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

pub struct MeshApp {
    node: Forever<MeshNode<
        'static,
        BurrBoardElementsHandler,
        SoftdeviceAdvertisingBearer,
        FlashStorage<SharedFlash<'static, Flash>>,
        SoftdeviceRng,
    >>,
    publisher: ActorContext<BoardSensorPublisher>,
}

impl MeshApp {
    pub fn enable() -> Self {
        Self {
            node: Forever::new(),
            publisher: ActorContext::new(),
        }
    }

    pub fn mount(&'static self, s: Spawner, sd: &'static Softdevice, p: &BoardPeripherals) {
        extern "C" {
            static __storage: u8;
        }

        let storage: FlashStorage<SharedFlash<'static, Flash>> =
            FlashStorage::new(unsafe { &__storage as *const u8 as usize }, p.flash.clone());

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

        let node = self.node.put(
            MeshNode::new(elements, capabilities, bearer, storage, rng),
        );

        /*
        let mesh_node =
        //let mesh_node = MeshNode::new(capabilities, bearer, storage, rng).force_reset();
        */
    }
}

pub struct BoardSensorPublisher {
    ticker: Ticker,
    board: BoardPeripherals,
    context: Option<AppElementsContext<'static>>,
}

impl BoardSensorPublisher {
    pub fn new(interval: Duration, board: BoardPeripherals) -> Self {
        Self {
            ticker: Ticker::every(interval),
            board,
            context: None,
        }
    }
}

pub enum PublisherMessage {
    Connect(AppElementsContext<'static>),
    SetPeriod(Duration),
}

impl Actor for BoardSensorPublisher {
    type Message<'m> = PublisherMessage;
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where
        Self: 'm,
        M: 'm + Inbox<Self::Message<'m>>;

    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self::Message<'m>>,
        mut inbox: M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self::Message<'m>> + 'm,
    {
        async move {
            loop {
                let next = inbox.next();
                let tick = self.ticker.next();

                pin_mut!(next);
                pin_mut!(tick);

                match select(next, tick).await {
                    Either::Left((m, _)) => match m {
                        PublisherMessage::Connect(ctx) => {
                            info!("Connected to mesh {}", ctx.address());
                            self.context.replace(ctx.clone());
                        }
                        PublisherMessage::SetPeriod(period) => {
                            info!("Adjusting publish period to {} millis", period.as_millis());
                            self.ticker = Ticker::every(period);
                        }
                    },
                    Either::Right((_, _)) => {
                        let accel = self.board.accel.request(AccelRead).await;
                        let analog = self.board.analog.request(AnalogRead).await;
                        let (button_a, counter_a) = self.board.counter_a.request(CounterRead).await;
                        let (button_b, counter_b) = self.board.counter_b.request(CounterRead).await;

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
                            // Report battery status through model
                            let c = ctx.for_element_model::<GenericBatteryServer>(0);
                            let message = GenericBatteryMessage::Status(GenericBatteryStatus::new(
                                (data.battery as u32 * 127 / 100) as u8,
                                0,
                                0,
                                GenericBatteryFlags {
                                    presence: GenericBatteryFlagsPresence::PresentRemovable,
                                    indicator: GenericBatteryFlagsIndicator::Unknown,
                                    charging: GenericBatteryFlagsCharging::NotChargeable,
                                },
                            ));
                            match c.publish(message).await {
                                Ok(_) => {
                                    debug!("Published battery level");
                                }
                                Err(e) => {
                                    warn!("Error reporting battery level: {:?}", e);
                                }
                            }

                            // Report sensor data
                            let c = ctx.for_element_model::<SensorServer>(0);
                            let message = SensorMessage::Status(SensorStatus::new(data));
                            match c.publish(message).await {
                                Ok(_) => {
                                    debug!("Published sensor data");
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

    // States represents button_a, button_b, led_1, led_2, led_3, led_4 states as bits
    pub const STATES: PropertyId = PropertyId(1);
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
        SensorDescriptor::new(prop::STATES, 1),
        SensorDescriptor::new(prop::COUNTER_A, 2),
        SensorDescriptor::new(prop::COUNTER_B, 2),
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
    pub counter_a: u16,
    pub counter_b: u16,
    pub temperature: i16,
    pub brightness: u16,
    pub accel: (f32, f32, f32),
    pub battery: u8,
}

impl SensorData for SensorState {
    fn decode(&mut self, _: PropertyId, _: &[u8]) -> Result<(), ParseError> {
        todo!()
    }

    fn encode<const N: usize>(
        &self,
        property: PropertyId,
        xmit: &mut Vec<u8, N>,
    ) -> Result<(), InsufficientBuffer> {
        match property {
            prop::STATES => {
                let buttons_leds = self.button_a as u8;
                let buttons_leds = buttons_leds | (self.button_b as u8) << 1;
                let buttons_leds = buttons_leds | (self.red_led as u8) << 2;
                let buttons_leds = buttons_leds | (self.green_led as u8) << 3;
                let buttons_leds = buttons_leds | (self.blue_led as u8) << 4;
                let buttons_leds = buttons_leds | (self.yellow_led as u8) << 5;
                xmit.push(buttons_leds).map_err(|_| InsufficientBuffer)?;
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
