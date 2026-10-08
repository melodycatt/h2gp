use std::{
    collections::VecDeque,
    io::Read,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{self, IconData};
use interface::*;
use util_types::*;

const MESSAGE: u8 = 0x01;
const PRINT: u8 = 0x0;
// #[unsafe(no_mangle)]
fn main() -> eframe::Result<()> {
    let conf = config::load();

    let mut raw_port = serialport::new(conf.port.clone(), 115_200)
        .timeout(Duration::from_millis(10000))
        .open()
        .unwrap();

    raw_port.clear(serialport::ClearBuffer::All).unwrap();
    raw_port.write_data_terminal_ready(false).unwrap();
    raw_port.write_data_terminal_ready(true).unwrap();

    let port = Arc::new(Mutex::new(raw_port));
    let (msg_tx, msg_rx) = crossbeam::channel::unbounded();
    let (scan_tx, scan_rx) = crossbeam::channel::unbounded();
    let (log_tx, log_rx) = crossbeam::channel::unbounded();

    {
        let port = Arc::clone(&port);

        thread::spawn(move || {
            let mut log_len = 0;
            loop {
                if let Ok(mut port) = port.lock() {
                    while port.bytes_to_read().unwrap() > 0 {
                        let mut header = [0; 4];
                        port.read_exact(&mut header).expect("buh");
                        match header[0..2] {
                            [MESSAGE, MESSAGE] => {
                                let s = u16::from_be_bytes(header[2..4].try_into().unwrap());
                                if s != std::mem::size_of::<MessagePacket>() as u16 {
                                    panic!("wrong size")
                                }
                                let mut data = [0; std::mem::size_of::<MessagePacket>()];
                                port.read_exact(&mut data).unwrap();
                                let message: MessagePacket = *bytemuck::from_bytes(&data);
                                msg_tx.send(message).unwrap();
                            }
                            [PRINT, PRINT] => {
                                // println!("{header:?}");
                                let len = u16::from_be_bytes(header[2..4].try_into().unwrap());
                                if len > 300 {
                                    println!("{len}");
                                    println!("discarding");
                                    continue;
                                }
                                let mut buf = vec![0; len as usize];
                                port.read_exact(&mut buf).ok();
                                log_tx
                                    .send(format!(
                                        "{} ; {}",
                                        log_len,
                                        String::from_utf8(buf).unwrap()
                                    ))
                                    .unwrap();
                                log_len += 1;
                            }
                            [SCAN, SCAN] => {
                                let len = u16::from_be_bytes(header[2..4].try_into().unwrap());
                                if len
                                    != (core::mem::size_of::<HandshakeInit>()
                                        + core::mem::size_of::<[u8; 6]>())
                                        as u16
                                {
                                    panic!("wrong scan packet size!");
                                }
                                println!("yay!");
                                let mut data = [0; std::mem::size_of::<HandshakeInit>()];
                                port.read_exact(&mut data).unwrap();
                                println!("{data:?}");
                                let message: HandshakeInit = *bytemuck::from_bytes(&data);
                                println!("{message:#?}");
                                let mut mac = [0; 6];
                                port.read_exact(&mut mac).unwrap();
                                scan_tx.send((message, mac)).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
    let icon = image::load_from_memory(include_bytes!("../icon.jpg"))
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_decorations(false)
            .with_maximized(true)
            .with_active(true)
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            }),
        ..Default::default()
    };

    eframe::run_native(
        "H2GP telemetry",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx, Colors::REDBACK);
            let img = image::load_from_memory(include_bytes!("./img/redback.png"))
                .unwrap()
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            let texture = cc
                .egui_ctx
                .load_texture("redback", color_image, Default::default());
            let img = image::load_from_memory(include_bytes!("./img/miku.png"))
                .unwrap()
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            let miku = cc
                .egui_ctx
                .load_texture("miku", color_image, Default::default());
            let mut message_history = VecDeque::new();
            message_history.push_back(MessagePacket::default());
            let app = App {
                last_stick: 0.,
                last_stick_net: 0.,
                long_graphs: false,
                // shared_msg,
                msg_rx,
                log_rx,
                cpckt: CommandPacket {
                    purge_time: conf.purge_time,
                    purge_interval: conf.purge_interval,
                    fan_speed: 0,
                    restart: 0,
                    connect_to_mac: conf.default_addr,
                    send_wait: conf.send_wait,
                    _pad: [0; 3],
                },
                connected_brain: conf.default_addr,
                send_wait: conf.send_wait != 0,
                message_history,
                start: Instant::now(),
                port,
                lap_count: 0,
                fc_count: 0,
                texture,
                motion_mode: false,
                commands_visible: true,
                log_visible: true,
                colors: Colors::REDBACK,
                miku_mode: false,
                miku,
                d: 0.,
                last_millis: 0,
                ben: true,
                fcen: true,
                long_term: Vec::new(),
                log: Vec::new(),
                count: 0,
                port_path: conf.port,
                scan_rx,
                scanned_brains: Vec::new(),
                brain_rename: String::new(),
            };

            Ok(Box::new(app))
        }),
    )
}
