//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use core::cell::RefCell;

use embassy_rp::{gpio::Output, i2c::I2c, peripherals::I2C0};
use embassy_time::{Duration, Ticker};
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
    tca9548a: Tca9548a<I2c<'static, I2C0, embassy_rp::i2c::Async>>,
    mut pins: HardwarePins<'static>,
    mut led: Output<'static>,
) {
    static TCA9548A: StaticCell<RefCell<Tca9548a<I2c<'static, I2C0, embassy_rp::i2c::Async>>>> =
        StaticCell::new();
    let tca9548a = TCA9548A.init_with(|| RefCell::new(tca9548a));

    let i2c_ch0 = tca9548a::Channel::new(tca9548a, 0);
    let i2c_ch1 = tca9548a::Channel::new(tca9548a, 1);

    let mut axis0 = As5600::new(i2c_ch0);
    let mut axis1 = As5600::new(i2c_ch1);

    let mut ticker = Ticker::every(Duration::from_millis(1));
    let mut led_state = PinState::Low;

    loop {
        led.set_state(led_state);
        let buttons = pins.read();

        CHANNEL.signal(HardwareDescriptor {
            axis0: axis0.read_angle().await.unwrap(),
            axis1: axis1.read_angle().await.unwrap(),
            buttons: buttons.bits(),
        });

        ticker.next().await;
        led_state = !led_state;
    }
}
