#![no_std]
#![no_main]
#![deny(clippy::large_stack_frames)]
// #![deny(clippy::mem_forget)]

use core::cell::RefCell;
use core::fmt::{self, Write};

use bytemuck::{Pod, Zeroable};
use critical_section::Mutex;
use embedded_io::{Read, Write as IoWrite};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::efuse::Efuse;
use esp_hal::time::Instant;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{Blocking, handler, main};
use esp_radio::esp_now::{self, PeerInfo};
use esp_radio::{Controller, wifi};
use static_cell::StaticCell;

const MESSAGE: u8 = 0x01;
const PRINT: u8 = 0x00;

// test
// const PEER_ADDRS[mac_no]: [u8; 6] = [148, 169, 144, 109, 231, 180];
// real test old
include!(concat!(env!("OUT_DIR"), "/known_macs.rs"));

// KNOWN_MACS is now available as [[u8; 6]; N] with the exact count baked in
// const PEER_ADDRS: [[u8; 6]; 3] = [
//     [148, 169, 144, 122, 44, 104],
//     [148, 169, 144, 109, 231, 180],
//     [148, 169, 144, 106, 157, 104],
// ];
// old
// const PEER_ADDRS[mac_no]: [u8; 6] = [176, 129, 132, 5, 216, 100];

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CommandPacket {
    purge_time: u32,
    purge_interval: u32,
    fan_speed: u8,
    restart: u8,
    mac_no: u8,
    send_wait: u8,
    receivers: [u8; 2],
    _pad: [u8; 2],
}
#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct ControlPacket {
    forward: f32,
    steer: f32,
}
// union Packet {
//     controls: ControlPacket,
//     command: CommandPacket,
// }

#[derive(Default, Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
struct MessagePacket {
    acc: [f32; 3],
    gyr: [f32; 3],
    temp: f32,
    time_since_purge: u32,
    elapsed_time: u32,
    voltage: f32,
    current: f32,
    power: f32,
    fc_voltage: f32,
    fc_current: f32,
    batt_flags: u16,
    fc_flags: u16,
    energy_wh: f32,
    hydro_wh: f32,
    a1_voltage: f32,
    free_heap: u32,
    test_data: u32,
}

static USB_SERIAL: Mutex<RefCell<Option<UsbSerialJtag<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));
static RADIO: StaticCell<Controller> = StaticCell::new();

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

macro_rules! usb_println {
    ($($arg:tt)*) => {{
        critical_section::with(|cs| {
            if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs).as_mut() {
                let mut buf = [0u8; 512]; // big enough for MessagePacket debug
                let mut writer = BufWriter { buf: &mut buf, pos: 0 };
                let _ = writeln!(writer, $($arg)*);
                let len = writer.pos;

                usb.write(&[PRINT, PRINT]).ok();
                usb.write(&(len as u16).to_be_bytes()).ok();
                // usb.write(&[lenu8]).ok();
                usb.write(&buf[..len]).ok();
                usb.flush().ok();
            }
        });
    }};
}

#[handler]
fn on_receive() {
    usb_println!("hearing ya loud and clear cap'n!");
    critical_section::with(|cs| {
        usb_println!("startin the engines");
        if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs).as_mut() {
            let mut header = [0; 1];
            usb.read_exact(&mut header).ok();
            if header[0] == 0 {
                let mut buf = [0u8; core::mem::size_of::<CommandPacket>()];
                usb.read_exact(&mut buf).ok();
                let packet: &CommandPacket = bytemuck::from_bytes(&buf);

                *PENDING_CMD.borrow_ref_mut(cs) = Some(*packet);

                {
                    let mut buf = [0u8; 50];
                    let mut writer = BufWriter {
                        buf: &mut buf,
                        pos: 0,
                    };
                    let _ = writeln!(writer, "everythings happy :) got cmd");
                    let len = writer.pos;

                    usb.write(&[PRINT, PRINT]).ok();
                    usb.write(&(len as u16).to_be_bytes()).ok();
                    usb.write(&buf[..len]).ok();
                    usb.flush().ok();
                }
            } else if header[0] == 1 {
                let mut buf = [0u8; core::mem::size_of::<ControlPacket>()];
                usb.read_exact(&mut buf).ok();
                let packet: &ControlPacket = bytemuck::from_bytes(&buf);

                *PENDING_CTRL.borrow_ref_mut(cs) = Some(*packet);

                {
                    let mut buf = [0u8; 50];
                    let mut writer = BufWriter {
                        buf: &mut buf,
                        pos: 0,
                    };
                    let _ = writeln!(writer, "everythings happy :) got ctrl");
                    let len = writer.pos;

                    usb.write(&[PRINT, PRINT]).ok();
                    usb.write(&(len as u16).to_be_bytes()).ok();
                    usb.write(&buf[..len]).ok();
                    usb.flush().ok();
                }
            }
            usb.reset_rx_packet_recv_interrupt();
        }
    });
}
static PENDING_CMD: Mutex<RefCell<Option<CommandPacket>>> = Mutex::new(RefCell::new(None));
static PENDING_CTRL: Mutex<RefCell<Option<ControlPacket>>> = Mutex::new(RefCell::new(None));

