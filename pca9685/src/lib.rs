#![no_std]

//! Rust conversion of ServoDriver for esp-hal.
//!
//! This keeps the same register behavior and method logic as the C++ version,
//! but uses a raw `*mut` pointer to a blocking `esp_hal` I2C driver.
//!
//! The I2C type is expected to implement `embedded_hal::i2c::I2c`.
//! The only esp-hal-specific piece here is the delay helper.

use core::marker::PhantomData;

use esp_hal::DriverMode;
// use esp_hal::delay::DelayNs;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;

// REGISTER ADDRESSES
pub const PCA9685_MODE1: u8 = 0x00;
pub const PCA9685_MODE2: u8 = 0x01;
pub const PCA9685_SUBADR1: u8 = 0x02;
pub const PCA9685_SUBADR2: u8 = 0x03;
pub const PCA9685_SUBADR3: u8 = 0x04;
pub const PCA9685_ALLCALLADR: u8 = 0x05;
pub const PCA9685_LED0_ON_L: u8 = 0x06;
pub const PCA9685_LED0_ON_H: u8 = 0x07;
pub const PCA9685_LED0_OFF_L: u8 = 0x08;
pub const PCA9685_LED0_OFF_H: u8 = 0x09;
pub const PCA9685_ALLLED_ON_L: u8 = 0xFA;
pub const PCA9685_ALLLED_ON_H: u8 = 0xFB;
pub const PCA9685_ALLLED_OFF_L: u8 = 0xFC;
pub const PCA9685_ALLLED_OFF_H: u8 = 0xFD;
pub const PCA9685_PRESCALE: u8 = 0xFE;
pub const PCA9685_TESTMODE: u8 = 0xFF;

// MODE1 bits
pub const MODE1_ALLCAL: u8 = 0x01;
pub const MODE1_SUB3: u8 = 0x02;
pub const MODE1_SUB2: u8 = 0x04;
pub const MODE1_SUB1: u8 = 0x08;
pub const MODE1_SLEEP: u8 = 0x10;
pub const MODE1_AI: u8 = 0x20;
pub const MODE1_EXTCLK: u8 = 0x40;
pub const MODE1_RESTART: u8 = 0x80;

// MODE2 bits
pub const MODE2_OUTNE_0: u8 = 0x01;
pub const MODE2_OUTNE_1: u8 = 0x02;
pub const MODE2_OUTDRV: u8 = 0x04;
pub const MODE2_OCH: u8 = 0x08;
pub const MODE2_INVRT: u8 = 0x10;

pub const PCA9685_I2C_ADDRESS: u8 = 0x40;
pub const FREQUENCY_OSCILLATOR: u32 = 25_000_000;

pub const PCA9685_PRESCALE_MIN: u8 = 3;
pub const PCA9685_PRESCALE_MAX: u8 = 255;

#[derive(PartialEq)]
pub enum OutputMode {
    TotemPole,
    OpenDrain,
}

/// Class that stores state and functions for interacting with a PCA9685 PWM chip.
#[allow(non_camel_case_types)]
pub struct ServoDriver<'b, D: DriverMode> {
    i2c: *mut I2c<'b, D>,
    i2c_owned: PhantomData<&'b ()>,
    i2caddr: u8,
    oscillator_freq: u32,
    pub pwm_freq: f32,
}

impl<'b, D: DriverMode> ServoDriver<'b, D> {
    /// Instantiates a new PCA9685 PWM driver chip with the default I2C address.
    pub fn new(i2c: &I2c<'b, D>) -> Self {
        Self::with_address(PCA9685_I2C_ADDRESS, i2c)
    }

    /// Instantiates a new PCA9685 PWM driver chip with a custom I2C address.
    pub fn with_address(addr: u8, i2c: &I2c<'b, D>) -> Self {
        Self {
            i2caddr: addr,
            i2c: i2c as *const _ as *mut I2c<'b, D>,
            i2c_owned: PhantomData,
            oscillator_freq: 0,
            pwm_freq: 1000.0,
        }
    }

    /// Instantiates a new PCA9685 PWM driver chip from a raw I2C pointer.
    pub fn from_raw(addr: u8, i2c: *mut I2c<'b, D>) -> Self {
        Self {
            i2caddr: addr,
            i2c,
            i2c_owned: PhantomData,
            oscillator_freq: 0,
            pwm_freq: 1000.0,
        }
    }

