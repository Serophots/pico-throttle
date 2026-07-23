#![no_std]
#![no_main]

use crate::driver::tca9548a::Tca9548a;
use crate::driver::{Button, HardwarePins};
use crate::tasks::HardwareDescriptor;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::i2c::{self, I2c};
use embassy_rp::peripherals::{I2C1, USB};
use embassy_rp::usb::Driver as UsbDriver;
use embassy_rp::{bind_interrupts, gpio::Pull};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use {defmt_rtt as _, panic_probe as _};

pub mod driver;
mod result;
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
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<I2C1>;
});

static CHANNEL: Signal<CriticalSectionRawMutex, HardwareDescriptor> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let led = Output::new(p.PIN_25, Level::Low);

    // reserve I2C0 & associated pins for the debug pico
    let _ = p.PIN_0;
    let _ = p.PIN_1;
    let _ = p.I2C0;

    let input_task_pins = HardwarePins::<'static> {
        throttle_disc_0: Button::new(Input::new(p.PIN_0, Pull::Up)),
        throttle_disc_1: Button::new(Input::new(p.PIN_1, Pull::Up)),
        throttle_toga_0: Button::new(Input::new(p.PIN_4, Pull::Up)),
        throttle_toga_1: Button::new(Input::new(p.PIN_5, Pull::Up)),
        eng_reverse_0: Button::new(Input::new(p.PIN_6, Pull::Up)),
        eng_reverse_1: Button::new(Input::new(p.PIN_7, Pull::Up)),
        eng_master_0: Button::new(Input::new(p.PIN_8, Pull::Up)),
        eng_master_1: Button::new(Input::new(p.PIN_9, Pull::Up)),
        ignition_crank: Button::new(Input::new(p.PIN_10, Pull::Up)),
        ignition_norm: Button::new(Input::new(p.PIN_11, Pull::Up)),
        ignition_start: Button::new(Input::new(p.PIN_12, Pull::Up)),
        parking_brake: Button::new(Input::new(p.PIN_13, Pull::Up)),
        unused_0: Button::new(Input::new(p.PIN_14, Pull::Up)),
        unused_1: Button::new(Input::new(p.PIN_15, Pull::Up)),
        unused_2: Button::new(Input::new(p.PIN_16, Pull::Up)),
        unused_3: Button::new(Input::new(p.PIN_17, Pull::Up)),
        unused_4: Button::new(Input::new(p.PIN_18, Pull::Up)),
        unused_5: Button::new(Input::new(p.PIN_19, Pull::Up)),
        unused_6: Button::new(Input::new(p.PIN_20, Pull::Up)),
        unused_7: Button::new(Input::new(p.PIN_21, Pull::Up)),
        unused_8: Button::new(Input::new(p.PIN_22, Pull::Up)),
        unused_9: Button::new(Input::new(p.PIN_26, Pull::Up)),
        unused_10: Button::new(Input::new(p.PIN_27, Pull::Up)),
        unused_11: Button::new(Input::new(p.PIN_28, Pull::Up)),
    };

    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(tasks::usb_task(driver).unwrap());

    let mut i2c_config = i2c::Config::default();
    i2c_config.scl_pullup = false;
    i2c_config.sda_pullup = false;

    let i2c = I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, i2c_config);
    let tca9548a = Tca9548a::new(i2c);

    spawner.spawn(tasks::input_task(tca9548a, input_task_pins, led).unwrap());
}
