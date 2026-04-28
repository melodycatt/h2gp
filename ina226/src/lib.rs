#![no_std]

//
// yes a tiny portion of this is chatgpt, cry about it
// by a tiny portion i mean maybe one function
//

use core::marker::PhantomData;

use esp_hal::{
    DriverMode,
    delay::{self, Delay},
    i2c::master::{Error, I2c, I2cAddress},
    time::Instant,
};

/// number of samples to average values over (i think)
#[derive(Clone, Copy)]
pub enum AverageMode {
    Avg1 = 0x0000,
    Avg4 = 0x0200,
    Avg16 = 0x0400,
    Avg64 = 0x0600,
    Avg128 = 0x0800,
    Avg256 = 0x0A00,
    Avg512 = 0x0C00,
    Avg1024 = 0x0E00,
}

/// conversion time in microseconds
/// (dont know what this means tbh) (check the datasheet)
#[derive(Clone, Copy)]
pub enum ConversionTime {
    Us140 = 0b00000000,
    Us204 = 0b00000001,
    Us332 = 0b00000010,
    Us588 = 0b00000011,
    Us1100 = 0b00000100,
    Us2116 = 0b00000101,
    Us4156 = 0b00000110,
    Us8244 = 0b00000111,
}

/// really dont know what this means, read the datasheet
/// although looking at the variants its mostly self explanatory
#[derive(Clone, Copy)]
pub enum MeasureMode {
    PowerDown,
    TriggeredCurrentOnly,
    TriggererdBusOnly,
    Triggered,
    PowerDown2,
    ContinuousCurrentOnly,
    ContinuousBusOnly,
    Continuous,
}

/// i dont really know, i think theres some kind of alert interrupt thingy going on?
#[derive(Clone, Copy)]
pub enum AlertType {
    ShuntOver = 0x8000,
    ShuntUnder = 0x4000,
    BusOver = 0x2000,
    BusUnder = 0x1000,
    PowerOver = 0x0800,
    CurrentOver = 0xFFFE,
    CurrentUnder = 0xFFFF,
    None = 0x0000,
}

/// INA226 driver
/// preferably have a clock speed from 1..400khz
/// unless in high speed mode but idk how to enable it so contribute to the library if you want
/// that
pub struct Ina226<'b, D: DriverMode> {
    wire: *mut I2c<'b, D>,
    _phantom: PhantomData<&'b ()>,
    averages: AverageMode,
    conv_time: ConversionTime,
    measure_mode: MeasureMode,
    alert_type: AlertType,
    i2c_address: I2cAddress,
    cal_val: u16,
    corr_factor: f32,
    conf_reg_copy: u16,
    current_divider_ma: f32,
    pwr_multiplier_mw: f32,
    /// was there a overflow in the measurements
    pub overflow: bool,
    /// not really sure how the alerts work but its on the datasheet i just didnt read it
    pub conv_alert: bool,
    /// not really sure how the alerts work but its on the datasheet i just didnt read it
    pub limit_alert: bool,
}

impl<'b, D: DriverMode> Ina226<'b, D> {
    /// i2c address, im pretty sure
    pub const ADDRESS: u8 = 0x40;
    /// Configuration Register
    pub const CONF_REG: u8 = 0x00;
    /// Shunt Voltage Register
    pub const SHUNT_REG: u8 = 0x01;
    /// Bus Voltage Register
    pub const BUS_REG: u8 = 0x02;
    /// Power Register
    pub const PWR_REG: u8 = 0x03;
    /// Current flowing through Shunt
    pub const CURRENT_REG: u8 = 0x04;
    /// Calibration Register
    pub const CAL_REG: u8 = 0x05;
    /// Mask/Enable Register
    pub const MASK_EN_REG: u8 = 0x06;
    /// Alert Limit Register
    pub const ALERT_LIMIT_REG: u8 = 0x07;
    /// Contains Unique Manbufacturer Identification Number
    pub const MAN_ID_REG: u8 = 0xFE;
    /// Contains unique ID
    pub const ID_REG: u8 = 0xFF;

