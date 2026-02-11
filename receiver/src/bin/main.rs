#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;

use alloc::string::String;
use alloc::vec::Vec;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{Blocking, handler, main, usb_serial_jtag};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::esp_now::{self, EspNow, PeerInfo};
use esp_radio::{Controller, wifi};
use bytemuck::{Pod, Zeroable};
use embedded_io::Write;
use static_cell::StaticCell;

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CommandPacket {
    purge_time: u32,
    purge_interval: u32,
    fan_speed: u8,
    _pad: [u8; 7]
}


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

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const PEER_ADDR: [u8; 6] = [ 0xB0, 0x81, 0x84, 0x05, 0xD6, 0xF4 ];

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    
    let mut usb_serial = usb_serial_jtag::UsbSerialJtag::new(peripherals.USB_DEVICE);
    usb_serial.write(b"ACCX_Avg,ACCY_Avg,ACCZ_Avg,CurrentAvg,BattV,FC_V,FC_I,UpTime,EnergyWh,HydroWh,A1_V\n");
    usb_serial.set_interrupt_handler(on_receive);
    usb_serial.listen_rx_packet_recv_interrupt();
    critical_section::with(|cs| USB_SERIAL.borrow_ref_mut(cs).replace(usb_serial));

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    let radio_init = RADIO.init_with(|| esp_radio::init().expect("couldnt init radio"));
    let (mut controller, interfaces) =
        esp_radio::wifi::new(&*radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    controller.set_mode(wifi::WifiMode::Sta).unwrap();
    controller.start().unwrap();

    let esp_now = interfaces.esp_now;
    esp_now.set_channel(0).unwrap();

    let peer_info = PeerInfo {
        interface: esp_now::EspNowWifiInterface::Sta,
        peer_address: PEER_ADDR,
        lmk: None,
        channel: Some(0),
        encrypt: false
    };
    esp_now.add_peer(peer_info).unwrap();
    critical_section::with(|cs| ESP_NOW.borrow_ref_mut(cs).replace(esp_now));

    loop {
        critical_section::with(|cs| {
            let mut esp_now = ESP_NOW.borrow_ref_mut(cs);
            let mut usb_serial = USB_SERIAL.borrow_ref_mut(cs);
            if let Some(esp_now) = esp_now.as_mut() &&
                let Some(usb_serial) = usb_serial.as_mut() {
                    if let Some(data) = esp_now.receive() && data.info.src_address == PEER_ADDR {
                        let message: MessagePacket = *bytemuck::from_bytes(data.data());    
                        usb_serial.write_fmt(format_args!("{:?}", message)).expect(":(");
                    }
                }
        })
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}

fn read_until(usb_serial: &mut UsbSerialJtag<'_, Blocking>, ch: char) -> String {
    let mut cch: char = usb_serial.read_byte().expect("couldnt read") as char;
    let mut string: Vec<char> = Vec::new();
    string.push(cch);
    while cch != ch {
        let byte = usb_serial.read_byte().expect("couldnt read");
        cch = byte as char;
        string.push(cch);
    }
    string.iter().collect()
}

static USB_SERIAL: Mutex<RefCell<Option<UsbSerialJtag<'static, Blocking>>>> = Mutex::new(RefCell::new(None));
static RADIO: StaticCell<Controller> = StaticCell::new();// static WIFI_CTRL: Mutex<RefCell<Option<(EspNow<'static>, Controller)>>> = Mutex::new(RefCell::new(None));
static ESP_NOW: Mutex<RefCell<Option<EspNow<'static>>>> = Mutex::new(RefCell::new(None));
static COMMAND: Mutex<RefCell<CommandPacket>> = Mutex::new(RefCell::new(CommandPacket { purge_time: 200, purge_interval: 50000, fan_speed: 128, _pad: [0; 7] }));

#[handler]
fn on_receive() {
    critical_section::with(|cs| {
        let mut usb_serial = USB_SERIAL.borrow_ref_mut(cs);
        let mut esp_now = ESP_NOW.borrow_ref_mut(cs);
        let mut command = COMMAND.borrow_ref_mut(cs);
        if let Some(usb_serial) = usb_serial.as_mut()
            && let Some(esp_now) = esp_now.as_mut() {
            let read = read_until(usb_serial, '\n');
            let line = read.trim();
            if line.starts_with("SET ") {
                let sp1 = line.find(" ").expect("waht");
                let sp2 = line[(sp1 + 1)..].find(" ").expect("what");
                let name = &line[(sp1 + 1)..sp2];
                let val: u32 = line[(sp2 + 1)..].parse().expect(":(");

                if name == "PURGE_TIME" {
                    command.purge_time = val
                } else if name == "PURGE_INTERVAL" {
                    command.purge_interval = val
                } else if name == "FAN_SPEED" {
                    command.fan_speed = val as u8
                }

                let waiter = esp_now.send(&PEER_ADDR, bytemuck::bytes_of(&*command)).expect("noooo");
                waiter.wait();
        
                usb_serial.write(b"sent command").expect("noooo");
            }
        }
    })
}