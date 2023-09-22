#![no_std]

use embedded_hal::blocking::i2c;
use bitflags::bitflags;
use core::fmt;

/// Library for the Texas instruments LM36011 inductorless LED driver
///
/// https://www.ti.com/lit/ds/symlink/lm36011.pdf?ts=1694461699965&ref_url=https%253A%252F%252Fwww.ti.com%252Fproduct%252FLM36011
///
/// This crate enables register only read / write, or complete register one-shot read/write based
/// on the last known register values and the bitflags crate for updating specific features.

/// Custom errors for the LM36011.
#[derive(Debug)]
pub enum LM36011Error<E> {
    I2CError(E),
    InvalidInput,
    CurrentOutOfRange,
    DeviceIDError,
}

/// Represents the configuration registers of the LM36011.
pub enum Register {
    /// Enable Register
    EnableRegister = 0x01,
    /// Configuration Register
    ConfigurationRegister = 0x02,
    /// LED Flash Brightness Register
    LEDFlashBrightnessRegister = 0x03,
    /// LED Torch Brightness Register
    LEDTorchBrightnessRegister = 0x04,
    /// Flags Register
    FlagsRegister = 0x05,
    /// Device ID Register
    DeviceIdRegister = 0x06,
}

/// implement display trait for Register Enum to be used in printing out to serial if needed
impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Register::EnableRegister => write!(f, "Enable Register"),
            Register::ConfigurationRegister => write!(f, "Configuration Register"),
            Register::LEDFlashBrightnessRegister => write!(f, "LED Flash Brightness Register"),
            Register::LEDTorchBrightnessRegister => write!(f, "LED Torch Brightness Register"),
            Register::FlagsRegister => write!(f, "Flags Register"),
            Register::DeviceIdRegister => write!(f, "Device ID Register"),
        }
    }
}

// Bitflags for the Enable Register (0x01)
bitflags! {
    pub struct EnableRegisterFlags: u8 {
        // Reserved for future use
        const ENABLE_REGISTER_RFU           = 0b1110_0000;
        // enables
        const IVFM_ENABLE                   = 0b0001_0000;
        const STROBE_TYPE_EDGE_TRIGGERED    = 0b0000_1000;
        const STROBE_ENABLE                 = 0b0000_0100;
        // mode settings
        const MODE_IR_DRIVE                 = 0b0000_0001;
        const MODE_TORCH                    = 0b0000_0010;
        const MODE_FLASH                    = 0b0000_0011;
        const MODE_MASK                     = 0b0000_0011;
    }
}

// Bitflags for the Configuration Register (0x02)
bitflags! {
    pub struct ConfigurationRegisterFlags: u8 {
        /// IVFM Levels (IVFM-D) [Bit 7-5]
        const IVFM_2_9V         = 0b0000_0000;
        const IVFM_3_0V         = 0b0010_0000;
        const IVFM_3_1V         = 0b0100_0000;
        const IVFM_3_2V         = 0b0110_0000;
        const IVFM_3_3V         = 0b1000_0000;
        const IVFM_3_4V         = 0b1010_0000;
        const IVFM_3_5V         = 0b1100_0000;
        const IVFM_3_6V         = 0b1110_0000;

        /// Flash Time-out Duration [Bit 4-1]
        const TIMEOUT_40MS      = 0b0000_0000;
        const TIMEOUT_80MS      = 0b0000_0010;
        const TIMEOUT_120MS     = 0b0000_0100;
        const TIMEOUT_160MS     = 0b0000_0110;
        const TIMEOUT_200MS     = 0b0000_1000;
        const TIMEOUT_240MS     = 0b0000_1010;
        const TIMEOUT_280MS     = 0b0000_1100;
        const TIMEOUT_320MS     = 0b0000_1110;
        const TIMEOUT_360MS     = 0b0001_0000;
        const TIMEOUT_400MS     = 0b0001_0010;
        const TIMEOUT_600MS     = 0b0001_0100;
        const TIMEOUT_800MS     = 0b0001_0110;
        const TIMEOUT_1000MS    = 0b0001_1000;
        const TIMEOUT_1200MS    = 0b0001_1010;
        const TIMEOUT_1400MS    = 0b0001_1100;
        const TIMEOUT_1600MS    = 0b0001_1110;

        /// Torch Ramp [Bit 0]
        const TORCH_RAMP_OFF    = 0b0000_0000;
        const TORCH_RAMP_1MS    = 0b0000_0001;
    }
}

