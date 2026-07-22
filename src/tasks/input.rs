//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use core::cell::RefCell;

use defmt::{error, info};
use embassy_rp::{gpio::Output, i2c::I2c, peripherals::I2C1};
use embassy_time::{Duration, Instant, Ticker};
use embedded_hal_1::digital::{OutputPin, PinState};
use static_cell::StaticCell;

use crate::{
    CHANNEL,
    driver::{
        HardwarePins,
        as5600::As5600,
        tca9548a::{self, Tca9548a},
    },
    tasks::HardwareDescriptor,
};

#[embassy_executor::task]
pub async fn input_task(
    tca9548a: Tca9548a<I2c<'static, I2C1, embassy_rp::i2c::Async>>,
    mut pins: HardwarePins<'static>,
    mut led: Output<'static>,
) {
    static TCA9548A: StaticCell<RefCell<Tca9548a<I2c<'static, I2C1, embassy_rp::i2c::Async>>>> =
        StaticCell::new();
    let tca9548a = TCA9548A.init_with(|| RefCell::new(tca9548a));

    let i2c_ch0 = tca9548a::Channel::new(tca9548a, 0);
    let i2c_ch1 = tca9548a::Channel::new(tca9548a, 1);

    let mut axis0 = As5600::new(i2c_ch0);
    let mut axis1 = As5600::new(i2c_ch1);

    let mut ticker = Ticker::every(Duration::from_millis(1));

    let mut led_state = PinState::High;
    let mut led_instant = Instant::now();

    loop {
        led.set_state(led_state);
        let buttons = pins.read();

        if led_instant.elapsed() > Duration::from_secs(1) {
            led_instant = Instant::now();
            led_state = !led_state;
        }

        CHANNEL.signal(HardwareDescriptor {
            axis0: axis0
                .read_angle()
                .await
                .map_err(|e| {
                    error!("{:?}", e);
                    e
                })
                .unwrap_or(u16::MAX),
            axis1: axis1
                .read_angle()
                .await
                .map_err(|e| {
                    error!("{:?}", e);
                    e
                })
                .unwrap_or(u16::MAX),
            buttons: buttons.bits(),
        });

        ticker.next().await;
    }
}
