#![deny(warnings)]
#![no_std]

//! # I2C Digitial Potentiometer DS3502 driver in Rust
//!
//! ![Crates.io License](https://img.shields.io/crates/l/ds3502)
//! ![Crates.io Version](https://img.shields.io/crates/v/ds3502)
//! ![docs.rs](https://img.shields.io/docsrs/ds3502)
//! ![Crates.io MSRV](https://img.shields.io/crates/msrv/ds3502)
//! ![embedded-hal Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbsaintjo%2Fds3502%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=%24.dependencies.embedded-hal&label=embedded-hal)
//!
//! Rust embedded driver for the DS3502 Digital Potentiometer by Analog Devices Inc./Maxim Integrated. Supports block and async APIs.
//!
//! ## Usage
//!
//! ```
//! # use embedded_hal::i2c::I2c;
//! # use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//! use ds3502::{Ds3502, Wiper, ControlRegisterMode};
//! # use ds3502::Ds3502Error;
//! # fn main() -> Result<(), Ds3502Error> {
//! # let expectations = [
//! # I2cTransaction::write(0x28, vec![2, 0x80]),
//! # I2cTransaction::write(0x28, vec![0, 88]),
//! # I2cTransaction::write(0x28, vec![0x2, ControlRegisterMode::WiperAndInitialValue as u8]),
//! # I2cTransaction::write(0x28, vec![0x0, 123]),
//! # I2cTransaction::write(0x28, vec![0x2, ControlRegisterMode::WiperOnly as u8]),
//! # ];
//! # let mut i2c_mock = I2cMock::new(&expectations);
//! # let i2c = i2c_mock.clone();
//! let mut digipot = Ds3502::blocking_init(i2c, Default::default())?;
//!
//! // By default, driver initialized with Control Register in Mode 1 (Don't save to EEPROM)
//! assert_eq!(digipot.mode(), ControlRegisterMode::WiperOnly);
//!
//! let wv = Wiper::try_from(88)?;
//! digipot.write_wiper(wv)?;
//!
//! // Set wiper value and save to EEPROM
//! let wv = Wiper::try_from(123)?;
//! digipot.write_and_save_wiper(wv);
//! # i2c_mock.done();
//! # Ok(())
//! # }
//! ```
//!
//! For a complete example that builds and flashes to a Raspberry Pi Pico using the `embassy` framework,
//! checkout the [`pico-example`](http://github.com/bsaintjo/ds3502/blob/main/examples/pico-example/) directory.
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! ds3502 = "0.1"
//! ```
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](http://www.github.com/bsaintjo/ds3502/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](http://www.github.com/bsaintjo/ds3502/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
//! work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
//! additional terms or conditions.

use embedded_hal::i2c::{Error as I2cError, I2c};
use embedded_hal_async::i2c::I2c as AsyncI2c;

/// Represents the  I2C address for the DS3502.
///
/// The DS3502 has address pins A1 and A0 to natively support up to four
/// DS3502s on the same I2C bus.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum I2cAddr {
    /// Default I2C Address (0x28)
    Default = 0x28,
    /// I2C Address with Address 0 pin high (0x29)
    Address0 = 0x29,
    /// I2C Address with Address 1 pin high (0x2a)
    Address1 = 0x2a,
    /// I2C Address with Address 0 & 1 pin high (0x2a)
    Address01 = 0x2b,
}

/// Configuration of the DS3502 for set-up.
///
/// # Differences from default behavior
/// According to the datasheet, by default, the control register (CR) is initialized in Mode 0 (0x00),
/// so I2C writes to both the Wiper Register (WR) and initial value register (IVR), which saves the WR value
/// to the EEPROM.
///
/// The EEPROM has a limited number of write cycles, so the default configuration initializes the driver with CR in Mode 1 (0x80)
/// the factory behavior by setting the control register mode to be wiper register only.
///
/// To restore factory behavior set the `mode` to [`ControlRegisterMode::WiperAndInitialValue`]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Config {
    /// The default I2C address is 0x28
    pub i2c_addr: I2cAddr,

    /// By default, EEPROM saves are disabled, see [`ControlRegisterMode`] for more details.
    pub mode: ControlRegisterMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            i2c_addr: I2cAddr::Default,
            mode: ControlRegisterMode::WiperOnly,
        }
    }
}

pub mod error {
    //! Error for DS3502
    use core::fmt::Display;

    use embedded_hal::i2c::ErrorKind;

