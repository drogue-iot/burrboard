use core::future::Future;
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

use crate::Leds;

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
            }
            Ok(())
        }
    }
}
