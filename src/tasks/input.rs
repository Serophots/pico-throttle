//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use defmt::info;
use embassy_rp::{i2c::I2c, peripherals::I2C0};

#[embassy_executor::task]
pub async fn input_task(i2c: I2c<'static, I2C0, embassy_rp::i2c::Async>) {
    info!("input task");
}
