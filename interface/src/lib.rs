use eframe::egui::{
    self, Color32, Context, Pos2, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
};
mod fonts;
pub use fonts::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

mod app;

pub mod config;
use util_types::*;
pub struct PolarGrid {
    pub radius_steps: usize,
    pub angle_steps: usize,
    pub scale: f32,

    pub axis_color: Color32,
    pub grid_color: Color32,
    pub point_color: Color32,
    pub background: Color32,
}

impl Default for PolarGrid {
    fn default() -> Self {
        Self {
            radius_steps: 5,
            angle_steps: 16,
            scale: 50.0,
            axis_color: Color32::WHITE,
            grid_color: Color32::from_gray(80),
            point_color: Color32::LIGHT_BLUE,
            background: Color32::from_gray(20),
        }
    }
}

impl PolarGrid {
    pub fn show(self, ui: &mut Ui, point: Vec2, size: Vec2, colors: Colors) -> Response {
        // let desired_size = egui::vec2(ui.available_width(), ui.available_height());
        // let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 0.0, colors.bg_light);

        let center = rect.center();

        // Helper to convert math coords (+y up) → egui coords (+y down)
        let to_screen = |v: Vec2| -> Pos2 {
            Pos2::new(
                center.x + v.x * self.scale,
                center.y - v.y * self.scale, // flip Y
            )
        };

        // Draw concentric circles
        for i in 1..=self.radius_steps {
            let r = i as f32 * self.scale;
            painter.circle_stroke(center, r, Stroke::new(1.0, colors.text_med));
        }

        // Draw radial lines
        for i in 0..self.angle_steps {
            let angle = i as f32 / self.angle_steps as f32 * std::f32::consts::TAU;
            let dir = Vec2::new(angle.cos(), angle.sin());
            let end = to_screen(dir * self.radius_steps as f32);
            painter.line_segment([center, end], Stroke::new(1.0, colors.text_med));
        }

        // Draw axes
        let max_r = self.radius_steps as f32;

        let x_axis_start = to_screen(Vec2::new(-max_r, 0.0));
        let x_axis_end = to_screen(Vec2::new(max_r, 0.0));
        painter.line_segment([x_axis_start, x_axis_end], Stroke::new(2.0, colors.text));

        let y_axis_start = to_screen(Vec2::new(0.0, -max_r));
        let y_axis_end = to_screen(Vec2::new(0.0, max_r));
        painter.line_segment([y_axis_start, y_axis_end], Stroke::new(2.0, colors.text));

        // Draw point
        let pos = to_screen(point);
        painter.circle_filled(pos, 4.0, colors.accent_xl);

        response
    }
}

// Example usage inside egui:
/*
ui.add_space(10.0);
PolarGrid::default().show(ui, egui::vec2(1.0, 1.5), egui::vec2(300.0, 300.0));
*/

