use std::{
    collections::VecDeque,
    f32::consts::PI,
    time::{Duration, Instant},
};

use eframe::egui::{self, Stroke};
use egui_plot::{Line, Plot, PlotPoints};
use interface::*;

// const MESSAGE: u8 = 0x42;
// const PRINT: u8 = 0x41;

const HISTORY: usize = 500;

// TODO: swappable mac addr

#[allow(dead_code)]
struct App {
    cpckt: CommandPacket,
    mac_no: u8,
    // purge_time: u32,
    // purge_interval: u32,
    // fan_speed: u8,
    lap_count: usize,
    fc_count: usize,

    message_history: VecDeque<MessagePacket>,

    start: Instant,
    miku: egui::TextureHandle,
    texture: egui::TextureHandle,

    motion_mode: bool,
    commands_visible: bool,
    log_visible: bool,
    colors: Colors,
    miku_mode: bool,
    d: f32,
    last_millis: u32,
}

impl App {
    fn push(hist: &mut VecDeque<MessagePacket>, y: MessagePacket) {
        hist.push_back(y);
        if hist.len() > HISTORY {
            hist.pop_front();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        const FLIPPED_GRAVITY: glam::Vec3 = glam::Vec3::new(0., 0., 1.);
        let mut msg = MessagePacket::default();
        let log = &["disconnected".to_owned()];

        [msg.acc[1], msg.acc[2]] = [msg.acc[2], -msg.acc[1]];
        msg.gyr = msg.gyr.map(|x| x * PI / 180.);

        Self::push(&mut self.message_history, msg);

        ctx.set_pixels_per_point(2.5);
        if self.log_visible {
            show_log(ctx, self)
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

                    // ui.set_width(ui.min_size().x);
                    // ui.shrink_height_to_current()
                    if ui.button("reset").clicked() {
                        self.d = 0.;
                    }
                    ui.label(format!("{}", self.d));
                    let delta = (msg.elapsed_time - self.last_millis) as f32 / 1000.;
                    self.last_millis = msg.elapsed_time;
                    self.d += (-msg.acc[1] - 1.) * 9.8 * delta;

                    ui.add(FuelGauge::new(8.3, 0., 11.5))
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.override_text_color = Some(self.colors.text);
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
                                    self.message_history
                                        .iter()
                                        .enumerate()
                                        .map(|x| [x.0 as f64, x.1.voltage as f64]),
                                ),
                            )
                            .color(self.colors.accent),
                        );
                        plot_ui.line(
                            Line::new(
                                "fuel cell",
                                PlotPoints::from_iter(
                                    self.message_history
                                        .iter()
                                        .enumerate()
                                        .map(|x| [x.0 as f64, x.1.fc_voltage as f64]),
                                ),
                            )
                            .color(self.colors.accent_xl),
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
                                    self.message_history
                                        .iter()
                                        .enumerate()
                                        .map(|x| [x.0 as f64, x.1.current as f64]),
                                ),
                            )
                            .color(self.colors.accent),
                        );
                        plot_ui.line(
                            Line::new(
                                "fuel cell",
                                PlotPoints::from_iter(
                                    self.message_history
                                        .iter()
                                        .enumerate()
                                        .map(|x| [x.0 as f64, x.1.fc_current as f64]),
                                ),
                            )
                            .color(self.colors.accent_xl),
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
                                        self.message_history
                                            .iter()
                                            .enumerate()
                                            .map(|x| [x.0 as f64, x.1.a1_voltage as f64]),
                                    ),
                                )
                                .color(self.colors.accent_xl),
                            );
                            plot_ui.line(
                                Line::new(
                                    "imu",
                                    PlotPoints::from_iter(
                                        self.message_history
                                            .iter()
                                            .enumerate()
                                            .map(|x| [x.0 as f64, x.1.temp as f64]),
                                    ),
                                )
                                .color(self.colors.accent),
                            );
                        });
                })
            });
        });
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
                    }
                    ui.add_space(10.);

                    if ui.button("restart brain").clicked() {
                        self.cpckt.restart = 1;
                        self.cpckt.mac_no = 0;
                    }
                    if ui.button("restart receiver").clicked() {}
                });
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn main() -> eframe::Result<()> {
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
            let img = image::load_from_memory(include_bytes!("../img/redback.png"))
                .unwrap()
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            let texture = cc
                .egui_ctx
                .load_texture("redback", color_image, Default::default());
            let img = image::load_from_memory(include_bytes!("../img/miku.png"))
                .unwrap()
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            let miku = cc
                .egui_ctx
                .load_texture("miku", color_image, Default::default());
            let app = App {
                cpckt: CommandPacket {
                    purge_time: 150,
                    purge_interval: 20_000,
                    fan_speed: 0,
                    restart: 0,
                    mac_no: 0,
                    _pad: [0; 2],
                    receivers: [0; 2],
                    send_wait: 0,
                },
                mac_no: 1,
                message_history: VecDeque::new(),
                start: Instant::now(),
                lap_count: 0,
                fc_count: 0,
                texture,
                motion_mode: false,
                commands_visible: true,
                log_visible: false,
                colors: Colors::REDBACK,
                miku_mode: false,
                miku,
                d: 0.,
                last_millis: 0,
            };
            Ok(Box::new(app))
        }),
    )
}
