use std::path::PathBuf;

pub trait HasSystemPath {
    fn path(&self) -> &PathBuf;
}
pub trait HasUI {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts);
}

pub trait SystemFile: HasSystemPath + HasUI {}

pub trait ReplicantFile: HasUI {}

pub trait IsBXONAsset: HasUI {}