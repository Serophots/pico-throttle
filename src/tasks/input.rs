//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use core::cell::RefCell;

use embassy_rp::{gpio::Output, i2c::I2c, peripherals::I2C1};
use embassy_time::{Duration, Instant, Ticker, WithTimeout};
use embedded_hal_1::digital::{OutputPin, PinState};
use static_cell::StaticCell;

use crate::{
    CHANNEL,
    driver::{
        HardwarePins,
        as5600::{self, As5600},
        tca9548a::{self, Tca9548a},
    },
    result::ResultExt,
    tasks::HardwareDescriptor,
};

const I2C_TIMEOUT: Duration = Duration::from_millis(2);

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

    let mut sensor_axis0 = As5600::new(i2c_ch0);
    let mut sensor_axis1 = As5600::new(i2c_ch1);

    let mut ticker = Ticker::every(Duration::from_millis(1));

    let mut led_state = PinState::High;
    let mut led_instant = Instant::now();

    loop {
        led.set_state(led_state);

        if led_instant.elapsed() > Duration::from_secs(1) {
            led_instant = Instant::now();
            led_state = !led_state;
        }

        let descriptor = {
            let buttons = pins.read().bits();

            // Safety:
            // These futures should NOT be polled in parallel
            // there's some interior mutabilty going on under
            // the hood which I've not been able to express
            // in idiomatic rust :(
            let axis0 = read_angle(&mut sensor_axis0).await;
            let axis0_status = read_status(&mut sensor_axis0).await;
            let axis1 = read_angle(&mut sensor_axis1).await;
            let axis1_status = read_status(&mut sensor_axis1).await;

            // Note ^: Bunch reads on the same axis so the
            // multiplexer has to switch channels less often

            HardwareDescriptor {
                axis0,
                axis1,
                axis0_status,
                axis1_status,
                buttons,
            }
        };

        CHANNEL.signal(descriptor);

        ticker.next().await;
    }
}

#[inline]
async fn read_angle<I2C>(sensor: &mut As5600<I2C>) -> u16
where
    I2C: embedded_hal_async::i2c::I2c,
    <I2C as embedded_hal_1::i2c::ErrorType>::Error: defmt::Format,
{
    sensor
        .read_angle()
        .with_timeout(I2C_TIMEOUT)
        .await
        .err_log()
        .transpose()
        .err_log()
        .flatten()
        .unwrap_or(u16::MAX)
}

#[inline]
async fn read_status<I2C>(sensor: &mut As5600<I2C>) -> u8
where
    I2C: embedded_hal_async::i2c::I2c,
    <I2C as embedded_hal_1::i2c::ErrorType>::Error: defmt::Format,
{
    sensor
        .read_status()
        .with_timeout(I2C_TIMEOUT)
        .await
        .err_log()
        .transpose()
        .err_log()
        .flatten()
        .as_ref()
        .map(as5600::Status::bits)
        .unwrap_or(0)
}
