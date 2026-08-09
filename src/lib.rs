#![no_std]

mod registers;
pub use registers::*;
mod transport;
use transport::*;

use embedded_hal_async::i2c::I2c;


#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NewDataAvailable {
    pub temp: bool,
    pub gyro: bool,
    pub accelerometer: bool,
}

pub struct Lsm6ds3tr<TransportType>
where
    TransportType: Transport,
{
    transport: TransportType,
    xl_config: Option<(OdrXl, FsXl)>,
    g_config: Option<(OdrG, FsG)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NormalizationError<TransportError> {
    Transport(TransportError),
    NotConfigured,
}

impl<TransportError> From<TransportError> for NormalizationError<TransportError> {
    fn from(err: TransportError) -> Self {
        NormalizationError::Transport(err)
    }
}

impl<I2cType> Lsm6ds3tr<I2cTransport<I2cType>>
where
    I2cType: I2c,
{
    const LSM6DS3TR_ID: u8 = 0x6A; // SA0 = 0

    /// Creates a device with address with SA0=0
    pub fn new_i2c(i2c: I2cType) -> Self {
        Self::new_i2c_with_sa0(i2c, 0)
    }

    /// Creates a device with SA0 offset (0 or 1)
    pub fn new_i2c_with_sa0(i2c: I2cType, sa0: u8) -> Self {
        Self {
            transport: I2cTransport {
                i2c,
                address: Self::LSM6DS3TR_ID + sa0,
            },
            xl_config: None,
            g_config: None,
        }
    }
}

impl<TransportType> Lsm6ds3tr<TransportType>
where
    TransportType: Transport,
{
    const WHO_AM_I_ID: u8 = 0b01101010;

    /// Returns the value of the WHO_AM_I register
    pub async fn read_whoami(&mut self) -> Result<u8, TransportType::Error> {
        let mut buffer = [0u8; 1];
        self.transport
            .read(RegisterAddress::WhoAmI as u8, &mut buffer)
            .await?;
        Ok(buffer[0])
    }

    /// Reads the WHO_AM_I register, checking the value against the expected for this chip
    pub async fn check(&mut self) -> Result<bool, TransportType::Error> {
        Ok(self.read_whoami().await? == Self::WHO_AM_I_ID)
    }

    /// Issues a software reset
    pub async fn reset(&mut self) -> Result<(), TransportType::Error> {
        let mut ctrl3 = Ctrl3CStruct::default();
        ctrl3.set_sw_reset(true);
        self.transport
            .write_u8(RegisterAddress::Ctrl3C as u8, u8::from(ctrl3))
            .await?;
        Ok(())
    }

    /// Sets the accelerometer configuration
    pub async fn config_xl(&mut self, odr: OdrXl, fs: FsXl) -> Result<(), TransportType::Error> {
        let ctrl1 = Ctrl1Struct::new(Bw0Xl::Hz1k5, false, fs, odr);
        self.transport
            .write_u8(RegisterAddress::Ctrl1Xl as u8, u8::from(ctrl1))
            .await?;
        self.xl_config = Some((odr, fs));
        Ok(())
    }

    /// Sets the gyroscope configuration
    pub async fn config_g(&mut self, odr: OdrG, fs: FsG) -> Result<(), TransportType::Error> {
        let ctrl2 = Ctrl2Struct::new(fs, odr);
        self.transport
            .write_u8(RegisterAddress::Ctrl2G as u8, u8::from(ctrl2))
            .await?;
        self.g_config = Some((odr, fs));
        Ok(())
    }

    /// Returns whether there is new data, from the status register
    pub async fn new_data(&mut self) -> Result<NewDataAvailable, TransportType::Error> {
        let mut buffer = [0u8; 1];
        self.transport
            .read(RegisterAddress::StatusReg as u8, &mut buffer)
            .await?;
        let status_reg = StatusRegStruct::from(buffer[0]);
        Ok(NewDataAvailable {
            temp: status_reg.tda(),
            gyro: status_reg.gda(),
            accelerometer: status_reg.xlda(),
        })
    }

    /// Reads the temperature, returning the raw data (0 = 25c, 256 LSB/C)
    pub async fn read_temp_raw(&mut self) -> Result<i16, TransportType::Error> {
        let mut buffer = [0u8; 2];
        self.transport
            .read(RegisterAddress::OutTempL as u8, &mut buffer)
            .await?;
        Ok(i16::from_le_bytes(buffer))
    }

    /// Reads the temperature, converted to Celsius, assuming center temperature offset
    pub async fn read_temp_celsius(&mut self) -> Result<f32, TransportType::Error> {
        let raw = self.read_temp_raw().await?;
        let temp_c = (raw as f32) / 256.0 + 25.0;
        Ok(temp_c)
    }

    /// Reads the accelerometer, returning the raw i16 triple
    pub async fn read_xl_raw(&mut self) -> Result<(i16, i16, i16), TransportType::Error> {
        let mut buffer = [0u8; 6];
        self.transport
            .read(RegisterAddress::OutXLXl as u8, &mut buffer)
            .await?;
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
        Ok((x, y, z))
    }

    /// Reads the accelerometer, returning the f32 triple normalized in units of g
    pub async fn read_xl_g(
        &mut self,
    ) -> Result<(f32, f32, f32), NormalizationError<TransportType::Error>> {
        let scale = if let Some((_, fs)) = self.xl_config {
            fs.sensitivity_mg_lsb() / 1000.0
        } else {
            return Err(NormalizationError::NotConfigured);
        };

        let (x_raw, y_raw, z_raw) = self.read_xl_raw().await?;
        Ok((
            x_raw as f32 * scale,
            y_raw as f32 * scale,
            z_raw as f32 * scale,
        ))
    }

    /// Reads the gyroscope, returning the raw i16 triple
    pub async fn read_g_raw(&mut self) -> Result<(i16, i16, i16), TransportType::Error> {
        let mut buffer = [0u8; 6];
        self.transport
            .read(RegisterAddress::OutXLG as u8, &mut buffer)
            .await?;
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
        Ok((x, y, z))
    }

    pub async fn read_g_dps(
        &mut self,
    ) -> Result<(f32, f32, f32), NormalizationError<TransportType::Error>> {
        let scale = if let Some((_, fs)) = self.g_config {
            fs.sensitivity_mdps_lsb() / 1000.0
        } else {
            return Err(NormalizationError::NotConfigured);
        };

        let (x_raw, y_raw, z_raw) = self.read_g_raw().await?;
        Ok((
            x_raw as f32 * scale,
            y_raw as f32 * scale,
            z_raw as f32 * scale,
        ))
    }
}
