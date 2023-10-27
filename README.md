# LM36011 Rust Driver

This crate provides a Rust driver for the Texas Instruments LM36011 inductorless LED driver.

## Usage

Add this to your `Cargo.toml` file:

```toml
[dependencies]
lm36011 = "0.1.0"
```
# Examples

```rust
use lm36011::LM36011;
// Assume create_i2c_device is a function that initializes and returns an I2C device
use your_hal_crate::create_i2c_device;

fn main() {
    let i2c = create_i2c_device();
    let mut driver = LM36011::new(i2c);
    match driver.set_flash_current(150.0) {
        Ok(_) => println!("Flash current set successfully"),
        Err(e) => eprintln!("Error setting flash current: {:?}", e),
    }
}
```

# Documentation

The API documentation can be built with cargo doc or [viewed online](https://docs.rs/lm36011/).

# License

This project is licensed under the MIT License - see the LICENSE file for details.

# Contributing

Pull requests are welcome.

# Acknowledgements

ChatGPT 4 assisted in the creation of parts of this code and documentation.  It's great for learning new skills!