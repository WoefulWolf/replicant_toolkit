use std::path::PathBuf;

use crate::traits::*;

pub mod generic_file;
pub mod bxon;
pub mod tp_archive_file_param;
pub mod zstd;
pub mod pack;
pub mod tp_gx_tex_head;

struct UnknownFile {}

pub struct UnknownFileManager {
    path: PathBuf,
    runtime: tokio::runtime::Handle,

    unknown_file: UnknownFile
}

impl UnknownFileManager {
    pub fn new(path: PathBuf, runtime: tokio::runtime::Handle) -> Result<Self, std::io::Error> {
        Ok(Self {
            path,
            runtime,
            unknown_file: UnknownFile {}
        })
    }
}

impl Manager for UnknownFileManager {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label("Unknown Replicant file");
    }

    fn title(&self) -> String {
        format!("{} Unknown", egui_phosphor::regular::SEAL_QUESTION)
    }
}

impl Resource for UnknownFileManager {}

impl ResourceManager for UnknownFileManager {}