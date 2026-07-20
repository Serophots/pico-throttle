use embassy_rp::gpio::Input;

use crate::HardwareButtons;

pub struct HardwarePins<'d> {
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

impl<'d> HardwarePins<'d> {
    pub fn read(&mut self) -> HardwareButtons {
        let mut buttons = HardwareButtons::empty();

        let HardwarePins {
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

        for (button, hardware_button) in [
            (throttle_disc_0, HardwareButtons::THROTTLE_DISC_0),
            (throttle_disc_1, HardwareButtons::THROTTLE_DISC_1),
            (throttle_toga_0, HardwareButtons::THROTTLE_TOGA_0),
            (throttle_toga_1, HardwareButtons::THROTTLE_TOGA_1),
            (eng_master_0, HardwareButtons::ENG_MASTER_0),
            (eng_master_1, HardwareButtons::ENG_MASTER_1),
            (eng_reverse_0, HardwareButtons::ENG_REVERSE_0),
            (eng_reverse_1, HardwareButtons::ENG_REVERSE_1),
            (ignition_crank, HardwareButtons::IGNITION_CRANK),
            (ignition_norm, HardwareButtons::IGNITION_NORM),
            (ignition_start, HardwareButtons::IGNITION_START),
            (parking_brake, HardwareButtons::PARKING_BRAKE),
            (unused_0, HardwareButtons::UNUSED_0),
            (unused_1, HardwareButtons::UNUSED_1),
            (unused_2, HardwareButtons::UNUSED_2),
            (unused_3, HardwareButtons::UNUSED_3),
            (unused_4, HardwareButtons::UNUSED_4),
            (unused_5, HardwareButtons::UNUSED_5),
            (unused_6, HardwareButtons::UNUSED_6),
            (unused_7, HardwareButtons::UNUSED_7),
        ] {
            if button.read() {
                buttons |= hardware_button;
            }
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