// Bitflags for the LED Flash Brightness Register (0x03)
bitflags! {
    pub struct LedFlashBrightnessFlags: u8 {
        /// LED Flash Brightness Level [Bit 6:0]
        const FLASH_11MA    = 0x00;
        const FLASH_257MA   = 0x15;
        const FLASH_750MA   = 0x3F;
        const FLASH_1030MA  = 0x5F;
        const FLASH_1200MA  = 0x66;
        const FLASH_1500MA  = 0x7F;

        /// Thermal Current Scale-Back [Bit 7]
        const THERMAL_SCALEBACK_ENABLED = 0b1000_0000;
    }
}

// Bitflags for the LED Torch Brightness Register (0x04)
bitflags! {
    pub struct LedTorchBrightnessFlags: u8 {
        // Reserved for future use
        const TORCH_BRIGHTNESS_RFU  = 0b1000_0000;
        // Torch currents
        const TORCH_2_4MA           = 0x00;
        const TORCH_64MA            = 0x15;
        const TORCH_188MA           = 0x3F;
        const TORCH_258MA           = 0x5F;
        const TORCH_302MA           = 0x66;
        const TORCH_376MA           = 0x7F;
    }
}

// Bitflags for the Flags Register (0x05)
bitflags! {
    pub struct FlagRegisterFlags: u8 {
        // Reserved for future use
        const FLAGS_REGISTER_RFU            = 0b1000_0000;
        // Flags
        const IVFM_TRIP                     = 0b0100_0000;
        const VLED_SHORT_FAULT              = 0b0010_0000;
        const THERMAL_CURRENT_SCALE_BACK    = 0b0000_1000;
        const THERMAL_SHUTDOWN_FAULT        = 0b0000_0100;
        const UVLO_FAULT                    = 0b0000_0010;
        const FLASH_TIMEOUT_FLAG            = 0b0000_0001;
    }
}

// Bitflags for the Device ID Register (0x06)
bitflags! {
    /// Represents the Device ID and RESET Register of the LM36011.
    pub struct DeviceIdFlags: u8 {
        // Software RESET
        // 0 = Normal (default)
        // 1 = Force device RESET
        const SOFTWARE_RESET            = 0b1000_0000;

        // Reserved for Future Use
        const DEVICE_ID_RFU             = 0b0100_0000;

        // Device ID
        const DEVICE_ID_MASK            = 0b0011_1000;

        // Silicon Revision Bits
        const SILICON_REVISION_MASK     = 0b0000_0111;
    }
}

/// I2C address for the LM36011 device.
const LM36011_I2C_ADDRESS: u8 = 0x64;

/// Represents the LM36011 device with an associated I2C interface.
pub struct LM36011<I2C> {
    /// The I2C interface used to communicate with the device.
    i2c: I2C,
    pub enable_flags: EnableRegisterFlags,
    pub config_flags: ConfigurationRegisterFlags,
    pub flash_brightness_flags: LedFlashBrightnessFlags,
    pub torch_brightness_flags: LedTorchBrightnessFlags,
    pub flag_register_flags: FlagRegisterFlags,
    pub device_id: DeviceIdFlags,
}

impl<I2C> fmt::Display for LM36011<I2C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Enable Register: {:?}, \
            Configuration Register: {:?}, \
            LED Flash Brightness Register: {:?}, \
            LED Torch Brightness Register: {:?}, \
            Flags Register: {:?}, \
            Device ID Register: {:?}",
            self.enable_flags,
            self.config_flags,
            self.flash_brightness_flags,
            self.torch_brightness_flags,
            self.flag_register_flags,
            self.device_id
        )
    }
}

