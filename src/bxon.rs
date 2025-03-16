use std::io::{Read, Seek};
use std::path::PathBuf;
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::tp_archive_file_param::TpArchiveFileParam;
use crate::traits::{HasTopBarUI, HasUI, IsBXONAsset, ReplicantFile};
use crate::util::ReadUtilExt;

pub struct BXON {
    id: [u8; 4],
    version: u32,
    project_id: u32,
    relative_offset_asset_name: u32,
    offset_asset_name: u64,
    relative_offset_asset_data: u32,
    offset_asset_data: u64,
    asset_name: String,

    asset: Box<dyn IsBXONAsset>
}

impl BXON {
    pub fn new<R: Read + Seek>(path: PathBuf, mut reader: R) -> Result<Self, std::io::Error> {
        let mut id: [u8; 4] = [0; 4];
        reader.read(&mut id)?;
        let version = reader.read_u32::<byteorder::LittleEndian>()?;
        let project_id = reader.read_u32::<byteorder::LittleEndian>()?;

        let (offset_asset_name, relative_offset_asset_name) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let (offset_asset_data, relative_offset_asset_data) = reader.read_offsets::<byteorder::LittleEndian>()?;

        // Read asset name until first null byte
        reader.seek(std::io::SeekFrom::Start(offset_asset_name))?;
        let asset_name = reader.read_string()?;

        reader.seek(std::io::SeekFrom::Start(offset_asset_data))?;
        let asset = match asset_name.as_str() {
            "tpArchiveFileParam" => {
                let asset = TpArchiveFileParam::new(path, reader)?;
                Box::new(asset)
            },
            _ => {
                panic!("Unknown asset type: {}", asset_name);
            }
        };

        Ok(Self {
            id,
            version,
            project_id,
            relative_offset_asset_name,
            offset_asset_name,
            relative_offset_asset_data,
            offset_asset_data,
            asset_name,
            asset
        })
    }
}

impl HasUI for BXON {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        egui::Window::new(format!("{} BXON", egui_phosphor::regular::CUBE))
        .constrain_to(ui.available_rect_before_wrap())
        .resizable([true, true])
        .collapsible(true)
        .movable(true)
        .show(ui.ctx(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.label(format!("Version: {}", self.version));
                ui.label(format!("Project Id: {}", self.project_id));
                ui.separator();
                self.asset.paint(ui, toasts);
            });
        });
    }
}

impl HasTopBarUI for BXON {
    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.asset.paint_top_bar(ui, toasts);
    }
}

impl ReplicantFile for BXON {}