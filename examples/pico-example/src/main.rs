//! Example code that loops through all wiper positions (0 ..=127) and
//! prints the ADC value, wiper position, and voltage.
//!
//! Based on the i2c example from the embassy framework.
#![no_std]
#![no_main]

use ds3502::{Ds3502, Wiper};
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{Adc, Channel},
    gpio::Pull,
    i2c,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let adc_pin = p.PIN_26;

    // Default I2C config enables internal pull-up resistors.
    let i2c = i2c::I2c::new_blocking(p.I2C0, scl, sda, Default::default());
    let mut digitpot = Ds3502::blocking_init(i2c, Default::default()).unwrap();

    let mut channel = Channel::new_pin(adc_pin, Pull::None);
    let mut adc = Adc::new_blocking(p.ADC, Default::default());

    for value in (0u8..128).cycle() {
        let wiper = Wiper::try_from(value).unwrap();
        digitpot.write_wiper(wiper).unwrap();
        Timer::after_millis(100).await;
        if let Ok(val) = adc.blocking_read(&mut channel) {
            let voltage = (val as f32 * 3.3) / 4096.;
            defmt::info!(
                "ADC Value: {=u16}, Wiper Value: {=u8}, Voltage: {=f32}",
                val,
                wiper.inner(),
                voltage
            );
            Timer::after_secs(1).await;
        }
    }
}
