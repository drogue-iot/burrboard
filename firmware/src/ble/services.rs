use heapless::Vec;

#[nrf_softdevice::gatt_server]
pub struct BurrBoardServer {
    pub board: BurrBoardService,
    pub device_info: DeviceInformationService,
    pub firmware: FirmwareUpdateService,
}

/// Gatt services for our module
#[nrf_softdevice::gatt_service(uuid = "1860")]
pub struct BurrBoardService {
    #[characteristic(uuid = "2a6e", read, notify)]
    temperature: i16,
    #[characteristic(uuid = "2b01", read, notify)]
    brightness: i16,

    #[characteristic(uuid = "2101", read, notify)]
    accel_x: i16,
    #[characteristic(uuid = "2102", read, notify)]
    accel_y: i16,
    #[characteristic(uuid = "2103", read, notify)]
    accel_z: i16,

    #[characteristic(uuid = "2a19", read, notify)]
    battery_level: u8,

    #[characteristic(uuid = "2aeb", read, notify)]
    button_a: u32,

    #[characteristic(uuid = "2aeb", read, notify)]
    button_b: u32,

    #[characteristic(uuid = "2ae2", write)]
    red_led: u8,
    #[characteristic(uuid = "2ae2", write)]
    green_led: u8,
    #[characteristic(uuid = "2ae2", write)]
    blue_led: u8,
    #[characteristic(uuid = "2ae2", write)]
    yellow_led: u8,

    #[characteristic(uuid = "1b25", read, write)]
    report_interval: u16,
}

#[nrf_softdevice::gatt_service(uuid = "180a")]
pub struct DeviceInformationService {
    #[characteristic(uuid = "2a24", read)]
    pub model_number: Vec<u8, 32>,
    #[characteristic(uuid = "2a25", read)]
    pub serial_number: Vec<u8, 32>,
    #[characteristic(uuid = "2a27", read)]
    pub hardware_revision: Vec<u8, 4>,
    #[characteristic(uuid = "2a29", read)]
    pub manufacturer_name: Vec<u8, 32>,
}

#[nrf_softdevice::gatt_service(uuid = "1861")]
pub struct FirmwareUpdateService {
    #[characteristic(uuid = "1234", write)]
    firmware: Vec<u8, 8>,

    #[characteristic(uuid = "1235", read, write)]
    offset: u32,

    #[characteristic(uuid = "1236", write)]
    control: u8,
}
