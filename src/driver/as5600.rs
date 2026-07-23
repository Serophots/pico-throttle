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
    _6 = 0b001,
    _7 = 0b010,
    _9 = 0b011,
    _18 = 0b100,
    _21 = 0b101,
    _24 = 0b110,
    _10 = 0b111,
}

#[derive(derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u16)]
pub enum Watchdog {
    On = 1,
    Off = 0,
}

impl Config {
    const POWER_MODE: u16 = 0b1100_0000_0000_0000;
    const HYSTERESIS: u16 = 0b0011_0000_0000_0000;
    #[allow(unused)]
    const OUTPUT_STAGE: u16 = 0b0000_1100_0000_0000;
    #[allow(unused)]
    const PWM_FREQ: u16 = 0b0000_0011_0000_0000;
    const SLOW_FILTER: u16 = 0b0000_0000_1100_0000;
    const FAST_FILTER_THRESHOLD: u16 = 0b0000_0000_0011_1000;
    const WATCHDOG: u16 = 0b0000_0000_0000_0100;

    pub fn new(inner: u16) -> Self {
        Config(inner & 0b0011_1111_1111_1111)
    }

    pub fn get_power_mode(&self) -> PowerMode {
        PowerMode::try_from((self.0 & Config::POWER_MODE) >> 14).expect("bit mask")
    }

    pub fn set_power_mode(&mut self) {
        unimplemented!()
    }

    pub fn get_hysteresis(&self) -> Hysteresis {
        Hysteresis::try_from((self.0 & Config::HYSTERESIS) >> 12).expect("bit mask")
    }

    pub fn set_hysteresis(&mut self) {
        unimplemented!()
    }

    pub fn get_slow_filter(&self) -> SlowFilter {
        SlowFilter::try_from((self.0 & Config::SLOW_FILTER) >> 6).expect("bit mask")
    }

    pub fn set_slow_filter(&mut self) {
        unimplemented!()
    }

    pub fn get_fast_filter_threshold(&self) -> FastFilterThreshold {
        FastFilterThreshold::try_from((self.0 & Config::FAST_FILTER_THRESHOLD) >> 3)
            .expect("bit mask")
    }

    pub fn set_fast_filter_threshold(&mut self) {
        unimplemented!()
    }

    pub fn get_watchdog(&self) -> Watchdog {
        Watchdog::try_from((self.0 & Config::WATCHDOG) >> 2).expect("bit mask")
    }

    pub fn set_watchdog(&mut self) {
        unimplemented!()
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

    async fn read_u8(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];
        self.i2c.write_read(ADDRESS, &[reg], &mut buf).await?;
        Ok(u8::from_be_bytes(buf))
    }

    async fn read_u16(&mut self, reg_hi: u8) -> Result<u16, I2C::Error> {
        let mut buf = [0u8; 2];
        self.i2c.write_read(ADDRESS, &[reg_hi], &mut buf).await?;
        Ok(u16::from_be_bytes(buf))
    }

    pub async fn read_config(&mut self) -> Result<Config, I2C::Error> {
        Ok(Config::new(self.read_u16(REG_CONF_HI).await?))
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
