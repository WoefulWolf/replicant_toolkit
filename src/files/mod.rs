use crate::traits::{HasResource, HasUI, ReplicantFile, ReplicantResourceFile};

pub mod generic_file;
pub mod bxon;
pub mod tp_archive_file_param;
pub mod zstd;
pub mod pack;
pub mod tp_gx_tex_head;

struct UnknownReplicantFile {
}

impl HasUI for UnknownReplicantFile {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label("Unknown Replicant file");
    }

    fn title(&self) -> String {
        format!("{} Unknown", egui_phosphor::regular::SEAL_QUESTION)
    }
}

impl HasResource for UnknownReplicantFile {}

impl ReplicantResourceFile for UnknownReplicantFile {}
impl ReplicantFile for UnknownReplicantFile {}