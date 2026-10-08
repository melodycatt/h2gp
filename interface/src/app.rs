use std::{f32::consts::PI, io::Write, thread, time::Duration};

use super::*;
use crate::config::{Config, save};
use eframe::egui::{Layout, RichText};
use egui_plot::{Line, Plot, PlotPoints};
const HISTORY: usize = 500;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.start.elapsed().as_millis() < 1000 {
            apply_theme(ctx, self.colors)
        };
        const FLIPPED_GRAVITY: glam::Vec3 = glam::Vec3::new(0., 0., 1.);
        let idle = self.msg_rx.is_empty();
        let msgs = self.msg_rx.try_iter().map(|mut msg| {
            [msg.acc[1], msg.acc[2]] = [msg.acc[2], -msg.acc[1]];
            msg.gyr = msg.gyr.map(|x| x * PI / 180.);
            self.count += 1;
            if self.count == 35 {
                self.count = 0;
                self.long_term.push(msg);
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

        if self.log_visible {
            show_log(ctx, self)
        }

        let msg = self.message_history.back().unwrap();
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
                        apply_theme(ctx, col);
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

        egui::Window::new("connect to a brain")
            .resizable(true)
            .movable(true)
            .default_pos([500., 500.])
            .collapsible(true)
            .show(ctx, |ui| {
                if ui.button("scan for a brain").clicked()
                    && let Ok(mut port) = self.port.lock()
                {
                    self.scanned_brains = Vec::with_capacity(self.scanned_brains.len());
                    let _ = port.write_all(&[1]);
                }
                self.scanned_brains.extend(self.scan_rx.try_iter());
                for i in self.scanned_brains.iter() {
                    egui::Frame::new()
                        .stroke(Stroke::new(1., self.colors.text))
                        .show(ui, |ui| {
                            ui.label(RichText::new(i.0.name.as_str()).size(20.).strong());
                            egui::Grid::new("brain descriptor").show(ui, |ui| {
                                ui.label("address: ");
                                ui.label(format!("{:?}", i.1));
                                ui.end_row();
                                ui.label("available slots: ");
                                ui.label(format!("{}", i.0.available_slots));
                            });
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                if i.1 == self.connected_brain {
                                    if ui.button("disconnect").clicked() {
                                        self.connected_brain = [0; 6];
                                        self.cpckt.restart = 0;
                                        self.cpckt.connect_to_mac = [0; 6];
                                        if let Ok(mut port) = self.port.lock() {
                                            let _ = port.write_all(&[0]);
                                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                                        }
                                        // let _ = port.write_all(&[2]);
                                    }
                                    if ui.button("rename").clicked()
                                        && let Ok(mut port) = self.port.lock()
                                    {
                                        let _ = port.write_all(&[3]);
                                        let _ = port.write_all(bytemuck::bytes_of(&RenamePacket {
                                            new_name: StaticString::new(&self.brain_rename),
                                        }));
                                    }
                                    ui.text_edit_singleline(&mut self.brain_rename);
                                } else if i.0.available_slots > 0 {
                                    if ui.button("connect").clicked() {
                                        self.cpckt.restart = 0;
                                        self.cpckt.connect_to_mac = i.1;
                                        self.connected_brain = i.1;
                                        if let Ok(mut port) = self.port.lock() {
                                            let _ = port.write_all(&[0]);
                                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                                        }
                                    }
                                }
                            })
                        });
                }
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
                                    .custom_formatter(|x, _| format!("{:05?}", x as u32))
                                    .speed(0.),
                            );

                            ui.end_row();
                            ui.label("purge time:");
                            ui.add(
                                egui::DragValue::new(&mut self.cpckt.purge_time)
                                    .custom_formatter(|x, _| format!("{:05?}", x as u32))
                                    .speed(0.),
                            );
                            ui.end_row();
                            ui.add_space(10.);

                            // ui.end_row();
                            // ui.label("receiver 1:");
                            // if ui
                            //     .add(egui::DragValue::new(&mut self.receivers[0]).speed(0.))
                            //     .changed()
                            // {
                            //     self.cpckt.receivers[0] = self.receivers[0];
                            // };
                            // ui.end_row();
                            // ui.label("receiver 2:");
                            // if ui
                            //     .add(egui::DragValue::new(&mut self.receivers[1]).speed(0.))
                            //     .changed()
                            // {
                            //     self.cpckt.receivers[1] = self.receivers[1];
                            // };
                            ui.end_row();
                            ui.label("send wait:");
                            if ui.checkbox(&mut self.send_wait, "").changed() {
                                self.cpckt.send_wait = self.send_wait as u8;
                            };
                            ui.end_row();
                        });

                    if ui.button("send command").clicked() {
                        self.cpckt.restart = 0;

                        if let Ok(mut port) = self.port.lock() {
                            let _ = port.write_all(&[0]);
                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                        }
                    }
                    ui.add_space(10.);

                    if ui.button("restart brain").clicked() {
                        self.cpckt.restart = 1;
                        self.connected_brain = [0; 6];
                        if let Ok(mut port) = self.port.lock() {
                            let _ = port.write_all(&[0]);
                            let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                        }
                    }
                    if ui.button("restart receiver").clicked()
                        && let Ok(mut port) = self.port.lock()
                    {
                        self.connected_brain = [0; 6];
                        self.cpckt.restart = 0;
                        self.cpckt.connect_to_mac = [0; 6];
                        let _ = port.write_all(&[0]);
                        let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));

                        thread::sleep(Duration::from_millis(100));
                        port.write_data_terminal_ready(false).unwrap();
                        thread::sleep(Duration::from_millis(100));
                        port.write_data_terminal_ready(true).unwrap();
                    }
                    ui.add_space(15.);
                    // ui.label("brain #:");
                    // if ui
                    //     .add(egui::DragValue::new(&mut self.connected_brain).speed(0.))
                    //     .changed()
                    // {
                    //     self.cpckt.connect_to_mac = self.connected_brain;
                    // };
                    // if ui.button("connect").clicked() {
                    //     self.cpckt.restart = 0;
                    //     if let Ok(mut port) = self.port.lock() {
                    //         let _ = port.write_all(&[0]);
                    //         let _ = port.write_all(bytemuck::bytes_of(&self.cpckt));
                    //     }
                    // }
                });
        }

        ctx.request_repaint_after(Duration::from_millis(if idle { 200 } else { 1000 / 20 }));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        save(&Config {
            port: self.port_path.clone(),
            default_addr: [0; 6],
            purge_time: self.cpckt.purge_time,
            purge_interval: self.cpckt.purge_interval,
            send_wait: self.send_wait as u8,
        })
        .expect("PLEASE")
    }
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
                    // plot_ui.line(
                    //     Line::new(
                    //         "fuel cell",
                    //         PlotPoints::from_iter(
                    //             app.long_term
                    //                 .iter()
                    //                 .enumerate()
                    //                 .map(|x| [x.0 as f64, x.1.a1_voltage as f64]),
                    //         ),
                    //     )
                    //     .color(app.colors.accent_xl),
                    // );
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
                    // plot_ui.line(
                    //     Line::new(
                    //         "fuel cell",
                    //         PlotPoints::from_iter(
                    //             app.message_history
                    //                 .iter()
                    //                 .enumerate()
                    //                 .map(|x| [x.0 as f64, x.1.a1_voltage as f64]),
                    //         ),
                    //     )
                    //     .color(app.colors.accent_xl),
                    // );
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