pub fn cube_widget(ui: &mut Ui, mut rot: [f32; 3], size: Vec2, colors: Colors) -> Response {
    (rot[0], rot[1], rot[2]) = (rot[1], rot[2], rot[0]);
    // (rot[0], rot[1]) = (rot[1], rot[0]);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, colors.bg_light);

    let center = rect.center();
    let scale = rect.width().min(rect.height()) * 0.7;
    let camera_dist = 3.0;

    // Cube vertices
    let verts = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];

    // Cube edges
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let mut projected = Vec::with_capacity(8);

    for v in verts {
        let mut p = v;

        // rotate X
        let (sx, cx) = rot[0].sin_cos();
        p = [p[0], p[1] * cx - p[2] * sx, p[1] * sx + p[2] * cx];

        // rotate Y
        let (sy, cy) = rot[1].sin_cos();
        p = [p[0] * cy + p[2] * sy, p[1], -p[0] * sy + p[2] * cy];

        // rotate Z
        let (sz, cz) = rot[2].sin_cos();
        p = [p[0] * cz - p[1] * sz, p[0] * sz + p[1] * cz, p[2]];

        // simple perspective projection
        let z = p[2] + camera_dist;
        let x = p[0] / z;
        let y = p[1] / z;

        projected.push(Pos2::new(center.x + x * scale, center.y - y * scale));
    }

    for &(a, b) in &edges[0..4] {
        painter.line_segment(
            [projected[a], projected[b]],
            Stroke::new(2., colors.accent_l),
        );
    }
    for &(a, b) in &edges[4..] {
        painter.line_segment(
            [projected[a], projected[b]],
            Stroke::new(2.0, colors.accent),
        );
    }

    response
}
#[allow(dead_code)]
pub struct App {
    // shared_msg: Arc<Mutex<(MessagePacket, Vec<String>)>>,
    pub log_rx: crossbeam::channel::Receiver<String>,
    pub msg_rx: crossbeam::channel::Receiver<MessagePacket>,
    pub scan_rx: crossbeam::channel::Receiver<(HandshakeInit, [u8; 6])>,
    pub cpckt: CommandPacket,
    pub connected_brain: [u8; 6],
    pub send_wait: bool,
    // purge_time: u32,
    // purge_interval: u32,
    // fan_speed: u8,
    pub lap_count: usize,
    pub fc_count: usize,

    pub message_history: VecDeque<MessagePacket>,

    pub start: Instant,
    pub port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    pub miku: egui::TextureHandle,
    pub texture: egui::TextureHandle,

    pub motion_mode: bool,
    pub commands_visible: bool,
    pub log_visible: bool,
    pub colors: Colors,
    pub miku_mode: bool,
    pub d: f32,
    pub last_millis: u32,

    pub ben: bool,
    pub fcen: bool,
    pub log: Vec<String>,
    pub count: usize,
    pub long_term: Vec<MessagePacket>,
    pub long_graphs: bool,
    pub port_path: String,
    pub last_stick: f32,
    pub last_stick_net: f32,

    pub brain_rename: String,
    pub scanned_brains: Vec<(HandshakeInit, [u8; 6])>,
}

pub fn show_log(ctx: &Context, state: &mut App) {
    let msg = state.message_history.back().unwrap_or(&MessagePacket {
        acc: [0.; 3],
        gyr: [0.; 3],
        temp: 0.,
        time_since_purge: 0,
        elapsed_time: 0,
        voltage: 0.,
        current: 0.,
        power: 0.,
        fc_voltage: 0.,
        fc_current: 0.,
        batt_flags: 0,
        fc_flags: 0,
        energy_wh: 0.,
        hydro_wh: 0.,
        a1_voltage: 0.,
        free_heap: 0,
        detected_zero: 0,
        current_cal: 0,
    });

    egui::SidePanel::right("log")
        .resizable(true)
        .show(ctx, |ui| {
            let text_xl = hex("#e8f2ff");
            // ui.set_width(250.);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.style_mut().visuals.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, text_xl);

            ui.vertical(|ui| {
                let fmt = format!("{:#?}", msg);
                for line in fmt.lines() {
                    ui.label(line);
                }

                ui.label("WH THIS STICK:");
                egui::Grid::new("wh this stick")
                    .num_columns(2)
                    .max_col_width(100.)
                    .show(ui, |ui| {
                        ui.label("net");
                        ui.label("hydro");
                        ui.end_row();
                        ui.label(format!("{}", msg.energy_wh - state.last_stick_net));
                        ui.label(format!("{}", msg.hydro_wh - state.last_stick));
                        ui.end_row();
                        if ui.button("reset").clicked() {
                            state.last_stick_net = msg.energy_wh;
                        }
                        if ui.button("reset").clicked() {
                            state.last_stick = msg.hydro_wh;
                        }
                        ui.end_row();
                    });
                if ui.button("reset both").clicked() {
                    state.last_stick_net = msg.energy_wh;
                    state.last_stick = msg.hydro_wh;
                }

                // ui.columns(2, |cols| {
                //     cols[0].label(format!(
                //         "WH THIS STICK: {} net, {} hydro",
                //         ,
                //         msg.energy_wh - state.last_stick
                //     ))
                //     cols
                // })
            });

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in state.log.iter() {
                        ui.label(line.trim());
                    }
                });
        });
}

