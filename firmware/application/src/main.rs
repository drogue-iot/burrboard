#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![allow(unused)]

use drogue_device::{
    actors::button::{Button, ButtonPressed},
    actors::dfu::*,
    actors::flash::*,
    drivers::led::Led as LedDriver,
    traits::led::Led as _,
    ActorContext, Address,
};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, NoPin, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::{P0_02, P0_03, P0_04, P0_05, P0_06, P0_26, P0_27, P0_28, P0_30};
use embassy_nrf::uarte;
use embassy_nrf::{interrupt, Peripherals};
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::raw;
use nrf_softdevice::{Flash, Softdevice};

mod fmt;

#[cfg(not(any(feature = "gatt", feature = "mesh")))]
compile_error!("Neither 'gatt' nor 'mesh' activated. You must activate exactly one of the following features: gatt, mesh");

cfg_if::cfg_if! {
    if #[cfg(feature = "mesh")] {
        use drogue_device::actors::ble::mesh::MeshNode;
        use drogue_device::drivers::ble::mesh::bearer::nrf52::{
            Nrf52BleMeshFacilities, SoftdeviceAdvertisingBearer, SoftdeviceRng,
        };
        use drogue_device::drivers::ble::mesh::composition::{
            CompanyIdentifier, Composition, ElementDescriptor, ElementsHandler, Features, Location,
            ProductIdentifier, VersionIdentifier,
        };
        use drogue_device::drivers::ble::mesh::driver::elements::ElementContext;
        use drogue_device::drivers::ble::mesh::driver::DeviceError;
        use drogue_device::drivers::ble::mesh::model::generic::GENERIC_ONOFF_SERVER;
        use drogue_device::drivers::ble::mesh::pdu::access::AccessMessage;
        use drogue_device::drivers::ble::mesh::provisioning::{
            Algorithms, Capabilities, InputOOBActions, OOBSize, OutputOOBActions, PublicKeyType,
            StaticOOBType,
        };
        use drogue_device::drivers::ble::mesh::storage::FlashStorage;

        mod mesh;
        use mesh::*;
    } else if #[cfg(feature = "gatt")] {
        mod gatt;
        use gatt::*;

    }
}

#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;

#[cfg(feature = "log")]
mod logger;

#[cfg(not(feature = "defmt"))]
use panic_reset as _;

mod accel;
mod analog;
mod counter;
mod watchdog;

use accel::*;
use analog::*;
use counter::*;
use watchdog::*;

pub type RedLed = LedDriver<Output<'static, P0_06>>;
pub type GreenLed = LedDriver<Output<'static, P0_30>>;
pub type BlueLed = LedDriver<Output<'static, P0_28>>;
pub type YellowLed = LedDriver<Output<'static, P0_02>>;

pub type ButtonA = Input<'static, P0_27>;
pub type ButtonB = Input<'static, P0_26>;

pub type BatteryPin = P0_04;
pub type TemperaturePin = P0_05;
pub type LightPin = P0_03;

