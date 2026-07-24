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

    let as5600_config = {
        let mut cfg = as5600::Config::new(0);
        cfg.set_power_mode(as5600::PowerMode::Nom);
        cfg.set_hysteresis(as5600::Hysteresis::One);
        cfg.set_slow_filter(
            // Highest latency but lowest noise
            as5600::SlowFilter::X16,
        );
        cfg.set_fast_filter_threshold(
            // Slow -> Fast: 18 counts
            // Fast -> Slow: 2 counts
            as5600::FastFilterThreshold::Counts18,
        );
        cfg.set_watchdog(
            // Don't try to move into low power mode, ever
            as5600::Watchdog::Off,
        );
        cfg
    };
    let mut sensor_axis0 = As5600::new_with_config(i2c_ch0, as5600_config)
        .with_timeout(I2C_TIMEOUT)
        .await
        .err_log()
        .transpose()
        .err_log()
        .flatten();
    let mut sensor_axis1 = As5600::new_with_config(i2c_ch1, as5600_config)
        .with_timeout(I2C_TIMEOUT)
        .await
        .err_log()
        .transpose()
        .err_log()
        .flatten();

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
            let mut buttons = pins.read().bits();

            // Safety:
            // These futures should NOT be polled in parallel
            // there's some interior mutabilty going on under
            // the hood which I've not been able to express
            // in idiomatic rust :(

            let mut axis0 = u16::MAX / 2;
            let mut axis1 = u16::MAX / 2;
            let mut axis0_status = as5600::Status::empty();
            let mut axis1_status = as5600::Status::empty();

            if let Some(sensor_axis0) = &mut sensor_axis0 {
                axis0 = read_angle(sensor_axis0).await;
                axis0_status = read_status(sensor_axis0).await;
            }

            if let Some(sensor_axis1) = &mut sensor_axis1 {
                axis1 = read_angle(sensor_axis1).await;
                axis1_status = read_status(sensor_axis1).await;
            }

            // Note ^: Bunch reads on the same axis so the
            // multiplexer has to switch channels less often

            buttons = (buttons | ((axis0_status.bits() as u32) << 23))
                | ((axis1_status.bits() as u32) << 26);

            HardwareDescriptor {
                axis0,
                axis1,
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
        // .err_log()
        .ok()
        .transpose()
        .err_log()
        .flatten()
        .unwrap_or(u16::MAX)
}

#[inline]
async fn read_status<I2C>(sensor: &mut As5600<I2C>) -> as5600::Status
where
    I2C: embedded_hal_async::i2c::I2c,
    <I2C as embedded_hal_1::i2c::ErrorType>::Error: defmt::Format,
{
    sensor
        .read_status()
        .with_timeout(I2C_TIMEOUT)
        .await
        // .err_log()
        .ok()
        .transpose()
        .err_log()
        .flatten()
        .unwrap_or(as5600::Status::empty())
}
