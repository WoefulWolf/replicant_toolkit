use std::path::PathBuf;

pub trait HasSystemPath {
    fn path(&self) -> &PathBuf;
}
pub trait HasUI {
    fn title(&self) -> String {
        "Unknown".to_string()
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {

    }

    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {

    }

    fn paint_floating(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {

    }
}

pub trait SystemFile: HasSystemPath + HasUI {}

pub trait ReplicantFile: HasUI {}
pub trait ReplicantResourceFile: HasUI + HasResource {}


pub trait HasResource {
    fn get_resource_size(&self) -> u32 {
        0
    }

    fn set_resource(&mut self, resource: Vec<u8>) {

    }

    fn resource_preview(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {

    }
}

pub trait IsBXONAsset: HasUI + HasResource {}