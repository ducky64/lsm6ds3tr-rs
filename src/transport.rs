use embedded_hal_async::i2c::I2c;

/// Transport layer that supports both SPI and I2C
#[allow(async_fn_in_trait)]
pub trait Transport {
    type Error;
    async fn write_u8(&mut self, addr: u8, data: u8) -> Result<(), Self::Error>;
    async fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct I2cTransport<I2cType> {
    pub i2c: I2cType,
    pub address: u8,
}

impl<I2cType> Transport for I2cTransport<I2cType>
where
    I2cType: I2c,
{
    type Error = I2cType::Error;

    async fn write_u8(&mut self, addr: u8, data: u8) -> Result<(), I2cType::Error> {
        self.i2c.write(self.address, &[addr, data]).await?;
        Ok(())
    }

    async fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), I2cType::Error> {
        self.i2c.write_read(self.address, &[addr], buffer).await?;
        Ok(())
    }
}
