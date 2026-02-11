#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]


use esp_backtrace as _;
use esp_hal::analog::adc::AdcConfig;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::ledc::channel::{self, ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{Blocking, analog, gpio, ledc, main, usb_serial_jtag};
use esp_hal::time::{Instant, Rate};
use esp_println::println;
use icm42688::Icm42688;
use ina226::Ina226;

extern crate alloc;

use esp_radio::esp_now::{self, PeerInfo};
use esp_radio::wifi;
use esp_hal::i2c;

// todo make ina226 and icm42688 libraries
// VC = voltage and current monitor
// MG = motion + gyro monitor

const BATTERY_I2C_ADDR: u8 = 0x41;
const FC_I2C_ADDR: u8 = 0x44;

const SRC_ADDR: [u8; 6] = [0x58, 0x8C, 0x81, 0xA1, 0x0F, 0xC4];

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CommandPacket {
    purge_time: u32,
    purge_interval: u32,
    fan_speed: u8,
    _pad: [u8; 7]
}

use bytemuck::{Pod, Zeroable};

#[derive(Default, Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
struct MessagePacket {
    // todo status message (my idea is a status code and associated byte(s) of info or someth)
    acc: [f32; 3],
    gyr: [f32; 3],
    temp: f32,
    voltage: f32,
    current: f32,
    power: f32,
    elapsed_time: u32,

    battery_voltage: f32,
    fc_voltage: f32,
    fc_current: f32,
    energy_wh: f32,
    hydro_wh: f32,
    a1_voltage: f32,
}

// maybe static here?

// maybe persistent parameters
//esp now peer info


// todo change types
#[derive(Default)]
struct State {
    purge_time: u32,
    purge_interval: u32,
    purge_complete: bool,
    fan_speed: u8,

    previous_time: Option<Instant>,
    short_previous_time: Option<Instant>, //rename this
    // VCs
    energy_wh: f32,
    hydro_wh: f32, // fc energy
    last_sample_ms: Option<Instant>,

    // mg monitor

    message: MessagePacket,

    start_time: Option<Instant>
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();
macro_rules! unwrap_or_report {
    ($expr:expr, $usb_serial:expr, $module:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                $usb_serial.write_fmt(format_args!("module: {:?} line: {:?} error: {:?}", $module, line!(), &e)).unwrap();
                //send_error_msg($esp_now, $module, &e, line!());
                continue;
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
enum Module {
    FcVcInit = 0x00,
    BatVcInit = 0x01,
    ImuInit = 0x02,
    I2CInit = 0x03,
    FcVc = 0x04,
    BatVc = 0x05,
    Imu = 0x06,
    Adc = 0x08,
    Ledc = 0x09
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let mut state = State::default();
    let mut peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let mut usb_serial = usb_serial_jtag::UsbSerialJtag::new(peripherals.USB_DEVICE);
    usb_serial.write(b"hello\n");
    
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
    esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    
    let esp_radio_ctrl = esp_radio::init().unwrap();
    let wifi = peripherals.WIFI;
    let (mut controller, interfaces) = esp_radio::wifi::new(&esp_radio_ctrl, wifi, Default::default()).unwrap();
    controller.set_mode(wifi::WifiMode::Sta).unwrap();
    controller.start().unwrap();

    usb_serial.write(b"got past wifi\n");
    /*let mut esp_now = interfaces.esp_now;
    esp_now.set_channel(0).unwrap();

    let peer_info = PeerInfo {
        interface: esp_now::EspNowWifiInterface::Sta,
        peer_address: SRC_ADDR,
        lmk: None,
        channel: Some(0),
        encrypt: false
    };
    esp_now.add_peer(peer_info).unwrap();*/

    
    let mut i2c = I2c::new(peripherals.I2C0.reborrow(), i2c::master::Config::default()).unwrap_or_else(|e| { usb_serial.write(b"sending\n"); send_error_msg(&mut usb_serial, Module::I2CInit, &e, line!()); loop {} }).with_scl(peripherals.GPIO7).with_sda(peripherals.GPIO6);
    usb_serial.write(b"hello\n");
    {
        let mut ina226 = Ina226::new(&mut i2c, FC_I2C_ADDR);
        esp_println::println!("hello?");
        // the write is probably erroring
        ina226.init().unwrap_or_else(|e| { 
            usb_serial.write(b"hello\n");
            send_error_msg(&mut usb_serial, Module::FcVcInit, &e, line!()); 
        });
        usb_serial.write(b"hello\n");
        ina226.set_resistor_range(0.003, 20.).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::FcVcInit, &e, line!()); loop {} 
        });
        ina226.wait_until_conversion_completed().unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::FcVcInit, &e, line!()); loop {} 
        });
    }
    {
        let mut ina226 = Ina226::new(&mut i2c, BATTERY_I2C_ADDR);
        ina226.init().unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::BatVcInit, &e, line!()); loop {} 
        });
        ina226.set_resistor_range(0.003, 20.).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::BatVcInit, &e, line!()); loop {} 
        });
        ina226.wait_until_conversion_completed().unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::BatVcInit, &e, line!()); loop {} 
        });
    }
    usb_serial.write(b"got past ina\n");
    {
        let mut icm42688 = Icm42688::new(&mut i2c, 0x68);
        icm42688.begin().unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::ImuInit, &e, line!()); loop {} 
        });
    }
    usb_serial.write(b"got pas icm\n");

    let mut valve1 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let mut valve2: Output<'_> = Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default());

    let mut cfg = AdcConfig::new();
    let mut therm = cfg.enable_pin(peripherals.GPIO4, analog::adc::Attenuation::_11dB);
    let mut adc = analog::adc::Adc::new(peripherals.ADC1, cfg);

    let ledc = ledc::Ledc::new(peripherals.LEDC.reborrow());
    let mut lstimer = ledc.timer(timer::Number::Timer0);
    lstimer.configure(timer::config::Config { duty: timer::config::Duty::Duty8Bit, clock_source: timer::LSClockSource::APBClk, frequency: Rate::from_hz(5000) }).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::Ledc, &e, line!()); loop {} 
        });
    let mut fan_channel = ledc.channel(channel::Number::Channel0, peripherals.GPIO20.reborrow());
    fan_channel.configure(channel::config::Config {
        timer: &lstimer,
        duty_pct: 0,
        drive_mode: gpio::DriveMode::PushPull,
    }).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::Ledc, &e, line!()); loop {} 
        });
    let mut batt_led_channel = ledc.channel(channel::Number::Channel0, peripherals.GPIO10.reborrow());
    batt_led_channel.configure(channel::config::Config {
        timer: &lstimer,
        duty_pct: 0,
        drive_mode: gpio::DriveMode::PushPull,
    }).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::Ledc, &e, line!()); loop {} 
        });
    let mut fc_led_channel = ledc.channel(channel::Number::Channel0, peripherals.GPIO9.reborrow());
    fc_led_channel.configure(channel::config::Config {
        timer: &lstimer,
        duty_pct: 0,
        drive_mode: gpio::DriveMode::PushPull,
    }).unwrap_or_else(|e| { 
            send_error_msg(&mut usb_serial, Module::Ledc, &e, line!()); loop {} 
        });

    usb_serial.write(b"got past init\n");


    state.start_time = Some(Instant::now());
    state.last_sample_ms = state.start_time;
    state.previous_time = state.start_time;
    state.short_previous_time = state.start_time;
    loop {
        usb_serial.write(b"loop start\n");
        fan_channel.set_duty_hw(state.fan_speed as u32);
        {
            let mut imu = Icm42688::new(&mut i2c, 0x68);
            unwrap_or_report!(imu.detect_config(), &mut usb_serial, Module::Imu);
            unwrap_or_report!(imu.get_agt(), &mut usb_serial, Module::Imu);
            let acc = imu.acc();
            let gyr = imu.gyr();
            state.message.gyr = gyr;
            state.message.acc = acc;
            state.message.temp = imu.temp();
        }
        let current_time = Instant::now();
        let dt = (current_time - state.last_sample_ms.unwrap()).as_micros() as f32 / 1000.;
        state.last_sample_ms = Some(current_time);
        {
            let mut ina226 = unwrap_or_report!(Ina226::new_initialized(&mut i2c, FC_I2C_ADDR), &mut usb_serial, Module::BatVc);
            let voltage = unwrap_or_report!(ina226.get_bus_voltage(), &mut usb_serial, Module::BatVc);
            let current = unwrap_or_report!(ina226.get_current(), &mut usb_serial, Module::BatVc);
            let power = unwrap_or_report!(ina226.get_bus_power(), &mut usb_serial, Module::BatVc);
            state.message.power = power;
            state.message.voltage = voltage;
            state.message.current = current;
            if ina226.overflow {
                println!("current overflow");
            }
            state.energy_wh += (voltage * current) * (dt / 3600000.);
        }
        {
            let mut ina226 = unwrap_or_report!(Ina226::new_initialized(&mut i2c, FC_I2C_ADDR), &mut usb_serial, Module::FcVc);
            let fc_voltage = unwrap_or_report!(ina226.get_bus_voltage(), &mut usb_serial, Module::FcVc);
            let fc_current = unwrap_or_report!(ina226.get_current(), &mut usb_serial, Module::FcVc);
            if ina226.overflow {
                println!("current overflow on fuel cell");
            }
            state.message.fc_current = fc_current;
            state.message.fc_voltage = fc_voltage;
            state.hydro_wh += (fc_voltage * fc_current)  * (dt / 3600000.);
        }
        if state.message.battery_voltage > 1. {
            batt_led_channel.set_duty_hw(50);
        } else {
            batt_led_channel.set_duty_hw(1);
        }

        if state.message.fc_voltage > state.message.battery_voltage + 0.1 {
            fc_led_channel.set_duty_hw(50);
        } else {
            fc_led_channel.set_duty_hw(1);
        }
        state.message.energy_wh = state.energy_wh;
        state.message.hydro_wh = state.hydro_wh;
        let purge_delta = (current_time - state.previous_time.unwrap()).as_millis() as u32;
        if purge_delta >= state.purge_interval {
            state.purge_complete = false;
            valve1.set_high();
            valve2.set_high();
            state.previous_time = Some(current_time)
        } else if purge_delta >= state.purge_time && !state.purge_complete {
            valve1.set_low();
            valve2.set_low();
            state.purge_complete = true;
        }

        let raw_therm = unwrap_or_report!(adc.read_oneshot(&mut therm), &mut usb_serial, Module::Adc);
        let v_out = raw_therm as f32;
        state.message.a1_voltage = v_out;
        
        state.message.elapsed_time = (current_time - state.start_time.unwrap()).as_millis() as u32;
        
        unwrap_or_report!(usb_serial.write_fmt(format_args!("{:?}", state.message)), &mut usb_serial, Module::Adc);
        /*let a = esp_now.send(&SRC_ADDR, bytemuck::bytes_of(&state.message)).expect("oh no");
        a.wait().expect("aaa");

        if let Some(data) = esp_now.receive() {
            if data.info.src_address != SRC_ADDR {
                continue;
            }
            let command: CommandPacket = *bytemuck::from_bytes(data.data());
            state.fan_speed = command.fan_speed;
            state.purge_interval = command.purge_interval;
            state.purge_time = command.purge_time;
        }*/
    }
}