    /// Error new-type around embedded-hal I2C [`ErrorKind`](https://docs.rs/embedded-hal/1.0.0/embedded_hal/i2c/enum.ErrorKind.html)
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct I2cErrorKind(pub(crate) ErrorKind);

    impl Display for I2cErrorKind {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0.fmt(f)
        }
    }
}

/// Error during DS3502 operation
#[derive(Debug, Clone, thiserror::Error, Hash, PartialEq, Eq)]
pub enum Ds3502Error {
    /// Wiper value must be between 0 and 127 inclusive.
    #[error("Invalid wiper position value.")]
    InvalidWiperValue,

    /// Error on the I2C Bus.
    #[error("I2C error: {0}")]
    I2cError(error::I2cErrorKind),
}

impl<E> From<E> for Ds3502Error
where
    E: I2cError,
{
    fn from(value: E) -> Self {
        Self::I2cError(error::I2cErrorKind(value.kind()))
    }
}

/// Represents the position of the digital wiper.
///
/// Must be between 0 - 127.
///
/// Example
/// ```
/// use ds3502::Wiper;
///
/// assert!(Wiper::try_from(0).is_ok());
/// assert!(Wiper::try_from(20).is_ok());
/// assert!(Wiper::try_from(127).is_ok());
/// assert!(Wiper::try_from(128).is_err());
/// ```
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wiper(u8);

impl Wiper {
    /// Retrieve inner wiper position
    #[must_use]
    pub fn inner(&self) -> u8 {
        self.0
    }
}

impl AsRef<u8> for Wiper {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl TryFrom<u8> for Wiper {
    type Error = Ds3502Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 127 {
            Err(Ds3502Error::InvalidWiperValue)
        } else {
            Ok(Self(value))
        }
    }
}

/// Control whether writes to the wiper register (WR) are also written to the initial value register (IVR) on the EEPROM.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlRegisterMode {
    /// Write to WR/IVR
    WiperAndInitialValue = 0x00,
    /// Write to WR only
    WiperOnly = 0x80,
}

/// Represents a driver for the DS3502.
///
/// # Example
/// ```
/// # use embedded_hal::i2c::I2c;
/// # use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
/// # use ds3502::{Ds3502Error, Ds3502, Wiper};
/// # fn main() -> Result<(), Ds3502Error> {
/// # let expectations = [
/// # I2cTransaction::write(0x28, vec![2, 0x80]),
/// # I2cTransaction::write(0x28, vec![0, 88]),
/// # ];
/// # let mut i2c_mock = I2cMock::new(&expectations);
/// # let i2c = i2c_mock.clone();
/// let mut digipot = Ds3502::blocking_init(i2c, Default::default())?;
/// let wv = Wiper::try_from(88)?;
/// digipot.write_wiper(wv)?;
/// # i2c_mock.done();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Ds3502<I2C> {
    i2c: I2C,
    config: Config,
}

impl<I2C> Ds3502<I2C> {
    fn new(i2c: I2C, config: Config) -> Self {
        Ds3502 { i2c, config }
    }

    /// Get the driver's current control register mode.  
    pub fn mode(&self) -> ControlRegisterMode {
        self.config.mode
    }
}

impl<I2C: I2c> Ds3502<I2C> {
    /// Initialize a blocking DS3502 driver.
    ///
    /// The default [`Config`] disables writes to initial value register. See [Differences from default behavior](`Config#differences-from-default-behavior`) for more details.
    ///
    /// # Errors
    /// Will return `Err` on I2C bus related problems.
    pub fn blocking_init(i2c: I2C, config: Config) -> Result<Self, Ds3502Error> {
        let mut pot = Ds3502::new(i2c, config);
        pot.set_mode(config.mode)?;
        Ok(pot)
    }

    /// Changes control register mode to allow for saving the wiper value to the EEPROM.
    ///
    /// By default, the wiper value from `write_wiper` is not saved following reset.
    /// Using this function with `ControlRegisterMode::ReadWrite` will save the value passed to `write_wiper`
    /// into the EEPROM, changing the default value loaded upon reset.
    ///
    /// # Errors
    /// Will return `Err` on I2C bus related problems.
    fn set_mode(&mut self, mode: ControlRegisterMode) -> Result<(), Ds3502Error> {
        self.config.mode = mode;
        self.i2c
            .write(self.config.i2c_addr as u8, &[0x02, mode as u8])?;
        Ok(())
    }