pub fn hex(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    egui::Color32::from_rgb(r, g, b)
}
#[derive(Clone, Copy)]
pub struct Colors {
    pub bg: egui::Color32,
    pub bg_xl: egui::Color32,
    pub bg_light: egui::Color32,
    pub text: egui::Color32,
    pub text_dull: egui::Color32,
    pub text_dark: egui::Color32,
    pub text_med: egui::Color32,
    pub text_light: egui::Color32,

    pub accent: egui::Color32,
    pub accent_l: egui::Color32,
    pub accent_xl: egui::Color32,
}
impl Colors {
    pub const REDBACK: Colors = Colors {
        bg: egui::Color32::from_rgb(0x00, 0x00, 0x00),
        bg_xl: egui::Color32::from_rgb(0x36, 0x00, 0x00),
        bg_light: egui::Color32::from_rgb(0x1a, 0x00, 0x00),

        text: egui::Color32::from_rgb(0xff, 0x00, 0x19),
        text_dull: egui::Color32::from_rgb(0xc8, 0xb7, 0xb7),
        text_dark: egui::Color32::from_rgb(0xad, 0x00, 0x00),
        text_med: egui::Color32::from_rgb(0xd4, 0x00, 0x00),
        text_light: egui::Color32::from_rgb(0xff, 0xab, 0xb3),

        accent: egui::Color32::from_rgb(0xff, 0x77, 0x00),
        accent_l: egui::Color32::from_rgb(0xff, 0xa6, 0x00),
        accent_xl: egui::Color32::from_rgb(0xff, 0xdd, 0x00),
    };

    pub const MIKU: Colors = Colors {
        bg: egui::Color32::from_rgb(0x4b, 0x58, 0x75),
        bg_light: egui::Color32::from_rgb(0x78, 0x7e, 0x90),
        bg_xl: egui::Color32::from_rgb(0xb3, 0xb5, 0xc4),

        text: egui::Color32::from_rgb(0x42, 0xe3, 0xdf),
        text_dull: egui::Color32::from_rgb(0x72, 0xae, 0xb3),
        text_dark: egui::Color32::from_rgb(0x11, 0x99, 0xa2),
        text_med: egui::Color32::from_rgb(0x1e, 0xcd, 0xc8),
        text_light: egui::Color32::from_rgb(0xd8, 0xf4, 0xf5),

        accent: egui::Color32::from_rgb(0xfe, 0xa0, 0xd1),
        accent_l: egui::Color32::from_rgb(0xfe, 0xcd, 0xb2),
        accent_xl: egui::Color32::from_rgb(0xf8, 0xfa, 0x4c),
    };
}

