use bitflags::bitflags;

pub struct HardwareState {
    axis0: u16,
    axis1: u16,
    buttons: HardwareButtons,
}

bitflags! {
    pub struct HardwareButtons: u32 {
        const THROTTLE_DISC_0 = 0b0000000000000001;
        const THROTTLE_DISC_1 = 0b0000000000000010;
        const THROTTLE_TOGA_0 = 0b0000000000000100;
        const THROTTLE_TOGA_1 = 0b0000000000001000;
        const ENG_MASTER_0    = 0b0000000000010000;
        const ENG_MASTER_1    = 0b0000000000100000;
        const ENG_REVERSE_0   = 0b0000000001000000;
        const ENG_REVERSE_1   = 0b0000000010000000;
        const IGNITION_CRANK  = 0b0000000100000000;
        const IGNITION_NORM   = 0b0000001000000000;
        const IGNITION_START  = 0b0000010000000000;
        const PARKING_BRAKE   = 0b0000100000000000;
    }
}
