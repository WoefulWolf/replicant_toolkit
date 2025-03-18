use std::path::PathBuf;

use eframe::egui;

use crate::traits::*;

pub struct GenericFile {
    pub path: PathBuf,
}

impl GenericFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl HasSystemPath for GenericFile {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl HasUI for GenericFile {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.label(egui::RichText::new(egui_phosphor::regular::SEAL_QUESTION).size(128.0));
        });
    }

    fn title(&self) -> String {
        format!("{} Unknown", egui_phosphor::regular::FILE)
    }
}
    

impl SystemFile for GenericFile {}