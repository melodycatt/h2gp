#![no_std]
#![no_main]
// #![allow(clippy::empty_loop, unused)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![allow(clippy::empty_loop)]
// Offsets from your partition table's "nvs" partition
const NAME_PART_OFF: usize = 0x9000;
const NAME_PART_SIZE: usize = 0x6000;

extern crate alloc;
use esp_backtrace as _;
use esp_hal::delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::main;
use esp_hal::time::{Instant, Rate};
use esp_hal::timer::timg::TimerGroup;
<<<<<<< HEAD
use esp_println::println;
use icm42688::Icm42688;
use ina226::Ina226;

extern crate alloc;

=======
// use esp_println::println;
>>>>>>> 738a6ad (fuck dude please)
use esp_hal::i2c;
use esp_radio::esp_now::{self, PeerInfo};
use esp_radio::wifi;
use esp_radio::wifi::sta::StationConfig;
use icm42688::Icm42688;
use ina226::Ina226;
use util_types::*;

// todo me ina226 and icm42688 libraries
// VC = voltage and current monitor
// MG = motion + gyro monitor

const BATTERY_I2C_ADDR: u8 = 0x44;
const FC_I2C_ADDR: u8 = 0x41;

include!(concat!(env!("OUT_DIR"), "/known_macs.rs"));
const SRC_ADDR: [u8; 6] = PEER_ADDRS[0];
// const SRC_ADDR: [u8; 6] = [88, 140, 129, 158, 39, 188];

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

    start_time: Option<Instant>,
    receivers: [[u8; 6]; 4],
    free_slots: u8,
    send_wait: u8,
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();
macro_rules! unwrap_or_report {
    ($expr:expr, $esp_now:expr, $module:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                send_error_msg($esp_now, $module, &e, line!());
                continue;
            }
        }
    };
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let mut state = State {
        purge_time: 150,
        purge_interval: 20000,
        send_wait: 1,
        free_slots: 4,
        ..Default::default()
    };
    let mut peripherals =
        esp_hal::init(esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::_80MHz));

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let mut batt_led = Output::new(
        peripherals.GPIO10.reborrow(),
        Level::Low,
        OutputConfig::default(),
    );
    let mut ready_led = Output::new(
        peripherals.GPIO5.reborrow(),
        Level::Low,
        OutputConfig::default(),
    );

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let wifi = peripherals.WIFI;
    let station_config =
        esp_radio::wifi::Config::Station(esp_radio::wifi::sta::StationConfig::default());
    let (mut controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default()).unwrap();
    controller.set_config(&station_config);

    let mut esp_now = interfaces.esp_now;

    for i in &PEER_ADDRS {
        let peer_info = PeerInfo {
            interface: esp_now::EspNowWifiInterface::Station,
            peer_address: *i,
            lmk: None,
            channel: Some(0),
            encrypt: false,
        };
        esp_now.add_peer(peer_info).unwrap();
    }

    /////
    ///// i2c
    /////
    let i2c = I2c::new(
        peripherals.I2C0.reborrow(),
        i2c::master::Config::default().with_frequency(Rate::from_hz(400_000)),
    )
    .unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::I2CInit, &e, line!());
        loop {}
    })
    .with_scl(peripherals.GPIO7)
    .with_sda(peripherals.GPIO6);

    let mut ina226 = Ina226::new(&i2c, FC_I2C_ADDR);
    ina226.init().unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::FcVcInit, &e, line!());
        loop {}
    });
    ina226.set_resistor_range(0.003, 8.).unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::FcVcInit, &e, line!());
        loop {}
    });
    ina226
        .wait_until_conversion_completed()
        .unwrap_or_else(|e| {
            send_error_msg(&mut esp_now, Module::FcVcInit, &e, line!());
            loop {}
        });

    let mut ina226_fc = Ina226::new(&i2c, BATTERY_I2C_ADDR);
    ina226_fc.init().unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::BatVcInit, &e, line!());
        loop {}
    });
    ina226_fc.set_resistor_range(0.003, 8.).unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::BatVcInit, &e, line!());
        loop {}
    });
    ina226_fc
        .wait_until_conversion_completed()
        .unwrap_or_else(|e| {
            send_error_msg(&mut esp_now, Module::BatVcInit, &e, line!());
            loop {}
        });

    //
    // imu
    //
    let mut imu = Icm42688::new(&i2c, 0x68);
    imu.begin().unwrap_or_else(|e| {
        send_error_msg(&mut esp_now, Module::ImuInit, &e, line!());
        loop {}
    });

    let _fan = Output::new(
        peripherals.GPIO8.reborrow(),
        Level::High,
        OutputConfig::default(),
    );
    let mut valve1 = Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());
    let mut valve2: Output<'_> =
        Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let delay = delay::Delay::new();
    delay.delay_millis(50);
    valve1.set_low();
    valve2.set_low();

    // let mut cfg = AdcConfig::new();
    // let mut therm = cfg.enable_pin(peripherals.GPIO3, analog::adc::Attenuation::_11dB);
    // let mut adc = analog::adc::Adc::new(peripherals.ADC1, cfg);

    state.start_time = Some(Instant::now());
    state.last_sample_ms = state.start_time;
    state.previous_time = state.start_time;
    println!("{PEER_ADDRS:?}");
    state.short_previous_time = state.start_time;
    let storage = esp_storage::FlashStorage::new(peripherals.FLASH);
    let mut nvs =
        esp_nvs::Nvs::new(NAME_PART_OFF, NAME_PART_SIZE, storage).expect("failed to create nvs");

    ready_led.set_high();
    let mut connected = false;

    loop {
        // unwrap_or_report!(imu.detect_config(), &mut esp_now, Module::Imu);
        unwrap_or_report!(imu.get_agt(), &mut esp_now, Module::Imu);
        let acc = imu.acc();
        let gyr = imu.gyr();
        state.message.gyr = gyr;
        state.message.acc = acc;
        state.message.temp = imu.temp();
        let current_time = Instant::now();
        let dt = (current_time - state.last_sample_ms.unwrap()).as_micros() as f32 / 1000.;
        state.last_sample_ms = Some(current_time);

        state.message.batt_flags =
            unwrap_or_report!(ina226.read_and_clear_flags(), &mut esp_now, Module::BatVc);
        state.message.fc_flags = unwrap_or_report!(
            ina226_fc.read_and_clear_flags(),
            &mut esp_now,
            Module::BatVc
        );
        let voltage = unwrap_or_report!(ina226.get_bus_voltage(), &mut esp_now, Module::BatVc);
        let current = unwrap_or_report!(ina226.get_current(), &mut esp_now, Module::BatVc);
        let power = unwrap_or_report!(ina226.get_bus_power(), &mut esp_now, Module::BatVc);
        state.message.power = power;
        state.message.voltage = voltage;
        state.message.current = current;
        /*if ina226.overflow {
            println!("current overflow");
        }*/
        state.energy_wh += (voltage * current) * (dt / 3600000.);

        let fc_voltage = unwrap_or_report!(ina226_fc.get_bus_voltage(), &mut esp_now, Module::FcVc);
        let fc_current = unwrap_or_report!(ina226_fc.get_current(), &mut esp_now, Module::FcVc);
        /*if ina226_fc.overflow {
            println!("current overflow on fuel cell");
        }*/
        state.message.fc_current = fc_current;
        state.message.fc_voltage = fc_voltage;
        state.hydro_wh += (fc_voltage * fc_current) * (dt / 3600000.);

        if state.message.voltage > 1. {
            batt_led.set_high();
        } else {
            batt_led.set_low();
        }

        state.message.energy_wh = state.energy_wh;
        state.message.hydro_wh = state.hydro_wh;
        let purge_delta = (current_time - state.previous_time.unwrap()).as_millis() as u32;
        state.message.time_since_purge = purge_delta;
        if purge_delta >= state.purge_interval {
            state.message.test_data = 1;
            state.purge_complete = false;
            valve1.set_high();
            valve2.set_high();
            state.previous_time = Some(current_time)
        } else if purge_delta >= state.purge_time && !state.purge_complete {
            state.message.test_data = 0;
            valve1.set_low();
            valve2.set_low();
            state.purge_complete = true;
        }
        // let raw_therm = unwrap_or_report!(
        //     nb::block!(adc.read_oneshot(&mut therm)),
        //     &mut esp_now,
        //     Module::Adc
        // );
        // let v_out = raw_therm as f32;
        // state.message.a1_voltage = v_out;

        state.message.elapsed_time = (current_time - state.start_time.unwrap()).as_millis() as u32;
        if let Some(data) = esp_now.receive() {
            let data = data.data();
            if data[0] == 0 {
                let command = bytemuck::pod_read_unaligned::<CommandPacket>(&data[1..]);
                if command.restart == 1 {
                    esp_hal::system::software_reset()
                }

                state.send_wait = command.send_wait;
                // state.receivers = command.receivers;
                state.fan_speed = command.fan_speed;
                state.purge_interval = command.purge_interval;
                state.purge_time = command.purge_time;
            }
            if data[1] == 1 {
                let scan = bytemuck::pod_read_unaligned::<ScanPacket>(&data[1..]);

                nvs.set("my_key", &b"meow")?;
                let data: [u8; 32] = nvs.get("my_key")?;
                let shake = HandshakeInit {
                    name: todo!(),
                    available_slots: todo!(),
                };
                // esp_now.send(dst_addr, data);
            }
        }
        state.message.free_heap = esp_alloc::HEAP.free() as u32;
        const MSG_LEN: usize = 1 + core::mem::size_of::<MessagePacket>();
        let mut send_buf = [0u8; MSG_LEN];
        send_buf[0] = 0x00;
        send_buf[1..].copy_from_slice(bytemuck::bytes_of(&state.message));
        if state.receivers[0] != 0 {
            if let Ok(waiter) = esp_now.send(&PEER_ADDRS[0], &send_buf)
            // esp_now.send(&PEER_ADDRS[state.receivers[0] as usize - 1], &send_buf)
            {
                connect(&mut connected, true, &mut ready_led);
                if state.send_wait != 0 {
                    let _ = waiter.wait();
                }
            } else {
                connect(&mut connected, false, &mut ready_led);
            }
        }
        if state.receivers[1] != 0 {
            if let Ok(waiter) = esp_now.send(&PEER_ADDRS[0], &send_buf)
            // esp_now.send(&PEER_ADDRS[state.receivers[1] as usize - 1], &send_buf)
            {
                connect(&mut connected, true, &mut ready_led);
                if state.send_wait != 0 {
                    let _ = waiter.wait();
                }
            } else {
                connect(&mut connected, false, &mut ready_led);
            }
        }

        // println!("sending!!!! {send_buf:?}");
        delay.delay_millis(1);
    }
}

fn connect(current: &mut bool, new: bool, ready_led: &mut Output) {
    if *current != new {
        *current = new;
        ready_led.set_level(match new {
            false => Level::Low,
            true => Level::High,
        });
    }
}

/// Send a formatted error message over ESP-NOW
fn send_error_msg<E: core::fmt::Debug>(
    esp_now: &mut esp_radio::esp_now::EspNow<'_>,
    module: Module,
    err: &E,
    line: u32,
) {
    use core::fmt::Write;
    // println!("{:?} {:?} {:?}", module, line, err);
    // [ module | line_lo | line_hi | len | utf8 bytes... ]
    let mut buf = [0u8; 221];

    buf[0] = 0x01;
    buf[1] = module as u8;
    [buf[2], buf[3]] = (line as u16).to_be_bytes();

    let mut w = BufWriter {
        buf: &mut buf[4..],
        pos: 0,
    };
    let _ = write!(&mut w, "{:?}", err);
    let pos = w.pos;

    buf[3] = pos as u8;

    let _ = esp_now.send(&SRC_ADDR, &buf[..4 + pos]);
}

// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     esp_hal::system::software_reset()
// }
