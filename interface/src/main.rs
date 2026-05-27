use std::{
    collections::VecDeque,
    f32::consts::PI,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{self, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use interface::*;

const MESSAGE: u8 = 0x01;
const PRINT: u8 = 0x0;

const HISTORY: usize = 500;

#[allow(dead_code)]
struct App {
    ctrl: ControlPacket,
    // shared_msg: Arc<Mutex<(MessagePacket, Vec<String>)>>,
    log_rx: crossbeam::channel::Receiver<String>,
    msg_rx: crossbeam::channel::Receiver<MessagePacket>,
    cpckt: CommandPacket,
    mac_no: u8,
    // purge_time: u32,
    // purge_interval: u32,
    // fan_speed: u8,
    lap_count: usize,
    fc_count: usize,

    message_history: VecDeque<MessagePacket>,

    start: Instant,
    port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    miku: egui::TextureHandle,
    texture: egui::TextureHandle,

    motion_mode: bool,
    commands_visible: bool,
    log_visible: bool,
    colors: Colors,
    miku_mode: bool,
    d: f32,
    last_millis: u32,

    ben: bool,
    fcen: bool,
    log: Vec<String>,
    count: usize,
    long_term: Vec<MessagePacket>,
    long_graphs: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        const FLIPPED_GRAVITY: glam::Vec3 = glam::Vec3::new(0., 0., 1.);
        let idle = self.msg_rx.is_empty();
        let msgs = self.msg_rx.try_iter().map(|mut msg| {
            [msg.acc[1], msg.acc[2]] = [msg.acc[2], -msg.acc[1]];
            msg.gyr = msg.gyr.map(|x| x * PI / 180.);
            self.count += 1;
            if self.count == 35 {
                self.count = 0;
                self.long_term.push(msg.clone());
            }
            msg
        });
        self.log.extend(self.log_rx.try_iter());
        // let smsg = self.shared_msg.lock().unwrap();
        // let mut msg = smsg.0;
        // let log = &smsg.1;

        self.message_history.extend(msgs);
        if self.message_history.len() > HISTORY {
            self.message_history
                .drain(..(self.message_history.len() - HISTORY));
        }
        let msg = self.message_history.back().unwrap();

        if self.log_visible {
            show_log(
                ctx,
                &mut self.cpckt,
                &mut self.mac_no,
                self.message_history
                    .back()
                    .unwrap_or(&MessagePacket::default()),
                &self.log,
            )
        }
        // const BASE_SIZE: egui::Vec2 = egui::vec2(164.8, 114.5);

        egui::Window::new("switcher")
            .resizable(false)
            .default_pos(egui::pos2(15., 15.))
            .max_width(117.625)
            .title_bar(false)
            .frame(
                egui::Frame::new()
                    .fill(self.colors.text_med)
                    .stroke(egui::Stroke::new(1.0, self.colors.text_dark)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let (col, tex) = if self.miku_mode {
                        (Colors::REDBACK, self.miku.clone())
                    } else {
                        (Colors::MIKU, self.texture.clone())
                    };
                    let size = tex.size_vec2();
                    let ar = size.y / size.x;
                    if ui
                        .add(
                            egui::Image::new((tex.id(), egui::vec2(100.0, 100. * ar)))
                                .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        apply_theme(ctx, col.clone());
                        self.colors = col;
                        self.miku_mode = !self.miku_mode;
                    }
                });

                // ui.horizontal(|ui| {
                //     let button2 = egui::Button::new("graphs");
                //     let button = egui::Button::new("motion");
                //
                //     if ui.add_enabled(self.motion_mode, button2).clicked() {
                //         self.motion_mode = false
                //     }
                //     if ui.add_enabled(!self.motion_mode, button).clicked() {
                //         self.motion_mode = true
                //     }
                // });

                let text_xl = hex("#e8f2ff");
                ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(1.5, text_xl);
                ui.visuals_mut().widgets.active.fg_stroke = Stroke::new(1.5, text_xl);
                ui.visuals_mut().widgets.hovered.fg_stroke = Stroke::new(1.5, text_xl);
                ui.visuals_mut().widgets.noninteractive.fg_stroke =
                    Stroke::new(1.5, self.colors.bg);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.motion_mode, "");
                    ui.add_space(-10.);
                    ui.label("motion")
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.commands_visible, "");
                    ui.add_space(-10.);
                    ui.label("commands")
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.log_visible, "");
                    ui.add_space(-10.);
                    ui.label("log")
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.long_graphs, "");
                    ui.add_space(-10.);
                    ui.label("long term graphs")
                });

                // println!("{}", ui.min_size().x)
                // })
            });

        if self.motion_mode {
            egui::SidePanel::right("motion mode").show(ctx, |ui| {
                ui.set_width(250.);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let acc_vec: glam::Vec3 = msg.acc.into();
                    let acc_mag = acc_vec.length();
                    let dot = acc_vec.dot(FLIPPED_GRAVITY);
                    let flipped = dot / acc_mag > 0.8;

                    ui.label(format!(
                        "aa!! aaa! you are {}flipped",
                        if flipped { "" } else { "NOT " }
                    ));
                    PolarGrid::default().show(
                        ui,
                        egui::Vec2 {
                            x: msg.acc[0],
                            y: msg.acc[1],
                        },
                        egui::Vec2 { x: 250., y: 250. },
                        self.colors,
                    );

                    cube_widget(ui, msg.gyr, egui::vec2(250.0, 250.0), self.colors);
                    // ui.set_width(ui.min_size().x);
                    // ui.shrink_height_to_current()
                    if ui.button("reset").clicked() {
                        self.d = 0.;
                    }
                    ui.label(format!("{}", self.d));
                    let delta = (msg.elapsed_time - self.last_millis) as f32 / 1000.;
                    self.last_millis = msg.elapsed_time;
                    self.d += (-msg.acc[1] - 1.) * 9.8 * delta;
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.override_text_color = Some(self.colors.text);
            if self.long_graphs {
                graphs_long(self, ui)
            } else {
                graphs(self, ui);
            }
        });

        // ctx.input(|z| {
        // if z.key_released(egui::Key::ArrowUp) {
        //     self.ctrl.forward = 10.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // if z.key_released(egui::Key::ArrowDown) {
        //     self.ctrl.forward = 10.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // if z.key_pressed(egui::Key::ArrowUp) {
        //     self.ctrl.forward = 90.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // if z.key_pressed(egui::Key::ArrowDown) {
        //     self.ctrl.forward = 0.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // if z.key_pressed(egui::Key::ArrowRight) {
        //     self.ctrl.steer = 75.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // if z.key_pressed(egui::Key::ArrowLeft) {
        //     self.ctrl.steer = 25.;
        //     if let Ok(mut port) = self.port.lock() {
        //         port.write_all(&[1]);
        //         let _ = port.write_all(bytemuck::bytes_of(&self.ctrl));
        //     }
        // }
        // });

        if self.commands_visible {
            egui::Window::new("command & control")
                .resizable(false)
                .movable(true)
                .default_pos([0., 1500.])
                .collapsible(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    let style = ui.style_mut();
                    style.text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::new(16., egui::FontFamily::Monospace),
                    );
                    style.text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(16., egui::FontFamily::Monospace),
                    );

                    egui::Grid::new("command grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("purge interval:");
                            ui.add(
                                egui::DragValue::new(&mut self.cpckt.purge_interval)
                                    .custom_formatter(|x, _| format!("{:05?}", x as u32)),
                            );

                            ui.end_row();
                            ui.label("purge time:");
                            ui.add(
                                egui::DragValue::new(&mut self.cpckt.purge_time)
                                    .custom_formatter(|x, _| format!("{:05?}", x as u32)),
                            );
                            ui.end_row();
                        });

                    if ui.button("send command").clicked() {
                        self.cpckt.restart = 0;
                        self.cpckt.mac_no = 0;

                        if let Ok(mut port) = self.port.lock() {
                            port.write_all(&[0]);
                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                        }
                    }
                    ui.add_space(10.);

                    if ui.button("restart brain").clicked() {
                        self.cpckt.restart = 1;
                        self.cpckt.mac_no = 0;

                        if let Ok(mut port) = self.port.lock() {
                            port.write_all(&[0]);
                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                        }
                    }
                    if ui.button("restart receiver").clicked() {
                        if let Ok(mut port) = self.port.lock() {
                            port.write_data_terminal_ready(false).unwrap();
                            thread::sleep(Duration::from_millis(100));
                            port.write_data_terminal_ready(true).unwrap();
                        }
                    }
                    if ui.button("connect").clicked() {
                        self.cpckt.restart = 0;
                        if let Ok(mut port) = self.port.lock() {
                            port.write_all(&[0]);
                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                        }
                    }
                });
        }

        ctx.request_repaint_after(Duration::from_millis(if idle { 100 } else { 1000 / 30 }));
    }
}