    /// Reset
    pub const RST: u16 = 0x8000;
    /// Alert function flag
    pub const AFF: u16 = 0x0010;
    /// Conversion ready flag
    pub const CVRF: u16 = 0x0008;
    /// Overflow flags
    pub const OVF: u16 = 0x0004;
    /// Alert pin polarity - if set then active-high
    pub const ALERT_POL: u16 = 0x0002;
    /// Latch enable - if set then alert flag remains until mask/enable register is read
    /// if not set then flag is cleared after next conversion within limits
    pub const LATCH_EN: u16 = 0x0001;

    #[allow(clippy::mut_from_ref)]
    fn i2c(&self) -> &mut I2c<'b, D> {
        unsafe { &mut *self.wire }
    }
    /// create a new instance
    /// **IMPORTANT** this does not initialize the ina226 or set any config, you must use `Ina226::init()`
    pub fn new(wire: &'b I2c<'b, D>, addr: u8) -> Self {
        Self {
            wire: wire as *const _ as *mut I2c<'b, D>,
            _phantom: PhantomData,
            i2c_address: I2cAddress::SevenBit(addr),
            averages: AverageMode::Avg4,
            conv_time: ConversionTime::Us1100,
            measure_mode: MeasureMode::Continuous,
            alert_type: AlertType::None,
            cal_val: 2048,
            corr_factor: 1.0,
            conf_reg_copy: 0,
            current_divider_ma: 40.0,
            pwr_multiplier_mw: 0.625,
            overflow: false,
            conv_alert: false,
            limit_alert: false,
        }
    }

    /// shouldnt be needed? but it doesnt matter
    /// create a new instance linked to an already configured ina226
    /// syncs config
    pub fn new_initialized(wire: &'b I2c<'b, D>, addr: u8) -> Result<Self, Error> {
        let mut dev = Self::new(wire, addr);
        dev.sync_from_hardware()?;
        Ok(dev)
    }

    fn sync_from_hardware(&mut self) -> Result<(), Error> {
        // Read back config
        let conf = self.read_register(Self::CONF_REG)?;
        self.conf_reg_copy = conf;

        // Extract fields
        self.averages = match conf & 0x0E00 {
            0x0000 => AverageMode::Avg1,
            0x0200 => AverageMode::Avg4,
            0x0400 => AverageMode::Avg16,
            0x0600 => AverageMode::Avg64,
            0x0800 => AverageMode::Avg128,
            0x0A00 => AverageMode::Avg256,
            0x0C00 => AverageMode::Avg512,
            _ => AverageMode::Avg1024,
        };

        self.conv_time = match (conf >> 3) & 0x07 {
            0 => ConversionTime::Us140,
            1 => ConversionTime::Us204,
            2 => ConversionTime::Us332,
            3 => ConversionTime::Us588,
            4 => ConversionTime::Us1100,
            5 => ConversionTime::Us2116,
            6 => ConversionTime::Us4156,
            _ => ConversionTime::Us8244,
        };

        self.measure_mode = match conf & 0x0007 {
            0 => MeasureMode::PowerDown,
            1 => MeasureMode::TriggeredCurrentOnly,
            2 => MeasureMode::TriggererdBusOnly,
            3 => MeasureMode::Triggered,
            5 => MeasureMode::ContinuousCurrentOnly,
            6 => MeasureMode::ContinuousBusOnly,
            _ => MeasureMode::Continuous,
        };

        // Read calibration and reconstruct math
        self.cal_val = self.read_register(Self::CAL_REG)?;

        // Reverse datasheet formulas
        let current_lsb = 0.00512 / (self.cal_val as f32 * self.corr_factor);
        self.current_divider_ma = 0.001 / current_lsb;
        self.pwr_multiplier_mw = 25000.0 * current_lsb;

        Ok(())
    }

    fn write_register(&mut self, reg: u8, val: u16) -> Result<(), Error> {
        let bytes = [reg, (val >> 8) as u8, (val & 0xFF) as u8];
        self.i2c().write(self.i2c_address, &bytes)
    }
    fn read_register(&mut self, reg: u8) -> Result<u16, Error> {
        let mut buf = [0u8, 0u8];
        self.i2c().write_read(self.i2c_address, &[reg], &mut buf)?;
        Ok(((buf[0] as u16) << 8) + buf[1] as u16)
    }

    /// initialize ina226 with current config
    pub fn init(&mut self) -> Result<(), Error> {
        let delay = delay::Delay::new();

        self.reset_ic()?;
        delay.delay_millis(1000);
        self.write_register(Self::CAL_REG, 2048)?;
        let mut current_config = self.read_register(Self::CONF_REG)?;
        current_config &= !(0x0E00);
        current_config |= self.averages as u16;
        current_config &= !(0x01C0);
        current_config &= !(0x0038);
        current_config |= (self.conv_time as u16) << 3;
        current_config |= (self.conv_time as u16) << 6;
        current_config &= !(0x0007);
        current_config |= self.measure_mode as u16;
        self.write_register(Self::CONF_REG, current_config)
    }

    /// reset
    pub fn reset_ic(&mut self) -> Result<(), Error> {
        //esp_println::println!("resetting ina226");
        self.write_register(Self::CONF_REG, Self::RST)?;

        //delay.delay_millis(1000);
        Ok(())
    }

    /// dont know, see datasheet
    pub fn set_correction_factor(&mut self, corr_factor: f32) -> Result<(), Error> {
        self.corr_factor = corr_factor;
        self.write_register(Self::CAL_REG, (self.cal_val as f32 * corr_factor) as u16)
    }
    /// see `AverageMode`
    pub fn set_average_mode(&mut self, averages: AverageMode) -> Result<(), Error> {
        self.averages = averages;
        let mut current_config = self.read_register(Self::CONF_REG)?;
        current_config &= !(0x0E00);
        current_config |= self.averages as u16;
        self.write_register(Self::CONF_REG, current_config)
    }
    /// dont know, see datasheet
    pub fn set_conversion_time(&mut self, conv_time: ConversionTime) -> Result<(), Error> {
        self.conv_time = conv_time;
        let mut current_config = self.read_register(Self::CONF_REG)?;
        current_config &= !(0x01C0);
        current_config &= !(0x0038);
        current_config |= (self.conv_time as u16) << 3;
        current_config |= (self.conv_time as u16) << 6;
        self.write_register(Self::CONF_REG, current_config)
    }
    /// see `MeasureMode`
    pub fn set_measure_mode(&mut self, measure_mode: MeasureMode) -> Result<(), Error> {
        self.measure_mode = measure_mode;
        let mut current_config = self.read_register(Self::CONF_REG)?;
        current_config &= !(0x0007);
        current_config |= self.measure_mode as u16;
        self.write_register(Self::CONF_REG, current_config)
    }

    /// not sure but i think this sets the correction value for measurements based on resistance and maximum current
    pub fn set_resistor_range(
        &mut self,
        resistance: f32,
        mut current_range: f32,
    ) -> Result<(), Error> {
        if current_range < 0. {
            current_range = 0.0819175 / resistance;
        }

        let current_lsb = current_range / 32768.0;
        self.cal_val = (0.00512 / (current_lsb * resistance)) as u16;
        self.current_divider_ma = 0.001 / current_lsb;
        self.pwr_multiplier_mw = 25000. * current_lsb;

        self.write_register(
            Self::CAL_REG,
            (self.cal_val as f32 * self.corr_factor) as u16,
        )
    }

    pub fn get_shunt_voltage(&mut self) -> Result<f32, Error> {
        let val = self.read_register(Self::SHUNT_REG)? as i16;
        Ok(val as f32 * 0.0000025 * self.corr_factor)
        // self.read_register(Self::SHUNT_REG)
        //     .map(|x| x as f32 * 0.0000025 * self.corr_factor)
    }
    pub fn get_shunt_voltage_millis(&mut self) -> Result<f32, Error> {
        let val = self.read_register(Self::SHUNT_REG)? as i16;
        Ok(val as f32 * 0.0025 * self.corr_factor)
        // self.read_register(Self::SHUNT_REG)
        //     .map(|x| x as f32 * 0.0025 * self.corr_factor)
    }
    pub fn get_bus_voltage(&mut self) -> Result<f32, Error> {
        self.read_register(Self::BUS_REG)
            .map(|x| x as f32 * 0.00125)
    }
    pub fn get_current_millis(&mut self) -> Result<f32, Error> {
        let val = self.read_register(Self::CURRENT_REG)? as i16;
        Ok(val as f32 / self.current_divider_ma)
        // self.read_register(Self::CURRENT_REG)
        //     .map(|x| x as f32 / self.current_divider_ma)
    }
    pub fn get_current(&mut self) -> Result<f32, Error> {
        Ok(self.get_current_millis()? / 1000.)
    }
    pub fn get_bus_power(&mut self) -> Result<f32, Error> {
        self.read_register(Self::PWR_REG)
            .map(|x| x as f32 * self.pwr_multiplier_mw)
    }

    /// dont know, see INA226_WE
    pub fn start_single_measurement(&mut self) -> Result<(), Error> {
        let _ = self.read_register(Self::MASK_EN_REG)?;
        let val = self.read_register(Self::CONF_REG)?;
        self.write_register(Self::CONF_REG, val)?;
        let mut conv_ready = 0u16;
        let conv_start = esp_hal::time::Instant::now();
        while conv_ready == 0 && (Instant::now() - conv_start).as_millis() < 2000 {
            conv_ready = self.read_register(Self::MASK_EN_REG)? & 0x0008;
        }
        Ok(())
    }
    /// dont know, see INA226_WE
    pub fn start_single_measurement_no_wait(&mut self) -> Result<(), Error> {
        let _ = self.read_register(Self::MASK_EN_REG)?;
        let val = self.read_register(Self::CONF_REG)?;
        self.write_register(Self::CONF_REG, val)
    }

    pub fn power_down(&mut self) -> Result<(), Error> {
        self.conf_reg_copy = self.read_register(Self::CONF_REG)?;
        self.set_measure_mode(MeasureMode::PowerDown)
    }
    pub fn power_up(&mut self) -> Result<(), Error> {
        self.write_register(Self::CONF_REG, self.conf_reg_copy)?;
        let delay = Delay::new();
        delay.delay_micros(40);
        Ok(())
    }

    pub fn is_busy(&mut self) -> Result<bool, Error> {
        Ok((self.read_register(Self::MASK_EN_REG)? & 0x0008) == 0)
    }

    /// do this before reading values
    pub fn wait_until_conversion_completed(&mut self) -> Result<(), Error> {
        let _ = self.read_register(Self::MASK_EN_REG)?;
        let mut conv_ready = 0u16;
        while conv_ready == 0 {
            {
                conv_ready = self.read_register(Self::MASK_EN_REG)? & 0x0008;
            }
        }
        Ok(())
    }

    pub fn set_alert_pin_active_high(&mut self) -> Result<(), Error> {
        let mut val = self.read_register(Self::MASK_EN_REG)?;
        val |= 0x0002;
        self.write_register(Self::MASK_EN_REG, val)
    }
    pub fn enable_alert_latch(&mut self) -> Result<(), Error> {
        let mut val = self.read_register(Self::MASK_EN_REG)?;
        val |= 0x0001;
        self.write_register(Self::MASK_EN_REG, val)
    }
    pub fn enable_conv_ready_alert(&mut self) -> Result<(), Error> {
        let mut val = self.read_register(Self::MASK_EN_REG)?;
        val |= 0x0400;
        self.write_register(Self::MASK_EN_REG, val)
    }
    /// dont know, see datasheet
    pub fn set_alert_type(&mut self, alert_type: AlertType, limit: f32) -> Result<(), Error> {
        self.alert_type = alert_type;
        let alert_limit = match alert_type {
            AlertType::ShuntOver | AlertType::ShuntUnder => limit * 400.,
            AlertType::CurrentUnder => {
                self.alert_type = AlertType::ShuntUnder;
                limit * 2048. * self.current_divider_ma / self.cal_val as f32
            }
            AlertType::CurrentOver => {
                self.alert_type = AlertType::ShuntOver;
                limit * 2048. * self.current_divider_ma / self.cal_val as f32
            }
            AlertType::BusOver | AlertType::BusUnder => limit * 800.,
            AlertType::PowerOver => limit / self.pwr_multiplier_mw,
            AlertType::None => 0.,
        };

        self.write_register(Self::ALERT_LIMIT_REG, alert_limit as u16)?;

        let mut val = self.read_register(Self::MASK_EN_REG)?;
        val &= !(0xF800);
        val |= self.alert_type as u16;
        self.write_register(Self::MASK_EN_REG, val)
    }

    /// always do this before reading measurements
    pub fn read_and_clear_flags(&mut self) -> Result<(), Error> {
        let val = self.read_register(Self::MASK_EN_REG)?;
        self.overflow = (val >> 2) & 1 == 1;
        self.conv_alert = (val >> 3) & 1 == 1;
        self.limit_alert = (val >> 4) & 1 == 1;
        Ok(())
    }
}
