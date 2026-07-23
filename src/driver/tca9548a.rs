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
