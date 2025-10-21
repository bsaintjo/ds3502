<!-- cargo-rdme start -->

# I2C Digitial Potentiometer DS3502 driver in Rust

![Crates.io License](https://img.shields.io/crates/l/ds3502)
![Crates.io Version](https://img.shields.io/crates/v/ds3502)
![docs.rs](https://img.shields.io/docsrs/ds3502)
![Crates.io MSRV](https://img.shields.io/crates/msrv/ds3502)
![embedded-hal Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbsaintjo%2Fds3502%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=%24.dependencies.embedded-hal&label=embedded-hal)

Rust embedded driver for the DS3502 Digital Potentiometer by Analog Devices Inc./Maxim Integrated. Supports block and async APIs.

## Usage

```rust
use ds3502::{Ds3502, Wiper, ControlRegisterMode};
let mut digipot = Ds3502::blocking_init(i2c, Default::default())?;

// By default, driver initialized with Control Register in Mode 1 (Don't save to EEPROM)
assert_eq!(digipot.mode(), ControlRegisterMode::WiperOnly);

let wv = Wiper::try_from(88)?;
digipot.write_wiper(wv)?;

// Set wiper value and save to EEPROM
let wv = Wiper::try_from(123)?;
digipot.write_and_save_wiper(wv);
```

[Complete example with the Raspberry Pi Pico and `embassy`](http://github.com/bsaintjo/ds3502/blob/main/examples/pico-example/)

## Installation

Add to your `Cargo.toml`:

```toml
ds3502 = "0.1"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](http://www.github.com/bsaintjo/ds3502/blob/HEAD/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](http://www.github.com/bsaintjo/ds3502/blob/HEAD/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

<!-- cargo-rdme end -->