pub fn apply_theme(ctx: &egui::Context, colors: Colors) {
    use egui::{Stroke, Visuals};

    // helper
    // === your palette ===
    // let bg = hex("#00072b");
    // let self.colors.bg = hex("#000000");#4b5875
    // let self.colors.bg_xl = hex("#360000");#787e90
    // let self.colors.bg_light = hex("#1a0000");#b3b5c4
    // let self.colors.text_light = hex("#ffabb3");#d8f4f5
    // let self.colors.text = hex("#ff0019"); #42e3df
    // let self.colors.text_med = hex("#d40000"); #1ecdc8
    // let self.colors.text_dark = hex("#ad0000"); #1199a2
    // let self.colors.text_dull = hex(""#c8b7b7);#72aeb3
    //
    // let self.colors.accent = hex("#ff7700");#fea0d1
    // let self.colors.accent_l = hex("#ffa600");#fecdb2
    // let self.colors.accent_xl = hex("#ffdd00");#f8fa4c

    let mut visuals = Visuals::dark();

    let bg = colors.bg;
    let bg_light = colors.bg_light;
    let bg_xl = colors.bg_xl;

    let text = colors.text;
    let text_dark = colors.text_dark;
    let text_dull = colors.text_dull;
    let text_med = colors.text_med;
    let text_light = colors.text_light;

    let accent_l = colors.accent_l;
    let accent_xl = colors.accent_xl;

    visuals.panel_fill = bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = bg_light; // self.colors.text EDITS (DragValue!)
    visuals.faint_bg_color = bg_light;

    // === text ===
    // visuals.override_text_color = Some(text);
    visuals.weak_text_color = Some(text);

    // === selection ===
    visuals.selection.bg_fill = accent_l;
    visuals.selection.stroke = Stroke::new(1.0, bg_light);

    // === hyperlink ===
    visuals.hyperlink_color = accent_xl;

    // === widgets ===
    let w = &mut visuals.widgets;

    // inactive
    w.inactive.weak_bg_fill = bg;

    // hovered
    w.hovered.weak_bg_fill = bg_light;

    // active
    w.active.weak_bg_fill = bg;

    // open
    w.open.weak_bg_fill = bg_xl;

    w.noninteractive.bg_stroke = Stroke::new(1.0, text_dark);
    w.noninteractive.fg_stroke = Stroke::new(1.0, text_dull);

    // inactive
    w.inactive.bg_fill = bg;
    w.inactive.fg_stroke = Stroke::new(1.0, text);
    w.inactive.bg_stroke = Stroke::new(1.0, text_med);
    w.inactive.corner_radius = 0.0.into();

    // hovered
    w.hovered.bg_fill = bg_xl;
    w.hovered.fg_stroke = Stroke::new(1.5, text_light);
    w.hovered.bg_stroke = Stroke::new(1.0, text_med);
    w.hovered.corner_radius = 0.0.into();

    // active (clicked / dragging)
    w.active.bg_fill = bg;
    w.active.fg_stroke = Stroke::new(1.5, text_light);
    w.active.bg_stroke = Stroke::new(1.0, text);
    w.active.expansion = 0.0;
    w.active.corner_radius = 0.0.into();

    // open (menus, combo boxes)
    w.open.bg_fill = bg_light;
    w.open.fg_stroke = Stroke::new(1.0, text);
    w.open.corner_radius = 6.0.into();

    // === window rounding ===
    visuals.window_corner_radius = 0.0.into();
    setup_custom_fonts(ctx);
    ctx.set_visuals(visuals);
}

pub struct FuelGauge {
    value: f32,
    min: f32,
    max: f32,
    extra_max: f32,
    reserve_fraction: f32,
    size: f32,
}

impl FuelGauge {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            extra_max: (max - min) * 0.1,
            reserve_fraction: 0.15,
            size: 150.0,
        }
    }

    pub fn extra_max(mut self, extra: f32) -> Self {
        self.extra_max = extra.max(0.0);
        self
    }

    pub fn reserve_fraction(mut self, f: f32) -> Self {
        self.reserve_fraction = f.clamp(0.0, 1.0);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(20.0);
        self
    }
}

