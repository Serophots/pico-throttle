use crate::driver;

pub struct HardwareState {
    axis: u16,
    eng_mode_selector: driver::EngSelector,
}