esp_bootloader_esp_idf::esp_app_desc!();
#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let delay = Delay::new();
    delay.delay_millis(1000);

    let mut mac_no = 0;
    let mut usb = UsbSerialJtag::new(peripherals.USB_DEVICE);
    usb.write(&[0x0, 0x0, 0x0]).unwrap();
    usb.write(&7u16.to_be_bytes()).unwrap();
    usb.write(b"fuck uy").unwrap();
    usb.flush().unwrap();

    critical_section::with(|cs| USB_SERIAL.borrow_ref_mut(cs).replace(usb));

    critical_section::with(|cs| {
        if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs).as_mut() {
            usb.set_interrupt_handler(on_receive);
            usb.listen_rx_packet_recv_interrupt();
        }
    });
    let mac_address = Efuse::mac_address();
    usb_println!("{:?}", mac_address);
    usb_println!("binary stuff: {:?}", b"fuck uy");

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let radio_init = RADIO.init_with(|| esp_radio::init().expect("cannot init radio"));
    let (mut controller, interfaces) =
        esp_radio::wifi::new(&*radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    controller.set_mode(wifi::WifiMode::Sta).unwrap();
    controller.start().unwrap();

    let mut esp_now = interfaces.esp_now;
    for i in PEER_ADDRS.iter() {
        let peer_info = PeerInfo {
            interface: esp_now::EspNowWifiInterface::Sta,
            peer_address: *i,
            lmk: None,
            channel: Some(0),
            encrypt: false,
        };
        esp_now.add_peer(peer_info).unwrap();
    }

    let mut start = Instant::now();

    loop {
        while let Some(frame) = esp_now.receive()
            && frame.info.src_address == PEER_ADDRS[mac_no]
            && frame.data().len() == core::mem::size_of::<MessagePacket>() + 1
        {
            let frame = frame.data();
            // usb_println!("{:?}", frame);
            if frame[0] == 0x00 {
                let msg: MessagePacket = bytemuck::pod_read_unaligned(&frame[1..]);

                critical_section::with(|cs2| {
                    if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs2).as_mut() {
                        let len = core::mem::size_of::<MessagePacket>() as u16;
                        usb.write(&[MESSAGE, MESSAGE]).ok();
                        usb.write(&len.to_be_bytes()).ok();
                        usb.write(bytemuck::bytes_of(&msg)).ok();
                        usb.flush().ok();
                    }
                });
            } else if frame[0] == 0x01 {
                let module = TryInto::<Module>::try_into(frame[1]).unwrap();
                let line = u16::from_be_bytes([frame[2], frame[3]]);
                let str = str::from_utf8(&frame[4..]).unwrap();

                usb_println!(
                    "BRAIN FART!!!!!! MODULE: {module} |||||||| LINE: {line} |||||||| {str}"
                );
            }
        }

        if let Some(waiter) = critical_section::with(|cs| {
            if let Some(cmd) = PENDING_CMD.borrow_ref_mut(cs).take() {
                usb_println!(
                    "{} {:?}",
                    core::mem::size_of::<CommandPacket>(),
                    &[&[0], bytemuck::bytes_of(&cmd)].concat()
                );
                if cmd.mac_no != 0 {
                    mac_no = cmd.mac_no as usize - 1;
                    None
                } else if let Ok(waiter) = esp_now.send(
                    &PEER_ADDRS[mac_no],
                    &[&[0], bytemuck::bytes_of(&cmd)].concat(),
                ) {
                    usb_println!("get a 0 and sent cmd");
                    Some(waiter)
                } else {
                    None
                }
            } else if let Some(ctrl) = PENDING_CTRL.borrow_ref_mut(cs).take() {
                usb_println!("{:?}", &[&[1], bytemuck::bytes_of(&ctrl)].concat());
                if let Ok(waiter) = esp_now.send(
                    &PEER_ADDRS[mac_no],
                    &[&[1], bytemuck::bytes_of(&ctrl)].concat(),
                ) {
                    usb_println!("got a 1 and sent ctrl");
                    Some(waiter)
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            waiter.wait().unwrap();
            usb_println!("sent");
        }

        if start.elapsed().as_secs() >= 1 {
            start = Instant::now();
            usb_println!("Receiver alive");
        }

        delay.delay_millis(1);
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    usb_println!("PANIC: {}", info);

    loop {}
}
#[repr(u8)]
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
    Ledc = 0x09,
}
impl core::fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl TryFrom<u8> for Module {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x00 => Ok(Self::FcVcInit),
            0x01 => Ok(Self::BatVcInit),
            0x02 => Ok(Self::ImuInit),
            0x03 => Ok(Self::I2CInit),
            0x04 => Ok(Self::FcVc),
            0x05 => Ok(Self::BatVc),
            0x06 => Ok(Self::Imu),
            0x08 => Ok(Self::Adc),
            0x09 => Ok(Self::Ledc),
            _ => Err(()),
        }
    }
}