impl Widget for FuelGauge {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired = Vec2::splat(self.size);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());

        let painter = ui.painter_at(rect);
        let visuals = ui.visuals();

        let center = Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.05);
        let radius = rect.width().min(rect.height()) * 0.42;

        // 240° sweep (more "instrument-like")
        let start_angle = std::f32::consts::PI * 5.0 / 6.0; // 150°
        let end_angle = std::f32::consts::PI * 1.0 / 6.0; // 30°
        let sweep = start_angle - end_angle;

        let drawable_max = self.max + self.extra_max;
        let min = self.min;

        let range = (drawable_max - min).max(f32::EPSILON);

        let value = self.value.clamp(min, drawable_max);
        let t = ((value - min) / range).clamp(0.0, 1.0);

        let angle_at = |t: f32| start_angle - sweep * t;

        let needle_angle = angle_at(t);

        let reserve_t = self.reserve_fraction.clamp(0.0, 1.0);

        let arc_points = |a0: f32, a1: f32, r: f32| {
            let segments = 64;
            (0..=segments)
                .map(|i| {
                    let t = i as f32 / segments as f32;
                    let a = a0 + (a1 - a0) * t;
                    Pos2::new(center.x + a.cos() * r, center.y - a.sin() * r)
                })
                .collect::<Vec<_>>()
        };

        let normal_color = visuals.widgets.inactive.fg_stroke.color;
        let text_color = visuals.text_color();
        let bg_color = visuals.widgets.noninteractive.bg_fill;

        let reserve_color = Color32::RED;
        let overrange_color = Color32::from_rgb(255, 140, 0);

        // ---- Background arc ----
        painter.add(Shape::line(
            arc_points(start_angle, end_angle, radius),
            Stroke::new(radius * 0.06, bg_color),
        ));

        // ---- Normal arc (filled up to max) ----
        let normal_t = ((self.max - min) / range).clamp(0.0, 1.0);

        painter.add(Shape::line(
            arc_points(start_angle, angle_at(normal_t), radius),
            Stroke::new(radius * 0.06, normal_color),
        ));

        // ---- Reserve zone ----
        let reserve_end = (self.max - min) / range * reserve_t;
        painter.add(Shape::line(
            arc_points(angle_at(normal_t), angle_at(reserve_end), radius),
            Stroke::new(radius * 0.08, reserve_color),
        ));

        // ---- Overrange ----
        if self.extra_max > 0.0 {
            painter.add(Shape::line(
                arc_points(angle_at(normal_t), end_angle, radius),
                Stroke::new(radius * 0.06, overrange_color),
            ));
        }

        // ---- Ticks ----
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let a = angle_at(t);

            let is_major = i % 4 == 0;

            let inner_r = if is_major {
                radius * 0.78
            } else {
                radius * 0.86
            };

            let outer_r = radius * 0.95;

            let inner = Pos2::new(center.x + a.cos() * inner_r, center.y - a.sin() * inner_r);

            let outer = Pos2::new(center.x + a.cos() * outer_r, center.y - a.sin() * outer_r);

            painter.line_segment(
                [inner, outer],
                Stroke::new(if is_major { 2.0 } else { 1.0 }, text_color),
            );
        }

        // ---- Needle (triangular) ----
        let needle_len = radius * 0.82;
        let needle_width = radius * 0.06;

        let dir = Vec2::new(needle_angle.cos(), -needle_angle.sin());
        let perp = Vec2::new(-dir.y, dir.x);

        let tip = center + dir * needle_len;
        let left = center + perp * needle_width;
        let right = center - perp * needle_width;

        painter.add(Shape::convex_polygon(
            vec![tip, left, right],
            Color32::from_gray(220),
            Stroke::new(1.0, Color32::BLACK),
        ));

        // ---- Hub ----
        painter.circle_filled(center, radius * 0.08, text_color);
        painter.circle_filled(center, radius * 0.05, bg_color);

        // ---- Center value text ----
        let percent = ((value - min) / (self.max - min).max(f32::EPSILON)) * 100.0;

        painter.text(
            center - Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.0}%", percent.clamp(0.0, 999.0)),
            egui::FontId::proportional(radius * 0.18),
            text_color,
        );

        // ---- Labels ----
        let label_r = radius * 1.08;

        let empty_pos = Pos2::new(
            center.x + start_angle.cos() * label_r,
            center.y - start_angle.sin() * label_r,
        );

        let full_pos = Pos2::new(
            center.x + end_angle.cos() * label_r,
            center.y - end_angle.sin() * label_r,
        );

        painter.text(
            empty_pos,
            egui::Align2::CENTER_CENTER,
            "E",
            egui::FontId::proportional(radius * 0.14),
            text_color,
        );

        painter.text(
            full_pos,
            egui::Align2::CENTER_CENTER,
            "F",
            egui::FontId::proportional(radius * 0.14),
            text_color,
        );

        response
    }
}