    /// Setups the I2C interface and hardware.
    ///
    /// If `prescale` is non-zero, the chip is configured to use that prescale
    /// value as an external clock prescaler. Otherwise the default frequency is
    /// set to 1000 Hz.
    pub fn begin(&mut self, prescale: u8) -> bool {
        if self.i2c.is_null() {
            return false;
        }

        self.reset();
        self.set_oscillator_frequency(FREQUENCY_OSCILLATOR);

        if prescale != 0 {
            self.set_ext_clk(prescale);
        } else {
            self.set_pwm_freq(1000.0);
        }

        true
    }

    /// Sends a reset command to the PCA9685 chip over I2C.
    pub fn reset(&mut self) {
        self.write8(PCA9685_MODE1, MODE1_RESTART);
        Self::delay_ms(10);
    }

    /// Puts board into sleep mode.
    pub fn sleep(&mut self) {
        let awake = self.read8(PCA9685_MODE1);
        let sleep = awake | MODE1_SLEEP;
        self.write8(PCA9685_MODE1, sleep);
        Self::delay_ms(5);
    }

    /// Wakes board from sleep.
    pub fn wakeup(&mut self) {
        let sleep = self.read8(PCA9685_MODE1);
        let wakeup = sleep & !MODE1_SLEEP;
        self.write8(PCA9685_MODE1, wakeup);
    }

    /// Sets EXTCLK pin to use the external clock.
    pub fn set_ext_clk(&mut self, prescale: u8) {
        let oldmode = self.read8(PCA9685_MODE1);
        let mut newmode = (oldmode & !MODE1_RESTART) | MODE1_SLEEP;
        self.write8(PCA9685_MODE1, newmode);

        // This sets both the SLEEP and EXTCLK bits of MODE1 to switch to the
        // external clock.
        newmode |= MODE1_EXTCLK;
        self.write8(PCA9685_MODE1, newmode);

        self.write8(PCA9685_PRESCALE, prescale);

        Self::delay_ms(5);

        // Clear SLEEP and set RESTART + AI.
        self.write8(
            PCA9685_MODE1,
            (newmode & !MODE1_SLEEP) | MODE1_RESTART | MODE1_AI,
        );
    }

    /// Sets the PWM frequency for the entire chip, up to ~1.6 KHz.
    pub fn set_pwm_freq(&mut self, mut freq: f32) {
        if freq < 1.0 {
            freq = 1.0;
        }
        if freq > 3500.0 {
            freq = 3500.0;
        }
        self.pwm_freq = freq;

        let mut prescaleval = ((self.oscillator_freq as f32 / (freq * 4096.0)) + 0.5) - 1.0;

        if prescaleval < PCA9685_PRESCALE_MIN as f32 {
            prescaleval = PCA9685_PRESCALE_MIN as f32;
        }
        if prescaleval > PCA9685_PRESCALE_MAX as f32 {
            prescaleval = PCA9685_PRESCALE_MAX as f32;
        }

        let prescale = prescaleval as u8;

        let oldmode = self.read8(PCA9685_MODE1);
        let newmode = (oldmode & !MODE1_RESTART) | MODE1_SLEEP;

        self.write8(PCA9685_MODE1, newmode);
        self.write8(PCA9685_PRESCALE, prescale);
        self.write8(PCA9685_MODE1, oldmode);

        Self::delay_ms(5);

        // Turn on auto-increment and restart, just like the original C++ code.
        self.write8(PCA9685_MODE1, oldmode | MODE1_RESTART | MODE1_AI);
    }

    /// Sets the output mode of the PCA9685 to either open drain or totem pole.
    pub fn set_output_mode(&mut self, mode: OutputMode) {
        let oldmode = self.read8(PCA9685_MODE2);
        let newmode = if mode == OutputMode::TotemPole {
            oldmode | MODE2_OUTDRV
        } else {
            oldmode & !MODE2_OUTDRV
        };
        self.write8(PCA9685_MODE2, newmode);
    }

    /// Reads set Prescale from PCA9685.
    pub fn read_prescale(&mut self) -> u8 {
        self.read8(PCA9685_PRESCALE)
    }

