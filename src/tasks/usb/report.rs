use usbd_hid::descriptor::generator_prelude::*;

/// MouseReport describes a report and its companion descriptor than can be used
/// to send mouse movements and button presses to a host.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_8) = {
                #[packed_bits = 8] #[item_settings(data,variable,absolute)] buttons=input;
            };
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    #[item_settings(data,variable,relative)] x=input;
                };
                (usage = Y,) = {
                    #[item_settings(data,variable,relative)] y=input;
                };
                (usage = WHEEL,) = {
                    #[item_settings(data,variable,relative)] wheel=input;
                };
            };
            (usage_page = CONSUMER,) = {
                (usage = AC_PAN,) = {
                    #[item_settings(data,variable,relative)] pan=input;
                };
            };
        };
    }
)]
#[allow(dead_code)]
pub struct MouseReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8, // Scroll down (negative) or up (positive) this many units
    pub pan: i8,   // Scroll left (negative) or right (positive) this many units
}

/// KeyboardReport describes a report and its companion descriptor that can be
/// used to send keyboard button presses to a host and receive the status of the
/// keyboard LEDs.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits = 8] #[item_settings(data,variable,absolute)] modifier=input;
        };
        (usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings(constant,variable,absolute)] reserved=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits = 5] #[item_settings(data,variable,absolute)] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings(data,array,absolute)] keycodes=input;
        };
    }
)]
#[allow(dead_code)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
        (usage_page = GENERIC_DESKTOP, usage_min = 0x0000, usage_max = 0xFFFF) = {
            #[item_settings(data,variable,absolute)] axis=output;
        };
    }
)]
pub struct HardwareReport {
    axis: u16,
}
