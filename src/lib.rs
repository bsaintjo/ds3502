use embedded_hal::i2c::I2c;
use embedded_hal_async::i2c::I2c as AsyncI2c;

const DEFAULT_I2C_ADDR: u8 = 0x23;

struct Config {
    i2c_addr: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            i2c_addr: DEFAULT_I2C_ADDR,
        }
    }
}

pub struct Ds3502<I2C> {
    i2c: I2C,
}

pub enum Ds3502Error {
    InvalidWiperValue,
    I2cError,
}

#[derive(Clone, Copy, Debug)]
pub struct WiperValue(u8);

impl TryFrom<u8> for WiperValue {
    type Error = Ds3502Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 127 {
            Err(Ds3502Error::InvalidWiperValue)
        } else {
            Ok(Self(value))
        }
    }
}

enum ControlRegisterMode {
    WiperOnly,
    WiperAndInitialValue,
}

impl ControlRegisterMode {
    fn as_register_value(self) -> u8 {
        match self {
            ControlRegisterMode::WiperOnly => 0x00,
            ControlRegisterMode::WiperAndInitialValue => 0x80,
        }
    }
}

impl<I2C: I2c> Ds3502<I2C> {
    pub fn new_blocking(i2c: I2C) -> Self {
        Ds3502 { i2c }
    }

    /// Changes whether the call to write_wiper will be saved
    fn set_control_register_mode(&mut self, mode: ControlRegisterMode) -> Result<(), Ds3502Error> {
        self.i2c
            .write(DEFAULT_I2C_ADDR, &[0x02, mode.as_register_value()])
            .map_err(|_| Ds3502Error::I2cError)
    }

    // fn set_initial_value(&mut self, value: WiperValue) -> Result<(), Ds3502Error> {
    //     self.i2c
    //         .write(DEFAULT_I2C_ADDR, &[0x00, value.0])
    //         .map_err(|_| Ds3502Error::I2cError)
    // }

    /// Set wiper to value between 0 and 127
    pub fn write_wiper(&mut self, value: WiperValue) -> Result<(), Ds3502Error> {
        self.i2c
            .write(DEFAULT_I2C_ADDR, &[0x00, value.0])
            .map_err(|_| Ds3502Error::I2cError)
    }
}

impl<I2C: AsyncI2c> Ds3502<I2C> {
    pub fn new_async(i2c: I2C) -> Self {
        Ds3502 { i2c }
    }

    /// Set wiper to value between 0 and 127
    pub async fn async_write_wiper(&mut self, value: WiperValue) -> Result<(), Ds3502Error> {
        self.i2c
            .write(DEFAULT_I2C_ADDR, &[value.0])
            .await
            .map_err(|_| Ds3502Error::I2cError)
    }
}

pub fn adc_to_wiper_value(mut x: f32) -> f32 {
    x *= 5.0;
    x /= 1024.;
    x
}
