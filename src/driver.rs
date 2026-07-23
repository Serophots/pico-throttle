use embassy_rp::gpio::Input;

use crate::HardwareButtons;

pub mod tca9548a {
    use core::{cell::RefCell, marker::PhantomData};

    const ADDRESS: u8 = 0b01110000;

    pub struct Tca9548a<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c,
    {
        i2c: I2C,
        channel: Option<u8>,
    }

    impl<I2C> Tca9548a<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c,
    {
        pub fn new(i2c: I2C) -> Self {
            Tca9548a { i2c, channel: None }
        }

        pub async fn select(&mut self, channel: u8) -> Result<(), I2C::Error> {
            if self.channel == Some(channel) {
                return Ok(());
            }

            self.i2c.write(ADDRESS, &[1 << channel]).await?;
            self.channel = Some(channel);

            Ok(())
        }
    }

    pub struct Channel<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c + 'static,
    {
        tca9548a: &'static RefCell<Tca9548a<I2C>>,
        channel: u8,
        _phantom: PhantomData<I2C>,
    }

    impl<I2C> Channel<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c + 'static,
    {
        pub fn new(tca9548a: &'static RefCell<Tca9548a<I2C>>, channel: u8) -> Self {
            Channel {
                tca9548a,
                channel,
                _phantom: PhantomData,
            }
        }
    }

    impl<I2C> embedded_hal_1::i2c::ErrorType for Channel<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c + 'static,
    {
        type Error = I2C::Error;
    }

    impl<I2C> embedded_hal_async::i2c::I2c for Channel<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c + 'static,
    {
        async fn transaction(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal_async::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            // Safety: ermmm
            // actually this is really bad practise..
            let mut tca9548a = self.tca9548a.borrow_mut();
            tca9548a.select(self.channel).await?;
            tca9548a.i2c.transaction(address, operations).await?;
            Ok(())
        }
    }
}

pub mod as5600 {
    use bitflags::bitflags;

    const ADDRESS: u8 = 0b00110110;

    #[allow(unused)]
    const REG_ANGLE_LOWER: u8 = 0x0F;
    const REG_ANGLE_UPPER: u8 = 0x0E;
    const REG_STATUS: u8 = 0x0B;

    bitflags! {
        pub struct Status: u8 {
            const MAGNET_DETECTED  = 0b0010_0000;
            const MAGNET_WEAK      = 0b0001_0000;
            const MAGNET_STRONG    = 0b0000_1000;
        }
    }

    pub struct As5600<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c,
    {
        i2c: I2C,
    }

    impl<I2C> As5600<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c,
    {
        pub fn new(i2c: I2C) -> Self {
            As5600 { i2c }
        }

        pub async fn read_angle(&mut self) -> Result<u16, I2C::Error> {
            let mut angle_upper = [0u8; 1];
            let mut angle_lower = [0u8; 1];
            self.i2c
                .write_read(ADDRESS, &[REG_ANGLE_UPPER], &mut angle_upper)
                .await?;
            // AS5600 increments the address pointer to REG_ANGLE_LOWER
            self.i2c.read(ADDRESS, &mut angle_lower).await?;

            Ok(((angle_lower[0] as u16) << 8) | ((angle_upper[0] as u16) << 4))
        }

        pub async fn read_status(&mut self) -> Result<Status, I2C::Error> {
            let mut status = [0u8; 1];
            self.i2c
                .write_read(ADDRESS, &[REG_STATUS], &mut status)
                .await?;

            Ok(Status::from_bits_truncate(status[0]))
        }
    }
}

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
    pub unused_8: Button<'d>,
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
            unused_8,
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
            (unused_8, HardwareButtons::UNUSED_8),
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
        let unstable = self.gpio.is_low();

        // If I2C is timing out then buttons are
        // read 4x less frequently, and a high
        // debounce counter will result in noticeable
        // latency.
        if self.counter == 10 {
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
