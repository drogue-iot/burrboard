use core::future::Future;
use drogue_device::{Actor, Address, Inbox, Request};
use embassy_nrf::{interrupt, peripherals::SAADC, saadc};

pub struct AnalogSensors {
    saadc: saadc::Saadc<'static, 3>,
}

impl AnalogSensors {
    pub fn new(
        saadc: SAADC,
        mut temp: impl saadc::Input,
        mut light: impl saadc::Input,
        mut battery: impl saadc::Input,
    ) -> Self {
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

pub struct AnalogRead;

#[derive(Clone, Copy, Default)]
pub struct SensorValues {
    pub temperature: i16,
    pub brightness: u16,
    pub battery: u8,
}

pub type AnalogRequest = Request<AnalogRead, SensorValues>;

impl Actor for AnalogSensors {
    type Message<'m> = Request<AnalogRead, SensorValues>;

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
        Self: 'm,
    {
        async move {
            loop {
                let r = inbox.next().await;
                let mut buf = [0; 3];
                self.saadc.sample(&mut buf).await;

                let voltage = buf[0] as f32 * 3.3;
                let voltage = voltage / 4095 as f32;
                let temperature = (100.0 * (voltage - 0.5) * 100.0) as i16;
                let brightness = buf[1] as u16;

                let battery = buf[2] as u32;
                let battery = (100 * battery / 4056) as u8;

                trace!(
                    "Temperature: {:?}, brightness: {:?}, battery: {:?}",
                    temperature,
                    brightness,
                    battery
                );
                r.reply(SensorValues {
                    temperature,
                    brightness,
                    battery: battery as u8,
                })
                .await;
            }
        }
    }
}