    /// Sets the wiper value and saves it to the EEPROM.
    ///
    /// # Errors
    /// Will return `Err` on I2C bus related problems.
    pub fn write_and_save_wiper(&mut self, value: Wiper) -> Result<(), Ds3502Error> {
        self.set_mode(ControlRegisterMode::WiperAndInitialValue)?;
        self.write_wiper(value)?;
        self.set_mode(ControlRegisterMode::WiperOnly)?;
        Ok(())
    }

    /// Set digital potentiometer wiper to value.
    ///
    /// # Errors
    /// Will return `Err` on I2C bus related problems.
    pub fn write_wiper(&mut self, value: Wiper) -> Result<(), Ds3502Error> {
        self.i2c
            .write(self.config.i2c_addr as u8, &[0x00, value.0])?;
        Ok(())
    }
}

impl<I2C: AsyncI2c> Ds3502<I2C> {
    /// Initialize an async DS3502 driver.
    ///
    /// The default [`Config`] disables writes to initial value register. See [Differences from default behavior](`Config#differences-from-default-behavior`) for more details.
    ///
    /// # Errors
    /// Will return `Err` on I2C Bus problems.
    pub async fn async_init(i2c: I2C, config: Config) -> Result<Self, Ds3502Error> {
        let mut pot = Ds3502::new(i2c, config);
        pot.async_set_mode(config.mode).await?;
        Ok(pot)
    }

    /// Set wiper to value.
    ///
    /// # Errors
    /// Will return `Err` on I2C Bus problems.
    pub async fn async_write_wiper(&mut self, value: Wiper) -> Result<(), Ds3502Error> {
        self.i2c
            .write(self.config.i2c_addr as u8, &[0x00, value.0])
            .await?;
        Ok(())
    }

    /// Sets the wiper value and saves it to the EEPROM.
    ///
    /// # Errors
    /// Will return `Err` on I2C Bus problems.
    pub async fn async_write_and_save_wiper(&mut self, value: Wiper) -> Result<(), Ds3502Error> {
        self.async_set_mode(ControlRegisterMode::WiperAndInitialValue)
            .await?;
        self.async_write_wiper(value).await?;
        self.async_set_mode(ControlRegisterMode::WiperOnly).await?;
        Ok(())
    }

    /// Changes control register mode for wiper values (asynchronously).
    ///
    /// Read details from `set_control_register_mode` to learn more.
    ///
    /// # Errors
    /// Will return `Err` on I2C Bus problems.
    async fn async_set_mode(&mut self, mode: ControlRegisterMode) -> Result<(), Ds3502Error> {
        self.i2c
            .write(self.config.i2c_addr as u8, &[0x02, mode as u8])
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    use futures_lite::future::block_on;

    use super::*;

    extern crate std;

    use std::vec;

    fn run_blocking(i2c: Mock) -> Result<(), Ds3502Error> {
        let mut digipot = Ds3502::blocking_init(i2c, Default::default())?;
        let wv = Wiper::try_from(88)?;
        digipot.write_wiper(wv)?;
        let wv = Wiper::try_from(23)?;
        digipot.write_and_save_wiper(wv)?;
        Ok(())
    }

    async fn run_async(i2c: Mock) -> Result<(), Ds3502Error> {
        let mut digipot = Ds3502::async_init(i2c, Default::default()).await?;
        let wv = Wiper::try_from(88)?;
        digipot.async_write_wiper(wv).await?;
        let wv = Wiper::try_from(23)?;
        digipot.async_write_and_save_wiper(wv).await?;
        Ok(())
    }

    #[test]
    fn test_both() {
        let expectations = [
            // {blocking,async}_init
            Transaction::write(0x28, vec![0x2, ControlRegisterMode::WiperOnly as u8]),
            Transaction::write(0x28, vec![0x0, 88]),
            // {async_}write_and_save_wiper
            Transaction::write(
                0x28,
                vec![0x2, ControlRegisterMode::WiperAndInitialValue as u8],
            ),
            Transaction::write(0x28, vec![0x0, 23]),
            Transaction::write(0x28, vec![0x2, ControlRegisterMode::WiperOnly as u8]),
        ];
        let mut blocking_i2c = Mock::new(&expectations);
        let mut async_i2c = Mock::new(&expectations);

        run_blocking(blocking_i2c.clone()).unwrap();
        block_on(run_async(async_i2c.clone())).unwrap();

        assert_eq!(
            blocking_i2c.clone().collect::<vec::Vec<_>>(),
            async_i2c.clone().collect::<vec::Vec<_>>(),
        );

        blocking_i2c.done();
        async_i2c.done();
    }
}
