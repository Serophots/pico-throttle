use bitflags::bitflags;

const ADDRESS: u8 = 0b00110110;

#[allow(unused)]
const REG_ANGLE_LO: u8 = 0x0F;
const REG_ANGLE_HI: u8 = 0x0E;
const REG_STATUS: u8 = 0x0B;
const REG_CONF_HI: u8 = 0x07;
#[allow(unused)]
const REG_CONF_LO: u8 = 0x08;

bitflags! {
    pub struct Status: u8 {
        const MAGNET_DETECTED  = 0b0010_0000; //MD detected
        const MAGNET_WEAK      = 0b0001_0000; //ML weak
        const MAGNET_STRONG    = 0b0000_1000; //MH strong
    }
}

#[derive(Clone, Copy)]
pub struct Config(u16);

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum PowerMode {
    Nom = 0b00,
    Lpm1 = 0b01,
    Lpm2 = 0b10,
    Lpm3 = 0b11,
}

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum Hysteresis {
    Off = 0b00,
    One = 0b01,
    Two = 0b10,
    Three = 0b11,
}

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum SlowFilter {
    X16 = 0b00,
    X8 = 0b01,
    X4 = 0b10,
    X2 = 0b11,
}

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum FastFilterThreshold {
    SlowFilterOnly = 0b000,
    Counts6 = 0b001,
    Counts7 = 0b010,
    Counts9 = 0b011,
    Counts18 = 0b100,
    Counts21 = 0b101,
    Counts24 = 0b110,
    Counts10 = 0b111,
}

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum Watchdog {
    On = 1,
    Off = 0,
}

macro_rules! config_bitfield {
    ($($name:ident : $val:expr),* $(,)?) => {
        paste::paste! {
            impl Config {
                $(
                    pub const [<$name:upper>]: u16 = $val;

                    pub fn [<get_ $name:snake:lower>](&self) -> [<$name:camel>] {
                        [<$name:camel>]::try_from(
                            (self.0 & Config::[<$name:upper>]) >> Config::[<$name:upper>].lowest_one().unwrap(),
                        )
                        .expect("bit mask")
                    }


                    pub fn [<set_ $name:snake:lower>](&mut self, t: [<$name:camel>]) {
                        self.0 = self.0 | ((t as u16) << Config::[<$name:upper>].lowest_one().unwrap());
                    }
                )*
            }
        }
    }
}

config_bitfield! {
    power_mode: 0b1100_0000_0000_0000,
    hysteresis: 0b0011_0000_0000_0000,
    // output_stage: 0b0000_1100_0000_0000,
    // pwm_freq: 0b0000_0011_0000_0000,
    slow_filter: 0b0000_0000_1100_0000,
    fast_filter_threshold: 0b0000_0000_0011_1000,
    watchdog: 0b0000_0000_0000_0100,
}

impl Config {
    pub fn new(inner: u16) -> Self {
        Config(inner & 0b0011_1111_1111_1111)
    }

    pub fn inner(&self) -> u16 {
        self.0 & 0b0011_1111_1111_1111
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
    pub async fn new_with_config(i2c: I2C, config: Config) -> Result<Self, I2C::Error> {
        let mut as5600 = As5600 { i2c };
        as5600.write_config(config).await?;
        Ok(as5600)
    }

    async fn read_u8(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];
        self.i2c.write_read(ADDRESS, &[reg], &mut buf).await?;
        Ok(u8::from_be_bytes(buf))
    }

    #[allow(unused)]
    async fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), I2C::Error> {
        let bytes = value.to_be_bytes();
        self.i2c.write(ADDRESS, &[reg, bytes[0]]).await?;
        Ok(())
    }

    async fn read_u16(&mut self, reg_hi: u8) -> Result<u16, I2C::Error> {
        let mut buf = [0u8; 2];
        self.i2c.write_read(ADDRESS, &[reg_hi], &mut buf).await?;
        Ok(u16::from_be_bytes(buf))
    }

    async fn write_u16(&mut self, reg_hi: u8, value: u16) -> Result<(), I2C::Error> {
        let buf = value.to_be_bytes();
        self.i2c.write(ADDRESS, &[reg_hi, buf[0], buf[1]]).await?;
        Ok(())
    }

    async fn read_config(&mut self) -> Result<Config, I2C::Error> {
        Ok(Config::new(self.read_u16(REG_CONF_HI).await?))
    }

    fn write_config(&mut self, config: Config) -> impl Future<Output = Result<(), I2C::Error>> {
        self.write_u16(REG_CONF_HI, config.inner())
    }

    pub async fn mut_config<F>(&mut self, f: F) -> Result<(), I2C::Error>
    where
        F: FnOnce(Config) -> Config,
    {
        let config = self.read_config().await?;
        self.write_config(f(config)).await?;
        Ok(())
    }

    pub async fn read_angle(&mut self) -> Result<u16, I2C::Error> {
        // read 12 bits
        let value = self.read_u16(REG_ANGLE_HI).await? & 0b0000_1111_1111_1111;
        // scale to 16 bit range
        let expanded = ((value as u32) * 0xFFFF / 0x0FFF) as u16;

        Ok(expanded)
    }

    pub async fn read_status(&mut self) -> Result<Status, I2C::Error> {
        let status = self.read_u8(REG_STATUS).await?;
        Ok(Status::from_bits_truncate(status))
    }
}
