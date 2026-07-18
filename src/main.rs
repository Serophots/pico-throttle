#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver as UsbDriver;
use embassy_usb::class::hid::{
    self as usb_hid, HidBootProtocol, HidProtocolMode, HidReaderWriter, HidSubclass,
};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
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

static HID_PROTOCOL_MODE: AtomicU8 = AtomicU8::new(HidProtocolMode::Boot as u8);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(tasks::usb_task(driver).unwrap());

    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = usb_hid::State::new();
    let mut builder = usb_builder_new(driver);

    builder.handler(&mut device_handler);

    // Create classes on the builder.
    let hid = HidReaderWriter::<_, 1, 8>::new(
        &mut builder,
        &mut state,
        embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
            hid_subclass: HidSubclass::Boot,
            hid_boot_protocol: HidBootProtocol::Keyboard,
        },
    );

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Set up the signal pin that will be used to trigger the keyboard.
    let mut signal_pin = Input::new(p.PIN_16, Pull::Up);

    // Enable the schmitt trigger to slightly debounce.
    signal_pin.set_schmitt(true);

    let (reader, mut writer) = hid.split();

    // Do stuff with the class!
    let in_fut = async {
        loop {
            info!("Waiting for LOW on pin 16");
            signal_pin.wait_for_low().await;
            info!("LOW DETECTED");

            if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&[0, 0, 4, 0, 0, 0, 0, 0]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", e),
                };
            } else {
                // Create a report with the A key pressed. (no shift modifier)
                let report = KeyboardReport {
                    keycodes: [4, 0, 0, 0, 0, 0],
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                };
                // Send the report.
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                };
            }
            info!("awaiting high");
            signal_pin.wait_for_high().await;
            info!("HIGH DETECTED");
            if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&[0, 0, 0, 0, 0, 0, 0, 0]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", e),
                };
            } else {
                let report = KeyboardReport {
                    keycodes: [0, 0, 0, 0, 0, 0],
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                };
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                };
            }
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyRequestHandler {}

impl usb_hid::RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: usb_hid::ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(
        &mut self,
        id: usb_hid::ReportId,
        data: &[u8],
    ) -> embassy_usb::control::OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        embassy_usb::control::OutResponse::Accepted
    }

    fn get_protocol(&self) -> HidProtocolMode {
        let protocol = HidProtocolMode::from(HID_PROTOCOL_MODE.load(Ordering::Relaxed));
        info!("The current HID protocol mode is: {}", protocol);
        protocol
    }

    fn set_protocol(&mut self, protocol: HidProtocolMode) -> embassy_usb::control::OutResponse {
        info!("Switching to HID protocol mode: {}", protocol);
        HID_PROTOCOL_MODE.store(protocol as u8, Ordering::Relaxed);
        embassy_usb::control::OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<usb_hid::ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<usb_hid::ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl embassy_usb::Handler for MyDeviceHandler {
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