const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
const FIRMWARE_REVISION: Option<&str> = option_env!("REVISION");

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy::main(config = "config()")]
async fn main(s: embassy::executor::Spawner, p: Peripherals) {
    #[cfg(feature = "gatt")]
    let (sd, flash) = {
        let config = nrf_softdevice::Config {
            clock: Some(raw::nrf_clock_lf_cfg_t {
                source: raw::NRF_CLOCK_LF_SRC_RC as u8,
                rc_ctiv: 4,
                rc_temp_ctiv: 2,
                accuracy: 7,
            }),
            conn_gap: Some(raw::ble_gap_conn_cfg_t {
                conn_count: 6,
                event_length: 6,
            }),
            conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
            gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
                attr_tab_size: 32768,
            }),
            gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
                adv_set_count: 1,
                periph_role_count: 3,
                central_role_count: 0,
                central_sec_count: 0,
                _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
            }),
            gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
                p_value: b"BurrBoard" as *const u8 as _,
                current_len: 9,
                max_len: 9,
                write_perm: unsafe { core::mem::zeroed() },
                _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                    raw::BLE_GATTS_VLOC_STACK as u8,
                ),
            }),
            ..Default::default()
        };

        let sd = Softdevice::enable(&config);
        s.spawn(softdevice_task(sd)).unwrap();
        (sd, Flash::take(sd))
    };

    #[cfg(feature = "mesh")]
    let (facilities, flash) = {
        let facilities = Nrf52BleMeshFacilities::new("Drogue IoT BLE Mesh");
        let flash = facilities.flash();

        (facilities, flash)
    };

    #[cfg(feature = "log")]
    {
        logger::init(uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            p.P0_24,
            p.P0_25,
            NoPin,
            NoPin,
            Default::default(),
        ));
    }

    // Launch watchdog
    static WATCHDOG: ActorContext<Watchdog> = ActorContext::new();
    WATCHDOG.mount(s, Watchdog(Duration::from_secs(2)));

    // Ensure accel is ready
    Timer::after(Duration::from_millis(500)).await;

    // Actor for accelerometer
    static ACCEL: ActorContext<Accelerometer> = ActorContext::new();
    #[cfg(feature = "lsm")]
    let accel: Option<Address<Accelerometer>> =
        if let Ok(accel) = Accelerometer::new(p.TWISPI0, p.P0_12, p.P0_11) {
            Some(ACCEL.mount(s, accel))
        } else {
            None
        };
    #[cfg(not(feature = "lsm"))]
    let accel: Option<Address<Accelerometer>> = None;

    // Actor for all analog sensors
    static ANALOG: ActorContext<AnalogSensors> = ActorContext::new();
    let analog = ANALOG.mount(s, AnalogSensors::new(p.SAADC, p.P0_05, p.P0_03, p.P0_04));

    // LEDs
    let mut red = RedLed::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));
    let mut green = GreenLed::new(Output::new(p.P0_30, Level::Low, OutputDrive::Standard));
    let mut blue = BlueLed::new(Output::new(p.P0_28, Level::Low, OutputDrive::Standard));
    let mut yellow = YellowLed::new(Output::new(p.P0_02, Level::Low, OutputDrive::Standard));

    // Actor for button A and press counter
    static COUNTER_A: ActorContext<Counter> = ActorContext::new();
    static BUTTON_A: ActorContext<Button<ButtonA, ButtonPressed<Counter>>> = ActorContext::new();
    let counter_a = COUNTER_A.mount(s, Counter::new(BoardButton::A));
    let button_a = BUTTON_A.mount(
        s,
        Button::new(
            Input::new(p.P0_27, Pull::None),
            ButtonPressed(counter_a, CounterMessage::Increment),
        ),
    );

    // Actor for button B and press counter
    static COUNTER_B: ActorContext<Counter> = ActorContext::new();
    static BUTTON_B: ActorContext<Button<ButtonB, ButtonPressed<Counter>>> = ActorContext::new();
    let counter_b = COUNTER_B.mount(s, Counter::new(BoardButton::B));
    let button_b = BUTTON_B.mount(
        s,
        Button::new(
            Input::new(p.P0_26, Pull::None),
            ButtonPressed(counter_b, CounterMessage::Increment),
        ),
    );

    // Actor for shared access to flash
    static FLASH: ActorContext<SharedFlash<Flash>> = ActorContext::new();
    let flash = FLASH.mount(s, SharedFlash::new(flash));

    // Actor for DFU
    static DFU: ActorContext<FirmwareManager<SharedFlashHandle<Flash>>> = ActorContext::new();
    let dfu = DFU.mount(
        s,
        FirmwareManager::new(flash.into(), embassy_boot_nrf::updater::new()),
    );

    // Bootup animation
    red.on();
    Timer::after(Duration::from_secs(1)).await;
    green.on();
    Timer::after(Duration::from_secs(1)).await;
    blue.on();
    Timer::after(Duration::from_secs(1)).await;
    yellow.on();
    Timer::after(Duration::from_secs(1)).await;
    red.off();
    green.off();
    blue.off();
    yellow.off();

    let leds = Leds {
        red,
        green,
        blue,
        yellow,
    };

    #[cfg(feature = "mesh")]
    {
        extern "C" {
            static __storage: u8;
        }

        let bearer = facilities.bearer();
        let rng = facilities.rng();
        let storage = FlashStorage::new(unsafe { &__storage as *const u8 as usize }, flash.into());

        let capabilities = Capabilities {
            number_of_elements: 4,
            algorithms: Algorithms::default(),
            public_key_type: PublicKeyType::default(),
            static_oob_type: StaticOOBType::default(),
            output_oob_size: OOBSize::MaximumSize(4),
            output_oob_action: OutputOOBActions::default(),
            input_oob_size: OOBSize::MaximumSize(4),
            input_oob_action: InputOOBActions::default(),
        };

        let elements = BurrBoardElementsHandler::new(leds);

        static FACILITIES: ActorContext<Nrf52BleMeshFacilities> = ActorContext::new();

        FACILITIES.mount(s, facilities);

        static MESH: ActorContext<
            MeshNode<
                BurrBoardElementsHandler,
                SoftdeviceAdvertisingBearer,
                FlashStorage<SharedFlashHandle<Flash>>,
                SoftdeviceRng,
            >,
        > = ActorContext::new();

        let mesh_node = MeshNode::new(elements, capabilities, bearer, storage, rng);
        //let mesh_node = MeshNode::new(capabilities, bearer, storage, rng).force_reset();
        MESH.mount(s, mesh_node);
    }

    // BLE Gatt test service
    #[cfg(feature = "gatt")]
    {
        static GATT: Forever<BurrBoardServer> = Forever::new();
        let server = GATT.put(gatt_server::register(sd).unwrap());

        server.firmware.version_set(
            heapless::Vec::from_slice(FIRMWARE_REVISION.unwrap_or(FIRMWARE_VERSION).as_bytes())
                .unwrap(),
        );
        static MONITOR: ActorContext<BurrBoardMonitor> = ActorContext::new();
        let monitor = MONITOR.mount(
            s,
            BurrBoardMonitor::new(&server.board, analog, accel, counter_a, counter_b),
        );

        static FIRMWARE: ActorContext<BurrBoardFirmware> = ActorContext::new();
        let firmware = FIRMWARE.mount(s, BurrBoardFirmware::new(&server.firmware, dfu));
        s.spawn(bluetooth_task(sd, server, leds, monitor, firmware))
            .unwrap();
    }

    info!("Firmware started");
}

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

pub struct Leds {
    pub red: RedLed,
    pub green: GreenLed,
    pub blue: BlueLed,
    pub yellow: YellowLed,
}
