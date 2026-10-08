#![no_std]
#![no_main]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::Mutex;
use embedded_io::{Read, Write as IoWrite};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::efuse::Efuse;
use esp_hal::time::Instant;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{Blocking, handler, main};
#[allow(unused)]
use esp_println::println;
use esp_radio::esp_now::{self, PeerInfo};
use esp_radio::{Controller, wifi};
use static_cell::StaticCell;
use util_types::*;

include!(concat!(env!("OUT_DIR"), "/known_macs.rs"));
static USB_SERIAL: Mutex<RefCell<Option<UsbSerialJtag<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));
static RADIO: StaticCell<Controller> = StaticCell::new();
static PENDING_CMD: Mutex<RefCell<Option<CommandPacket>>> = Mutex::new(RefCell::new(None));
static PENDING_RENAME: Mutex<RefCell<Option<RenamePacket>>> = Mutex::new(RefCell::new(None));
static DO_SCAN: AtomicBool = AtomicBool::new(false);
macro_rules! usb_println {
    ($($arg:tt)*) => {{
        critical_section::with(|cs| {
            if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs).as_mut() {
                let mut buf = [0u8; 516]; // big enough for MessagePacket debug
                let mut writer = BufWriter { buf: &mut buf, pos: 4 };
                let _ = writeln!(writer, $($arg)*);
                let len = writer.pos;
                buf[0] = PRINT;
                buf[1] = PRINT;
                buf[2..4].copy_from_slice(&(len as u16 - 4).to_be_bytes());
                usb.write(&buf[..len]).ok();
                usb.flush().ok();
            }
        });
    }};
}

