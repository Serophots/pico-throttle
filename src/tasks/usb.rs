use core::sync::atomic::{AtomicBool, Ordering};

use defmt::{error, info};
use embassy_futures::join::join;
use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};
use embassy_time::{Duration, WithTimeout};
use embassy_usb::class::hid::{self as usb_hid, HidBootProtocol, HidReaderWriter, HidSubclass};
use embassy_usb::{self as usb};
use static_cell::StaticCell;
use usbd_hid::descriptor::SerializedDescriptor;

pub use descriptor::HardwareDescriptor;

use crate::CHANNEL;

#[embassy_executor::task]
pub async fn usb_task(driver: UsbDriver<'static, USB>) {
    static STATE: StaticCell<usb_hid::State<'static>> = StaticCell::new();
    static DEVICE_HANDLER: StaticCell<DeviceHandler> = StaticCell::new();
    static REQUEST_HANDLER: StaticCell<RequestHandler> = StaticCell::new();

    let request_handler = REQUEST_HANDLER.init_with(|| RequestHandler {});
    let device_handler = DEVICE_HANDLER.init_with(|| DeviceHandler::new());

    let mut builder: usb::Builder<'static, UsbDriver<'static, USB>> =
        crate::tasks::usb_builder(driver);

    builder.handler(device_handler);

    let state = STATE.init_with(|| usb_hid::State::new());

    let hid = HidReaderWriter::<_, 1, 8>::new(
        &mut builder,
        state,
        embassy_usb::class::hid::Config {
            report_descriptor: descriptor::HardwareDescriptor::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
            hid_subclass: HidSubclass::No,
            hid_boot_protocol: HidBootProtocol::None,
        },
    );

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

    let usb_writer = async {
        writer
            .write_serialize(&HardwareDescriptor::default())
            .await
            .unwrap();

        loop {
            match CHANNEL.wait().with_timeout(Duration::from_secs(1)).await {
                Ok(hardware_descriptor) => {
                    writer.write_serialize(&hardware_descriptor).await.unwrap();
                }
                Err(_) => {
                    error!("usb channel timed out");
                }
            }
        }
    };
    let usb_reader = async { reader.run(false, request_handler) };

    join(usb_fut, join(usb_writer, usb_reader)).await;
}

mod descriptor {
    use usbd_hid::descriptor::generator_prelude::*;

    #[gen_hid_descriptor(
        (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    #[item_settings(data,variable,absolute,volatile)] axis0=input;
                };
            };
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = Y,) = {
                    #[item_settings(data,variable,absolute,volatile)] axis1=input;
                };
            };
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = 32,) = {
                #[packed_bits = 32] #[item_settings(data,variable,absolute)] buttons=input;
            };
        }
    )]
    #[derive(Default)]
    pub struct HardwareDescriptor {
        pub axis0: u16,
        pub axis1: u16,
        pub buttons: u32,
    }
}

pub fn usb_builder<'d>(driver: UsbDriver<'d, USB>) -> usb::Builder<'d, UsbDriver<'d, USB>> {
    fn usb_config<'a>() -> usb::Config<'a> {
        let mut config = usb::Config::new(0x5e40, 0xface);
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

struct RequestHandler {}

impl usb_hid::RequestHandler for RequestHandler {
    // The default methods are implemented with no-op's
}

struct DeviceHandler {
    configured: AtomicBool,
}

impl DeviceHandler {
    fn new() -> Self {
        DeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl embassy_usb::Handler for DeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
