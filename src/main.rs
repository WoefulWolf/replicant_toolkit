use eframe::egui;

mod app;
mod traits;
mod generic_file;
mod bxon;
mod util;
mod tp_archive_file_param;
mod zstd;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "NieR Replicant ver.1.2247... Toolkit",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::<app::ReplicantToolkit>::default())
        }),
    )
}