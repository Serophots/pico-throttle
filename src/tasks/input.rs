//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState

use embassy_rp::{gpio::Input, i2c::I2c, peripherals::I2C0};
use embassy_time::Timer;

use crate::{CHANNEL, HardwareButtons, tasks::HardwareDescriptor};

#[embassy_executor::task]
pub async fn input_task(
    i2c: I2c<'static, I2C0, embassy_rp::i2c::Async>,
    mut pins: InputTaskPins<'static>,
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

pub struct InputTaskPins<'d> {
    pub throttle_disc_0: Button<'d>,
    pub throttle_disc_1: Button<'d>,
    pub throttle_toga_0: Button<'d>,
    pub throttle_toga_1: Button<'d>,
    pub eng_master_0: Button<'d>,
    pub eng_master_1: Button<'d>,
    pub eng_reverse_0: Button<'d>,
    pub eng_reverse_1: Button<'d>,
    pub ignition_crank: Button<'d>,
    pub ignition_norm: Button<'d>,
    pub ignition_start: Button<'d>,
    pub parking_brake: Button<'d>,
    pub unused_0: Button<'d>,
    pub unused_1: Button<'d>,
    pub unused_2: Button<'d>,
    pub unused_3: Button<'d>,
    pub unused_4: Button<'d>,
    pub unused_5: Button<'d>,
    pub unused_6: Button<'d>,
    pub unused_7: Button<'d>,
}

impl<'d> InputTaskPins<'d> {
    pub fn read(&mut self) -> HardwareButtons {
        let mut buttons = HardwareButtons::empty();

        let InputTaskPins {
            // ensure we're exhausting all fields
            throttle_disc_0,
            throttle_disc_1,
            throttle_toga_0,
            throttle_toga_1,
            eng_master_0,
            eng_master_1,
            eng_reverse_0,
            eng_reverse_1,
            ignition_crank,
            ignition_norm,
            ignition_start,
            parking_brake,
            unused_0,
            unused_1,
            unused_2,
            unused_3,
            unused_4,
            unused_5,
            unused_6,
            unused_7,
        } = self;

        if throttle_disc_0.read() {
            buttons |= HardwareButtons::THROTTLE_DISC_0;
        }
        if throttle_disc_1.read() {
            buttons |= HardwareButtons::THROTTLE_DISC_1;
        }
        if throttle_toga_0.read() {
            buttons |= HardwareButtons::THROTTLE_TOGA_0;
        }
        if throttle_toga_1.read() {
            buttons |= HardwareButtons::THROTTLE_TOGA_1;
        }
        if eng_master_0.read() {
            buttons |= HardwareButtons::ENG_MASTER_0;
        }
        if eng_master_1.read() {
            buttons |= HardwareButtons::ENG_MASTER_1;
        }
        if eng_reverse_0.read() {
            buttons |= HardwareButtons::ENG_REVERSE_0;
        }
        if eng_reverse_1.read() {
            buttons |= HardwareButtons::ENG_REVERSE_1;
        }
        if ignition_crank.read() {
            buttons |= HardwareButtons::IGNITION_CRANK;
        }
        if ignition_norm.read() {
            buttons |= HardwareButtons::IGNITION_NORM;
        }
        if ignition_start.read() {
            buttons |= HardwareButtons::IGNITION_START;
        }
        if parking_brake.read() {
            buttons |= HardwareButtons::PARKING_BRAKE;
        }
        if unused_0.read() {
            buttons |= HardwareButtons::UNUSED_0;
        }
        if unused_1.read() {
            buttons |= HardwareButtons::UNUSED_1;
        }
        if unused_2.read() {
            buttons |= HardwareButtons::UNUSED_2;
        }
        if unused_3.read() {
            buttons |= HardwareButtons::UNUSED_3;
        }
        if unused_4.read() {
            buttons |= HardwareButtons::UNUSED_4;
        }
        if unused_5.read() {
            buttons |= HardwareButtons::UNUSED_5;
        }
        if unused_6.read() {
            buttons |= HardwareButtons::UNUSED_6;
        }
        if unused_7.read() {
            buttons |= HardwareButtons::UNUSED_7;
        }

        buttons
    }
}

pub struct Button<'d> {
    gpio: Input<'d>,
    stable: bool,
    counter: u8,
}

impl<'d> Button<'d> {
    pub fn new(gpio: Input<'d>) -> Button<'d> {
        let stable = gpio.is_high();
        Button {
            gpio,
            stable,
            counter: 0,
        }
    }

    pub fn read(&mut self) -> bool {
        let unstable = self.gpio.is_high();

        if self.counter == 20 {
            self.stable = unstable;
            self.counter = 0;
        }

        if self.stable == unstable {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        self.stable
    }
}
