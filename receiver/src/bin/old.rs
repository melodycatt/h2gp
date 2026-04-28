#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use core::fmt::Write;

use bytemuck::{Pod, Zeroable};
use critical_section::Mutex;
use embedded_io::Read;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::efuse::Efuse;
use esp_hal::time::Instant;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{Blocking, handler, main, usb_serial_jtag};
use esp_println::println;
use esp_radio::esp_now::{self, EspNow, PeerInfo};
use esp_radio::{Controller, wifi};
use static_cell::StaticCell;

const MESSAGE: u8 = 0x01;
const PRINT: u8 = 0x02;
macro_rules! send {
    ($usb:expr, $value:expr) => {
        $usb.write(&[MESSAGE]);
        $usb.write(&core::mem::size_of::<MessagePacket>().to_be_bytes());
        $usb.write(bytemuck::bytes_of(&$value));
        $usb.flush_tx();
    };
}

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CommandPacket {
    purge_time: u32,
    purge_interval: u32,
    fan_speed: u8,
    _pad: [u8; 7],
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

    fc_voltage: f32,
    fc_current: f32,
    energy_wh: f32,
    hydro_wh: f32,
    a1_voltage: f32,
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();
const PEER_ADDR: [u8; 6] = [148, 169, 144, 122, 44, 104];
// const PEER_ADDR: [u8; 6] = [148, 169, 144, 109, 231, 180];
// // [148, 169, 144, 109, 231, 180] FOR BILLY BRAINz
// // [88, 140, 129, 158, 39, 188] FOR RECIEVER
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

    let mac_adress = Efuse::read_base_mac_address();
    let mut usb_serial = usb_serial_jtag::UsbSerialJtag::new(peripherals.USB_DEVICE);

    usb_serial
        .write_fmt(format_args!("hello! my mac address is {mac_adress:?}"))
        .expect("AAAA");

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
    //esp_now.set_channel(0).expect("its this right");

    let peer_info = PeerInfo {
        interface: esp_now::EspNowWifiInterface::Sta,
        peer_address: PEER_ADDR,
        lmk: None,
        channel: Some(0),
        encrypt: false,
    };
    esp_now.add_peer(peer_info).unwrap();
    critical_section::with(|cs| ESP_NOW.borrow_ref_mut(cs).replace(esp_now));

    let mut start = Instant::now();
    loop {
        critical_section::with(|cs| {
            let mut esp_now = ESP_NOW.borrow_ref_mut(cs);
            let mut usb_serial = USB_SERIAL.borrow_ref_mut(cs);
            if let Some(esp_now) = esp_now.as_mut()
                && let Some(usb_serial) = usb_serial.as_mut()
            {
                if let Some(data) = esp_now.receive()
                    && data.info.src_address == PEER_ADDR
                {
                    println!("hi");
                    let message: MessagePacket = *bytemuck::from_bytes(data.data());
                    usb_serial
                        .write_fmt(format_args!("{message:?}"))
                        .expect(":(");
                }
            }
        })
    }
}

static USB_SERIAL: Mutex<RefCell<Option<UsbSerialJtag<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));
static RADIO: StaticCell<Controller> = StaticCell::new();
static ESP_NOW: Mutex<RefCell<Option<EspNow<'static>>>> = Mutex::new(RefCell::new(None));

#[handler]
fn on_receive() {
    critical_section::with(|cs| {
        let mut usb_serial_ref = USB_SERIAL.borrow_ref_mut(cs);
        let mut esp_now_ref = ESP_NOW.borrow_ref_mut(cs);

        if let Some(usb_serial) = usb_serial_ref.as_mut() {
            let mut data = [0u8; core::mem::size_of::<CommandPacket>()];

            if usb_serial.read_exact(&mut data).is_err() {
                return;
            }

            let packet: &CommandPacket = bytemuck::from_bytes(&data);

            if let Some(esp_now) = esp_now_ref.as_mut()
                && let Ok(waiter) = esp_now.send(&PEER_ADDR, bytemuck::bytes_of(packet))
            {
                let _ = waiter.wait();
            }
        }
    });
}

struct UsbWriter<'a> {
    usb: &'a mut UsbSerialJtag<'a, Blocking>,
}

impl<'a> Write for UsbWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.usb.write(&[PRINT]);
        self.usb.write(&s.len().to_be_bytes());
        self.usb.write(s.as_bytes());
        self.usb.flush_tx();
        Ok(())
    }
}
