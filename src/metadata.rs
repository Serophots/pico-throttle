use embassy_usb as usb;

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
