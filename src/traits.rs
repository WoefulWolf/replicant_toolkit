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

pub trait SystemFile: HasSystemPath + HasUI + HasTopBarUI {}

pub trait ReplicantFile: HasUI + HasTopBarUI {}

pub trait IsBXONAsset: HasUI + HasTopBarUI {}