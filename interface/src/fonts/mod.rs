// Source - https://stackoverflow.com/a/79116040
// Posted by user284898
// Retrieved 2026-03-19, License - CC BY-SA 4.0

use std::sync::Arc;

use eframe::egui;
use eframe::egui::{FontData, FontDefinitions, FontFamily};
// use std::collections::BTreeMap;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    //
    // Load font =====================================
    //
    fonts.font_data.insert(
        "sauce".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "./SauceCodeProNerdFontMono-Regular.ttf"
        ))),
    );

    // let mut newfam = BTreeMap::new();
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "sauce".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "sauce".to_owned());

    // newfam.insert(FontFamily::Name("jost".into()), vec!["jost".to_owned()]);
    // fonts.families.append(&mut newfam);

    //
    // Set fonts =====================================
    //
    ctx.set_fonts(fonts);
}
