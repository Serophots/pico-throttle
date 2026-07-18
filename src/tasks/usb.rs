//! embassy task: USB
//! receives HardwareState
//! writes HID reports

use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};
use embassy_usb::class::hid::{
    self as usb_hid, HidBootProtocol, HidProtocolMode, HidReaderWriter, HidSubclass,
};
use embassy_usb::{self as usb};
use static_cell::StaticCell;

mod report;

#[embassy_executor::task]
pub async fn usb_task(driver: UsbDriver<'static, USB>) {
    let mut builder = usb_builder(driver);
    let mut state = usb_hid::State::new();

    // builder.handler(handler);

    let hid = HidReaderWriter::<_, 1, 8>::new(
        &mut builder,
        &mut state,
        embassy_usb::class::hid::Config {
            report_descriptor: usbd_hid::descriptor::KeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
            hid_subclass: HidSubclass::Boot,
            hid_boot_protocol: HidBootProtocol::Keyboard,
        },
    );
}

fn usb_builder<'d>(driver: UsbDriver<'d, USB>) -> usb::Builder<'d, UsbDriver<'d, USB>> {
    fn usb_config<'a>() -> usb::Config<'a> {
        let mut config = usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Serophots");
        config.product = Some("Pico Throttle Quadrant");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config.composite_with_iads = false;
        config.device_class = 0;
        config.device_sub_class = 0;
        config.device_protocol = 0;
        config
    }

    let config = usb_config();

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init_with(|| [0u8; _]),
        BOS_DESCRIPTOR.init_with(|| [0u8; _]),
        MSOS_DESCRIPTOR.init_with(|| [0u8; _]),
        CONTROL_BUF.init_with(|| [0u8; _]),
    )
}
