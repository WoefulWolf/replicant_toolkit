use std::path::PathBuf;

pub trait HasSystemPath {
    fn path(&self) -> &PathBuf;
}
pub trait HasUI {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts);
}

pub trait HasTopBarUI {
    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts);
}

pub trait HasWindowTitle {
    fn window_title(&self) -> String;
}

pub trait SystemFile: HasSystemPath + HasUI + HasTopBarUI + HasWindowTitle {}

pub trait ReplicantFile: HasUI + HasTopBarUI + HasWindowTitle {}

pub trait IsBXONAsset: HasUI + HasTopBarUI {}