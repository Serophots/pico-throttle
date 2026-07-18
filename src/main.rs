#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::Input;
use embassy_rp::i2c::{self, I2c};
use embassy_rp::peripherals::{I2C0, USB};
use embassy_rp::usb::Driver as UsbDriver;
use embassy_rp::{bind_interrupts, gpio::Pull};
use {defmt_rtt as _, panic_probe as _};

pub mod driver;
mod state;
mod tasks;

pub use state::*;

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Pico Throttle Quadrant"),
    embassy_rp::binary_info::rp_program_description!(
        c"Implements a USB Human Interface Device for a Raspberry Pi Pico 2 throttle quadrant. Written by Serophots."
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut signal_pin = Input::new(p.PIN_16, Pull::Up);
    signal_pin.set_schmitt(true);

    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(tasks::usb_task(driver, signal_pin).unwrap());

    let mut i2c_config = i2c::Config::default();
    i2c_config.scl_pullup = false;
    i2c_config.sda_pullup = false;

    let i2c = I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, i2c_config);
    spawner.spawn(tasks::input_task(i2c).unwrap());
}
