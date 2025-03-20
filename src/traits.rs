use std::path::PathBuf;



pub trait Manager {
    fn path(&self) -> &PathBuf;

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

pub trait Resource {
    fn get_resource_size(&self) -> u32 {
        0
    }

    fn set_resource(&mut self, resource: Vec<u8>) {

    }

    fn resource_preview(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {

    }
}

pub trait ResourceManager: Resource + Manager {}