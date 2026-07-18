use usbd_hid::descriptor::generator_prelude::*;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
        (usage_page = GENERIC_DESKTOP,) = {
            (usage = X,) = {
                #[item_settings(data,variable,absolute,volatile)] axis=input;
            };
        };
        (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_8,) = {
            #[packed_bits = 8] #[item_settings(data,variable,absolute)] buttons=input;
        };
    }
)]
pub struct HardwareDescriptor {
    pub axis: u16,
    pub buttons: u8,
}

fn main() {
    println!("{:02X?}", HardwareDescriptor::desc());
}