use core::fmt::{self, Write};

/// Small stack-only writer for no_std
struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len().saturating_sub(self.pos);
        let n = remaining.min(bytes.len());
        self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
        self.pos += n;
        Ok(())
    }
}

/// Send a formatted error message over ESP-NOW
fn send_error_msg<E: fmt::Debug>(
    usb_serial: &mut UsbSerialJtag<'_, Blocking>,
    // esp_now: &mut esp_radio::esp_now::EspNow<'_>,
    module: Module,
    err: &E,
    line: u32,
) {
    usb_serial.write(b"sending\n");
    //usb_serial.write_fmt(format_args!("module: {:?} line: {:?} error: {:?}", module, line, err)).unwrap();
    usb_serial.write(b"sent\n");
    // [ module | line_lo | line_hi | len | utf8 bytes... ]
    /*let mut buf = [0u8; 220];

    buf[0] = module as u8;
    buf[1] = (line & 0xFF) as u8;
    buf[2] = ((line >> 8) & 0xFF) as u8;
    
    let mut w = BufWriter {
        buf: &mut buf[4..],
        pos: 0,
    };
    let _ = write!(&mut w, "{:?}", err);
    let pos = w.pos;
    drop(w);
    
    buf[3] = pos as u8;

    let _ = esp_now.send(
        &SRC_ADDR,
        &buf[..4 + pos],
    );*/
}