impl<I2C, E> LM36011<I2C>
    where
        I2C: i2c::Write<Error=E> + i2c::WriteRead<Error=E>,
{
    /// Creates a new instance of the LM36011 with the provided I2C interface.
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            enable_flags: EnableRegisterFlags::IVFM_ENABLE,
            config_flags: ConfigurationRegisterFlags::IVFM_2_9V |
                ConfigurationRegisterFlags::TIMEOUT_600MS |
                ConfigurationRegisterFlags::TORCH_RAMP_1MS,
            flash_brightness_flags: LedFlashBrightnessFlags::FLASH_11MA |
                LedFlashBrightnessFlags::THERMAL_SCALEBACK_ENABLED,
            torch_brightness_flags: LedTorchBrightnessFlags::TORCH_2_4MA,
            flag_register_flags: FlagRegisterFlags::empty(),
            device_id: DeviceIdFlags::empty(),
        }
    }

    /// Sets the flash current of the LM36011 device.
    ///
    /// This function configures the flash current of the LM36011 by writing to the
    /// `LEDFlashBrightnessRegister`. The desired current value is passed as an argument.
    ///
    /// # Arguments
    ///
    /// * `current` - The desired flash current value to be set. The exact range and interpretation
    ///               of this value should be based on the LM36011 documentation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(E)` if there was an error during the operation. The error type `E` is determined by the I2C interface.
    ///
    /// # Example
    ///
    /// ```
    /// // Some initialization to get the device instance
    /// //(I2C needs to be inititilized first)
    /// let mut driver = lm36011::LM36011::new(i2c);; // Some initialization to get the device instance
    /// match driver.set_flash_current(0x55) {
    ///     Ok(_) => println!("Flash current set successfully"),
    ///     Err(e) => eprintln!("Error setting flash current: {:?}", e),
    /// }
    /// ```
    pub fn set_flash_current_hex(&mut self, current: u8) -> Result<(), LM36011Error<E>> {
        if current > 0b1000_0000 {
            return Err(LM36011Error::CurrentOutOfRange);
        }

        // Use the set_register function to set the flash current
        self.set_register(Register::LEDFlashBrightnessRegister, current)
    }

    /// Sets the flash current of the LM36011 device.
    ///
    /// This function configures the flash current of the LM36011 by writing to the
    /// `LEDFlashBrightnessRegister`. The desired current value is passed as an argument.
    ///
    /// # Arguments
    ///
    /// * `current` - The desired flash current value to be set. The input current in mA will be
    /// divided by 11.7 and converted to a u8 byte.  Note: since the resolution of the driver is
    /// 11.7mA, setting fractions of the current is likly overkill, but could be more accurate in a
    /// very small subset of results.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful.
    /// * `Err(E)` if there was an error during the operation. The error type `E` is determined by the I2C interface.
    ///
    /// # Example
    ///
    /// ```
    /// // Some initialization to get the device instance
    /// //(I2C needs to be inititilized first)
    /// let mut driver = lm36011::LM36011::new(i2c);; // Some initialization to get the device instance
    /// match driver.set_flash_current(150.0) {
    ///     Ok(_) => println!("Flash current set successfully"),
    ///     Err(e) => eprintln!("Error setting flash current: {:?}", e),
    /// }
    /// ```
    pub fn set_flash_current(&mut self, current: f32) -> Result<(), LM36011Error<E>> {
        if current < 0.0 || current > 1500.0 {
            return Err(LM36011Error::CurrentOutOfRange);
        }
        // take in the current in mA (f32) and convert it to a hex value
        let brightness_flags: u8 = (current / 11.7) as u8;

        // convert the u8 value to a LedFlashBrightnessFlags
        let mut brightness_bitflags =
            LedFlashBrightnessFlags::from_bits_truncate(brightness_flags);

        // Ensure the thermal current scale-back bit remains set/not set
        brightness_bitflags.set(
            LedFlashBrightnessFlags::THERMAL_SCALEBACK_ENABLED,
            self.flash_brightness_flags.contains(
                LedFlashBrightnessFlags::THERMAL_SCALEBACK_ENABLED),
        );

        // Use the set_register function to set the flash current
        self.set_register(Register::LEDFlashBrightnessRegister, brightness_flags)?;

        // update internal struct state
        self.flash_brightness_flags = brightness_bitflags;

        Ok(())
    }

    /// Retrieves the device ID from the LM36011.
    ///
    /// This function reads the `DeviceIdRegister` of the LM36011 device to obtain its ID.
    /// It uses the I2C `write_read` method to request and retrieve the device ID.
    ///
    /// # Returns
    ///
    /// * `Ok(u8)` containing the device ID if the read operation was successful.
    /// * `Err(E)` if there was an error during the read operation. The error type `E` is determined by the I2C interface.
    ///
    /// # Example
    ///
    /// ```
    /// // Some initialization to get the device instance
    /// //(I2C needs to be inititilized first)
    /// let mut driver = lm36011::LM36011::new(i2c);; // Some initialization to get the device instance
    /// match driver.get_device_id() {
    ///     Ok(id) => println!("LM36011 device ID: {}", id),
    ///     Err(e) => eprintln!("Error reading device ID: {:?}", e),
    /// }
    /// ```
    pub fn get_device_id(&mut self) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(LM36011_I2C_ADDRESS, &[Register::DeviceIdRegister as u8],
                            &mut buffer)?;
        Ok(buffer[0])
    }

    /// Retrieves the value of a specified register from the device.
    ///
    /// This function reads a byte of data from a specified register on the LM36011 device.
    /// It uses the I2C `write_read` method to request and retrieve the data.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register from which the data should be read. This is specified using the `Register` enum.
    ///
    /// # Returns
    ///
    /// * `Ok(u8)` containing the byte value read from the specified register if the read operation was successful.
    /// * `Err(E)` if there was an error during the read operation. The error type `E` is determined by the I2C interface.
    ///
    /// # Example
    ///
    /// ```
    /// // Some initialization to get the device instance
    /// //(I2C needs to be inititilized first)
    /// let mut driver = lm36011::LM36011::new(i2c);
    /// match driver.get_register(Register::DeviceIdRegister) {
    ///     Ok(value) => println!("Register value: {}", value),
    ///     Err(e) => eprintln!("Error reading register: {:?}", e),
    /// }
    /// ```
    pub fn get_register(&mut self, reg: Register) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(LM36011_I2C_ADDRESS, &[reg as u8], &mut buffer)?;
        Ok(buffer[0])
    }

    /// Sets the value of a specified register on the device.
    ///
    /// This function writes a given data byte to a specified register on the LM36011 device.
    /// It uses the I2C `write` method to send the data.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register to which the data should be written. This is specified using the `Register` enum.
    /// * `data` - The data byte to be written to the specified register.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the write operation was successful.
    /// * `Err(E)` if there was an error during the write operation. The error type `E` is determined by the I2C interface.
    ///
    /// # Example
    ///
    /// ```
    /// // Some initialization to get the device instance
    /// //(I2C needs to be inititilized first)
    /// let mut driver = lm36011::LM36011::new(i2c);
    /// let result = driver.set_register(Register::DeviceIdRegister, 0x01);
    /// if result.is_err() {
    ///     // Handle the error
    /// }
    /// ```
    pub fn set_register(&mut self, reg: Register, data: u8) -> Result<(), LM36011Error<E>> {
        let buffer: [u8; 2] = [reg as u8, data];
        self.i2c.write(LM36011_I2C_ADDRESS, &buffer).
            map_err(LM36011Error::I2CError)
    }

    /// Reads all the registers of the LM36011 and saves the register states to the respective bitflag structs.
    ///
    /// This function performs a single I2C read operation starting from the `EnableRegister` and reads 6 bytes,
    /// which correspond to the 6 registers of the LM36011. The read values are then saved to the respective
    /// bitflag structs for easy access and manipulation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the I2C read operation is successful.
    /// * `Err(E)` if the I2C read operation fails, where `E` is the error type of the I2C operations.
    ///
    /// # Usage
    ///
    /// ```rust
    /// let mut driver = LM36011::new(i2c_instance);
    ///
    /// if let Err(e) = driver.read_status() {
    ///     // Handle the error `e` here.
    /// }
    /// ```
    pub fn read_status(&mut self) -> Result<(), LM36011Error<E>> {
        // Read all 6 LM36011 registers
        let mut buffer = [0u8; 6];
        self.i2c.write_read(LM36011_I2C_ADDRESS,
                            &[Register::EnableRegister as u8], &mut buffer).
            map_err(LM36011Error::I2CError)?;

        // Save registers to the struct
        self.enable_flags = EnableRegisterFlags::from_bits_truncate(buffer[0]);
        self.config_flags = ConfigurationRegisterFlags::from_bits_truncate(buffer[1]);
        self.flash_brightness_flags = LedFlashBrightnessFlags::from_bits_truncate(buffer[2]);
        self.torch_brightness_flags = LedTorchBrightnessFlags::from_bits_truncate(buffer[3]);
        self.flag_register_flags = FlagRegisterFlags::from_bits_truncate(buffer[4]);
        self.device_id = DeviceIdFlags::from_bits_truncate(buffer[5]);

        Ok(())
    }

    /// Writes the bitflags settings to the LM36011 device.
    ///
    /// This function will take the current settings stored in the bitflag structs and write them to the
    /// respective registers on the LM36011 device using I2C.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming `i2c` is an initialized I2C instance`
    /// let mut driver = LM36011::new(i2c_instance);
    /// // Modify some settings
    /// driver.enable_flags.insert(EnableRegisterFlags::MODE_TORCH);
    /// driver.config_flags.insert(ConfigurationRegisterFlags::IVFM_3_4V);
    ///
    /// // Write the modified settings to the device
    /// match lm36011.write_status() {
    ///     Ok(_) => println!("Settings written successfully!"),
    ///     Err(e) => println!("Failed to write settings: {:?}", e),
    /// }
    /// ```
    ///
    pub fn write_status(&mut self) -> Result<(), LM36011Error<E>> {
        // create a buffer with all of the settings
        let buffer = [0x01,
            self.enable_flags.bits(),
            self.config_flags.bits(),
            self.flash_brightness_flags.bits(),
            self.torch_brightness_flags.bits(),
            //self.flag_register_flags.bits(),
            //self.device_id.bits(),
        ];

        self.i2c.write(LM36011_I2C_ADDRESS, &buffer)
            .map_err(LM36011Error::I2CError)
    }

    /// Performs a software reset on the LM36011 device.
    ///
    /// This function sends a specific command to the LM36011 device to initiate a software reset.
    /// The reset command is sent to the address `0x06` with the data `0b1000_0000`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut device = LM36011::new(i2c_instance);
    ///
    /// match device.software_reset() {
    ///     Ok(_) => println!("Software reset successful!"),
    ///     Err(e) => println!("Software reset failed with error: {:?}", e),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an `Err` variant of `LM36011Error` if there's an I2C communication error.

    pub fn software_reset(&mut self) -> Result<(), LM36011Error<E>> {
        let buffer = [0x06,0b1000_0000];
        self.i2c.write(LM36011_I2C_ADDRESS, &buffer)
            .map_err(LM36011Error::I2CError)
    }

    /// Verifies the device ID of the LM36011.
    ///
    /// This function reads the current status of the LM36011, including its device ID,
    /// and then checks if the silicon revision mask matches the expected value.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the device ID matches the expected value.
    /// * `Err(LM36011Error::InvalidInput)` if the device ID does not match the expected value.
    /// * `Err(LM36011Error::I2CError(E))` if there's an error during the I2C communication.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut driver = LM36011::new(i2c);
    /// match driver.verify_device_id() {
    ///     Ok(true) => println!("Device ID verified!"),
    ///     Err(LM36011Error::DeviceIDError) => println!("Device ID does not match!"),
    ///     Err(LM36011Error::I2CError(_)) => println!("Error verifying device ID due to I2C communication"),
    ///     _ => println!("Some other error occurred"),
    /// }
    /// ```
    pub fn verify_device_id(&mut self) -> Result<bool, LM36011Error<E>> {
        match self.read_status() {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Check if the read value matches the expected device ID
        if self.device_id & DeviceIdFlags::SILICON_REVISION_MASK ==
            DeviceIdFlags::from_bits_truncate(0x01) {
            Ok(true)
        } else {
            Err(LM36011Error::DeviceIDError)
        }
    }
// similarly, you can add other methods with detailed documentation.
}