// #[unsafe(no_mangle)]
fn main() -> eframe::Result<()> {
    const PORT_PATH: &str = "/dev/tty.usbmodem1101";
    let args: Vec<String> = std::env::args().collect();
    let ppath = if args.len() > 1 {
        println!("{:?}", &args[1]);
        &args[1]
    } else {
        PORT_PATH
    };

    let mut raw_port = serialport::new(ppath, 115_200)
        .timeout(Duration::from_millis(10000))
        .open()
        .unwrap();

    raw_port.clear(serialport::ClearBuffer::All).unwrap();
    raw_port.write_data_terminal_ready(false).unwrap();
    raw_port.write_data_terminal_ready(true).unwrap();
    // thread::sleep(Duration::from_secs(1));

    let port = Arc::new(Mutex::new(raw_port));
    let (msg_tx, msg_rx) = crossbeam::channel::unbounded();
    let (log_tx, log_rx) = crossbeam::channel::unbounded();
    // let shared_msg = Arc::new(Mutex::new((
    //     MessagePacket::default(),
    //     vec!["nothin to report boss".to_owned()],
    // )));
    //let log = Arc::new(Mutex::new(Vec::new()));

    // ---- Reader Thread ----
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
                                let message: MessagePacket =
                                    bytemuck::from_bytes::<MessagePacket>(&data).clone();
                                msg_tx.send(message).unwrap();
                            }
                            [PRINT, PRINT] => {
                                println!("{header:?}");
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
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_decorations(false)
            .with_maximized(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Telemetry GUI",
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
                ctrl: ControlPacket::default(),
                long_graphs: false,
                // shared_msg,
                msg_rx,
                log_rx,
                cpckt: CommandPacket {
                    purge_time: 150,
                    purge_interval: 20_000,
                    fan_speed: 0,
                    restart: 0,
                    mac_no: 0,
                    _pad: [0; 5],
                },
                mac_no: 3,
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
            };
            Ok(Box::new(app))
        }),
    )
}
fn graphs_long(app: &mut App, ui: &mut Ui) {
    ui.columns(2, |ui| {
        Plot::new("voltage")
            // .height(500.)
            .view_aspect(1.75)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .y_axis_label("voltage")
            .allow_axis_zoom_drag(false)
            .show(&mut ui[0], |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "battery",
                        PlotPoints::from_iter(
                            app.long_term
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.voltage as f64]),
                        ),
                    )
                    .color(app.colors.accent),
                );
                plot_ui.line(
                    Line::new(
                        "fuel cell",
                        PlotPoints::from_iter(
                            app.long_term
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.fc_voltage as f64]),
                        ),
                    )
                    .color(app.colors.accent_xl),
                );
            });

        Plot::new("fc data")
            .view_aspect(1.75)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .y_axis_label("current")
            .allow_axis_zoom_drag(false)
            .show(&mut ui[1], |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "battery",
                        PlotPoints::from_iter(
                            app.long_term
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.current as f64]),
                        ),
                    )
                    .color(app.colors.accent),
                );
                plot_ui.line(
                    Line::new(
                        "fuel cell",
                        PlotPoints::from_iter(
                            app.long_term
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.fc_current as f64]),
                        ),
                    )
                    .color(app.colors.accent_xl),
                );
            });
    });
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.allocate_ui(egui::Vec2::new(ui.available_width() * 0.5, 0.), |ui| {
            Plot::new("temp")
                .view_aspect(1.75)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .y_axis_label("temperature")
                .allow_axis_zoom_drag(false)
                .show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new(
                            "fuel cell",
                            PlotPoints::from_iter(
                                app.long_term
                                    .iter()
                                    .enumerate()
                                    .map(|x| [x.0 as f64, x.1.a1_voltage as f64]),
                            ),
                        )
                        .color(app.colors.accent_xl),
                    );
                    plot_ui.line(
                        Line::new(
                            "imu",
                            PlotPoints::from_iter(
                                app.long_term
                                    .iter()
                                    .enumerate()
                                    .map(|x| [x.0 as f64, x.1.temp as f64]),
                            ),
                        )
                        .color(app.colors.accent),
                    );
                });
        })
    });
}
fn graphs(app: &mut App, ui: &mut Ui) {
    ui.columns(2, |ui| {
        Plot::new("voltage")
            // .height(500.)
            .view_aspect(1.75)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .y_axis_label("voltage")
            .allow_axis_zoom_drag(false)
            .show(&mut ui[0], |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "battery",
                        PlotPoints::from_iter(
                            app.message_history
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.voltage as f64]),
                        ),
                    )
                    .color(app.colors.accent),
                );
                plot_ui.line(
                    Line::new(
                        "fuel cell",
                        PlotPoints::from_iter(
                            app.message_history
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.fc_voltage as f64]),
                        ),
                    )
                    .color(app.colors.accent_xl),
                );
            });

        Plot::new("fc data")
            .view_aspect(1.75)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .y_axis_label("current")
            .allow_axis_zoom_drag(false)
            .show(&mut ui[1], |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "battery",
                        PlotPoints::from_iter(
                            app.message_history
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.current as f64]),
                        ),
                    )
                    .color(app.colors.accent),
                );
                plot_ui.line(
                    Line::new(
                        "fuel cell",
                        PlotPoints::from_iter(
                            app.message_history
                                .iter()
                                .enumerate()
                                .map(|x| [x.0 as f64, x.1.fc_current as f64]),
                        ),
                    )
                    .color(app.colors.accent_xl),
                );
            });
    });
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.allocate_ui(egui::Vec2::new(ui.available_width() * 0.5, 0.), |ui| {
            Plot::new("temp")
                .view_aspect(1.75)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .y_axis_label("temperature")
                .allow_axis_zoom_drag(false)
                .show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new(
                            "fuel cell",
                            PlotPoints::from_iter(
                                app.message_history
                                    .iter()
                                    .enumerate()
                                    .map(|x| [x.0 as f64, x.1.a1_voltage as f64]),
                            ),
                        )
                        .color(app.colors.accent_xl),
                    );
                    plot_ui.line(
                        Line::new(
                            "imu",
                            PlotPoints::from_iter(
                                app.message_history
                                    .iter()
                                    .enumerate()
                                    .map(|x| [x.0 as f64, x.1.temp as f64]),
                            ),
                        )
                        .color(app.colors.accent),
                    );
                });
        })
    });
}