esp_bootloader_esp_idf::esp_app_desc!();
#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let delay = Delay::new();
    delay.delay_millis(1000);

    let mut connected_brain_addr = [0; 6];
    let mut usb = UsbSerialJtag::new(peripherals.USB_DEVICE);
    {
        let mut buf = [0u8; 516]; // big enough for MessagePacket debug
        let mut writer = BufWriter { buf: &mut buf, pos: 4 };
        let _ = writeln!(&mut writer, "fuck uy");
        let len = writer.pos;
        buf[0] = PRINT;
        buf[1] = PRINT;
        buf[2..4].copy_from_slice(&(len as u16 - 4).to_be_bytes());
        usb.write(&buf[..len]).ok();
        usb.flush().ok();
    }

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
    let mut start2 = Instant::now();

    loop {
        while let Some(frame) = esp_now.receive() {
            let sender = frame.info;
            let frame = frame.data();
            // println!("at least i heard something and its header was {}", frame[0]);
            // usb_println!("{:?}", frame);
            if frame[0] == 0x00 && sender.src_address == connected_brain_addr {
                let msg: MessagePacket = bytemuck::pod_read_unaligned(&frame[1..]);

                critical_section::with(|cs2| {
                    if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs2).as_mut() {
                        const LEN: usize = core::mem::size_of::<MessagePacket>();
                        let mut buf = [0u8; LEN + 4]; // big enough for MessagePacket debug
                        buf[0] = MESSAGE;
                        buf[1] = MESSAGE;
                        buf[2..4].copy_from_slice(&(LEN as u16).to_be_bytes());
                        buf[4..].copy_from_slice(bytemuck::bytes_of(&msg));
                        usb.write(&buf[..]).ok();
                        usb.flush().ok();
                    }
                });
            } else if frame[0] == 0x01 && sender.src_address == connected_brain_addr {
                let module = TryInto::<Module>::try_into(frame[1]).unwrap();
                let line = u16::from_be_bytes([frame[2], frame[3]]);
                let str = str::from_utf8(&frame[4..]).unwrap();

                usb_println!(
                    "BRAIN FART!!!!!! MODULE: {module} |||||||| LINE: {line} |||||||| {str}"
                );
            } else if frame[0] == 0x02 {
                usb_println!("booyah");
                let shake: HandshakeInit = bytemuck::pod_read_unaligned(&frame[1..]);
                critical_section::with(|cs2| {
                    if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs2).as_mut() {
                        const LEN1: usize = core::mem::size_of::<HandshakeInit>();
                        const LEN2: usize = core::mem::size_of::<[u8; 6]>();
                        let mut buf = [0u8; LEN1 + LEN2 + 4]; // big enough for MessagePacket debug
                        buf[0] = SCAN;
                        buf[1] = SCAN;
                        buf[2..4].copy_from_slice(&((LEN1 + LEN2) as u16).to_be_bytes());
                        buf[4..(4 + LEN1)].copy_from_slice(bytemuck::bytes_of(&shake));
                        buf[(4 + LEN1)..].copy_from_slice(&sender.src_address);
                        usb.write(&buf[..]).ok();
                        usb.flush().ok();
                    }
                });
            }
        }
        if let Some(waiter) = critical_section::with(|cs| {
            if let Some(cmd) = PENDING_RENAME.borrow_ref_mut(cs).take() {
                const MSG_LEN: usize = 1 + core::mem::size_of::<RenamePacket>();
                let mut send_buf = [0u8; MSG_LEN];
                send_buf[0] = 0x01;
                send_buf[1..].copy_from_slice(bytemuck::bytes_of(&cmd));
                let waiter = esp_now.send(&connected_brain_addr, &send_buf).unwrap();
                Some(waiter)
            } else {
                None
            }
        }) {
            waiter.wait().unwrap();
            usb_println!("received response");
        }

        if let Some(waiter) = critical_section::with(|cs| {
            if let Some(cmd) = PENDING_CMD.borrow_ref_mut(cs).take() {
                usb_println!(
                    "{} {:?}",
                    core::mem::size_of::<CommandPacket>(),
                    &[&[0], bytemuck::bytes_of(&cmd)].concat()
                );
                if cmd.connect_to_mac != connected_brain_addr {
                    if cmd.connect_to_mac != [0; 6] {
                        connected_brain_addr = cmd.connect_to_mac;
                        let shake = HandshakeResponse {
                            password: StaticString::new("ニャー"),
                        };
                        const MSG_LEN: usize = 1 + core::mem::size_of::<HandshakeResponse>();
                        let mut send_buf = [0u8; MSG_LEN];
                        send_buf[0] = 0x03;
                        send_buf[1..].copy_from_slice(bytemuck::bytes_of(&shake));
                        let waiter = esp_now.send(&connected_brain_addr, &send_buf).unwrap();
                        Some(waiter)
                    } else {
                        usb_println!("disconnecting");
                        let waiter = esp_now.send(&connected_brain_addr, &[4]).unwrap();
                        connected_brain_addr = [0; 6];
                        Some(waiter)
                    }
                } else if let Ok(waiter) = esp_now.send(
                    &connected_brain_addr,
                    &[&[0], bytemuck::bytes_of(&cmd)].concat(),
                ) {
                    if cmd.restart == 1 {
                        connected_brain_addr = [0; 6]
                    }
                    usb_println!("successfully sent, waiting for response");
                    Some(waiter)
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            waiter.wait().unwrap();
            usb_println!("received response");
        }

        if DO_SCAN.load(Ordering::Relaxed) {
            DO_SCAN.store(false, Ordering::Relaxed);
            usb_println!("scanning");
            let waiter = esp_now.send(&esp_now::BROADCAST_ADDRESS, &[2]).unwrap();
            waiter.wait().unwrap();
        }

        // {
        //     waiter.wait().unwrap();
        //     usb_println!("sent");
        // }

        if start.elapsed().as_secs() >= 1 {
            start = Instant::now();
            usb_println!("Receiver alive");
            critical_section::with(|cs2| {
                if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs2).as_mut() {
                    const LEN: usize = core::mem::size_of::<MessagePacket>();
                    let mut buf = [0u8; LEN + 4]; // big enough for MessagePacket debug
                    buf[0] = MESSAGE;
                    buf[1] = MESSAGE;
                    buf[2..4].copy_from_slice(&(LEN as u16).to_be_bytes());
                    buf[4..].copy_from_slice(bytemuck::bytes_of(&MessagePacket {
                        elapsed_time: start2.elapsed().as_millis() as u32,
                        voltage: start2.elapsed().as_secs() as f32,
                        ..Default::default()
                    }));
                    // usb.write(&buf[..]).ok();
                    // usb.flush().ok();
                }
            });
        }

        delay.delay_millis(1);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    usb_println!("PANIC: {}", info);

    loop {}
}

#[handler]
fn on_receive() {
    usb_println!("hearing ya loud and clear cap'n!");
    critical_section::with(|cs| {
        if let Some(usb) = USB_SERIAL.borrow_ref_mut(cs).as_mut() {
            let mut header = [0; 1];
            usb.read_exact(&mut header).ok();
            match header[0] {
                0 => {
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
                }
                1 => {
                    {
                        let mut buf = [0u8; 50];
                        let mut writer = BufWriter {
                            buf: &mut buf,
                            pos: 0,
                        };
                        let _ = writeln!(writer, "scan received");
                        let len = writer.pos;

                        usb.write(&[PRINT, PRINT]).ok();
                        usb.write(&(len as u16).to_be_bytes()).ok();
                        usb.write(&buf[..len]).ok();
                        usb.flush().ok();
                    }
                    DO_SCAN.store(true, Ordering::Relaxed);
                }
                2 => {}
                3 => {
                    // rename
                    let mut buf = [0u8; core::mem::size_of::<RenamePacket>()];
                    usb.read_exact(&mut buf).ok();
                    let packet: &RenamePacket = bytemuck::from_bytes(&buf);
                    *PENDING_RENAME.borrow_ref_mut(cs) = Some(*packet)
                }
                _ => {}
            }
            usb.reset_rx_packet_recv_interrupt();
        }
    });
}
