use bytemuck::{Pod, Zeroable};
use eframe::egui::{self, Color32, Context, Pos2, Response, Sense, Stroke, Ui, Vec2};
mod fonts;
pub use fonts::*;
#[derive(Default, Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct MessagePacket {
    pub acc: [f32; 3],
    pub gyr: [f32; 3],
    pub temp: f32,
    pub time_since_purge: u32,
    pub elapsed_time: u32,
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
    pub fc_voltage: f32,
    pub fc_current: f32,
    pub batt_flags: u16,
    pub fc_flags: u16,
    pub energy_wh: f32,
    pub hydro_wh: f32,
    pub a1_voltage: f32,
    pub free_heap: u32,
    pub test_data: u32,
}
#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CommandPacket {
    pub purge_time: u32,
    pub purge_interval: u32,
    pub fan_speed: u8,
    pub restart: u8,
    pub mac_no: u8,
    pub _pad: [u8; 5],
}
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

pub fn show_log(
    ctx: &Context,
    cpckt: &mut CommandPacket,
    mac_no: &mut u8,
    msg: &MessagePacket,
    log: &[String],
) {
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
                ui.horizontal(|ui| {
                    ui.label("connect to brain: ");
                    if ui.selectable_value(mac_no, 1, "1").clicked() {
                        cpckt.mac_no = 1;
                    }
                    if ui.selectable_value(mac_no, 2, "2").clicked() {
                        cpckt.mac_no = 2;
                    }
                    if ui.selectable_value(mac_no, 3, "3").clicked() {
                        cpckt.mac_no = 3;
                    }
                });
            });

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in log.iter() {
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
// TODO: swappable mac addr
#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ControlPacket {
    pub forward: f32,
    pub steer: f32,
}
