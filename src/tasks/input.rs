//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use embassy_rp::{i2c::I2c, peripherals::I2C0};
use embassy_time::Timer;

use crate::{CHANNEL, driver::HardwarePins, tasks::HardwareDescriptor};

#[embassy_executor::task]
pub async fn input_task(
    i2c: I2c<'static, I2C0, embassy_rp::i2c::Async>,
    mut pins: HardwarePins<'static>,
) {
    let mut counter: usize = 0;

    loop {
        counter = counter.wrapping_add(1);
        Timer::after_millis(1).await;

        let buttons = pins.read();

        if counter == 2000 {
            counter = 0;
        }

        if counter >= 1000 {
            CHANNEL.signal(HardwareDescriptor {
                axis0: 0,
                axis1: u16::MAX,
                buttons: buttons.bits(),
            });
        } else {
            CHANNEL.signal(HardwareDescriptor {
                axis0: u16::MAX,
                axis1: 0,
                buttons: buttons.bits(),
            });
        }
    }
}
