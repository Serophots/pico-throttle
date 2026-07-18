#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::Input;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver as UsbDriver;
use embassy_rp::{bind_interrupts, gpio::Pull};
use {defmt_rtt as _, panic_probe as _};

mod driver;
mod metadata;
mod state;
mod tasks;

pub use driver::*;
pub use metadata::*;
pub use state::*;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut signal_pin = Input::new(p.PIN_16, Pull::Up);
    signal_pin.set_schmitt(true);

    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(tasks::usb_task(driver, signal_pin).unwrap());
}
