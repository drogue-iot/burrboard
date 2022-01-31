use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::{Duration, Timer};
use embassy_nrf::{
    interrupt,
    peripherals::{P0_03, P0_04, P0_05, SAADC},
    saadc,
};

pub struct AnalogSensors {
    saadc: saadc::Saadc<'static, 3>,
}

impl AnalogSensors {
    pub fn new(saadc: SAADC, mut temp: P0_05, mut light: P0_03, mut battery: P0_04) -> Self {
        let config = saadc::Config::default();
        let temp_channel = saadc::ChannelConfig::single_ended(&mut temp);
        let light_channel = saadc::ChannelConfig::single_ended(&mut light);
        let mut bat_channel = saadc::ChannelConfig::single_ended(&mut battery);
        bat_channel.time = saadc::Time::_40US;
        bat_channel.gain = saadc::Gain::GAIN1_5;
        bat_channel.resistor = saadc::Resistor::BYPASS;
        let saadc = saadc::Saadc::new(
            saadc,
            interrupt::take!(SAADC),
            config,
            [temp_channel, light_channel, bat_channel],
        );
        Self { saadc }
    }
}

impl Actor for AnalogSensors {
    type OnMountFuture<'m, M>
    where
        Self: 'm,
        M: 'm,
    = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(&'m mut self, _: Address<Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                let mut buf = [0; 3];
                self.saadc.sample(&mut buf).await;

                let voltage = buf[0] as f32 * 3.3;
                let voltage = voltage / 4095 as f32;
                let temperature = (100.0 * (voltage - 0.5) * 100.0) as i16;
                let brightness = buf[1] as u16;
                let battery = buf[1] as u16;

                info!(
                    "Temperature: {:?}, brightness: {:?}, battery: {:?}",
                    temperature, brightness, battery
                );
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}