    /// Gets the PWM output of one of the PCA9685 pins.
    ///
    /// If `off` is true, returns the OFF value; otherwise returns the ON value.
    pub fn get_pwm(&mut self, num: u8, off: bool) -> u16 {
        let mut buffer = [0u8, 0u8];
        let mut reg = PCA9685_LED0_ON_L.wrapping_add(4 * num);
        if off {
            reg = reg.wrapping_add(2);
        }

        if !self.i2c.is_null() {
            let _ = self.i2c_mut().write_read(self.i2caddr, &[reg], &mut buffer);
        }

        u16::from(buffer[0]) | (u16::from(buffer[1]) << 8)
    }

    /// Sets the PWM output of one of the PCA9685 pins.
    ///
    /// Returns `0` if successful, otherwise `1`.
    pub fn set_pwm(&mut self, num: u8, on: u16, off: u16) -> u8 {
        let buffer = [
            PCA9685_LED0_ON_L.wrapping_add(4 * num),
            on as u8,
            (on >> 8) as u8,
            off as u8,
            (off >> 8) as u8,
        ];

        if self.i2c.is_null() {
            return 1;
        }

        let result = self.i2c_mut().write(self.i2caddr, &buffer);
        if result.is_ok() { 0 } else { 1 }
    }

    /// Helper to set pin PWM output.
    ///
    /// Sets pin without dealing with on/off tick placement and properly handles
    /// a zero value as completely off and 4095 as completely on.
    pub fn set_pin(&mut self, num: u8, mut val: u16, invert: bool) {
        if val > 4095 {
            val = 4095;
        }

        if invert {
            if val == 0 {
                self.set_pwm(num, 4096, 0);
            } else if val == 4095 {
                self.set_pwm(num, 0, 4096);
            } else {
                self.set_pwm(num, 0, 4095 - val);
            }
        } else if val == 4095 {
            self.set_pwm(num, 4096, 0);
        } else if val == 0 {
            self.set_pwm(num, 0, 4096);
        } else {
            self.set_pwm(num, 0, val);
        }
    }

    /// Sets the PWM output of one of the PCA9685 pins based on input microseconds.
    ///
    /// The output is not precise, matching the original implementation.
    pub fn write_microseconds(&mut self, num: u8, microseconds: u16) {
        let pulse = microseconds as f64;

        // Read prescale.
        let mut prescale = self.read_prescale() as u16;

        // Calculate pulse length for PWM based on the datasheet formula.
        prescale += 1;
        let mut pulselength = 1_000_000.0f64;
        pulselength *= prescale as f64;
        pulselength /= self.oscillator_freq as f64;

        let pulse = pulse / pulselength;
        self.set_pwm(num, 0, pulse as u16);
    }

    /// Getter for the internally tracked oscillator used for frequency calculations.
    pub fn get_oscillator_frequency(&self) -> u32 {
        self.oscillator_freq
    }

    /// Setter for the internally tracked oscillator used for frequency calculations.
    pub fn set_oscillator_frequency(&mut self, freq: u32) {
        self.oscillator_freq = freq;
    }

    /// Reads a single PCA9685 register through the I2C driver.
    fn read8(&mut self, addr: u8) -> u8 {
        let mut out = [0u8; 1];
        if self.i2c.is_null() {
            return 0;
        }

        let _ = self.i2c_mut().write_read(self.i2caddr, &[addr], &mut out);
        out[0]
    }

    /// Writes a single PCA9685 register through the I2C driver.
    fn write8(&mut self, addr: u8, d: u8) {
        if self.i2c.is_null() {
            return;
        }

        let buf = [addr, d];
        let _ = self.i2c_mut().write(self.i2caddr, &buf);
    }

    /// Delays for a number of milliseconds using esp-hal's blocking delay type.
    fn delay_ms(ms: u32) {
        let delay = Delay::new();
        delay.delay_millis(ms);
    }

    // #[allow(unsafe_op_in_unsafe_fn)]
    /// Converts the stored raw pointer into a mutable reference.
    ///
    /// This is unsafe because the caller must guarantee the I2C driver remains
    /// valid and uniquely borrowed for the lifetime of this object.
    fn i2c_mut(&self) -> &mut I2c<'b, D> {
        unsafe { &mut *self.i2c }
    }
}

