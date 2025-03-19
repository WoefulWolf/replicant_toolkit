use std::io::{Read, Seek};
use std::path::PathBuf;
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::files::tp_archive_file_param::TpArchiveFileParam;
use crate::traits::{HasResource, HasUI, IsBXONAsset, ReplicantFile, ReplicantResourceFile};
use crate::util::ReadUtilExt;

use super::tp_gx_tex_head::TpGxTexHead;

pub struct BXON {
    id: [u8; 4],
    version: u32,
    project_id: u32,
    relative_offset_asset_type: u32,
    offset_asset_type: u64,
    relative_offset_asset_data: u32,
    offset_asset_data: u64,
    asset_type: String,

    asset: Box<dyn IsBXONAsset>
}

impl BXON {
    pub fn new<R: Read + Seek>(path: PathBuf, mut reader: R, runtime: tokio::runtime::Handle) -> Result<Self, std::io::Error> {
        let mut id: [u8; 4] = [0; 4];
        reader.read(&mut id)?;
        let version = reader.read_u32::<byteorder::LittleEndian>()?;
        let project_id = reader.read_u32::<byteorder::LittleEndian>()?;

        let (offset_asset_type, relative_offset_asset_type) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let (offset_asset_data, relative_offset_asset_data) = reader.read_offsets::<byteorder::LittleEndian>()?;

        // Read asset name until first null byte
        reader.seek(std::io::SeekFrom::Start(offset_asset_type))?;
        let asset_type = reader.read_string()?;

        reader.seek(std::io::SeekFrom::Start(offset_asset_data))?;
        let asset: Box<dyn IsBXONAsset> = match asset_type.as_str() {
            "tpArchiveFileParam" => {
                let asset = TpArchiveFileParam::new(path, reader, runtime)?;
                Box::new(asset)
            },
            "tpGxTexHead" => {
                let asset = TpGxTexHead::new(path, reader)?;
                Box::new(asset)
            },
            _ => {
                let asset = UnknownBXONAsset { path, asset_type: asset_type.clone() };
                Box::new(asset)
            }
        };

        Ok(Self {
            id,
            version,
            project_id,
            relative_offset_asset_type,
            offset_asset_type,
            relative_offset_asset_data,
            offset_asset_data,
            asset_type,
            asset
        })
    }
}

impl HasUI for BXON {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label(format!("Version: {}", self.version));
        ui.label(format!("Project Id: {}", self.project_id));
        ui.separator();
        self.asset.paint(ui, toasts);
        self.asset.resource_preview(ui, toasts);
    }

    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.asset.paint_top_bar(ui, toasts);
    }

    fn title(&self) -> String {
        format!("{} BXON", egui_phosphor::regular::CUBE)
    }

    fn paint_floating(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.asset.paint_floating(ui, toasts);
    }
}

impl HasResource for BXON {
    fn get_resource_size(&self) -> u32 {
        self.asset.get_resource_size()
    }

    fn set_resource(&mut self, resource: Vec<u8>) {
        self.asset.set_resource(resource);
    }
}

impl ReplicantFile for BXON {}
impl ReplicantResourceFile for BXON {}

struct UnknownBXONAsset {
    path: PathBuf,
    asset_type: String,
}

impl HasUI for UnknownBXONAsset {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        egui::Frame::window(&ui.style()).show(ui, |ui| {
            ui.label(format!("Unknown BXON asset type: {}", self.asset_type));
        });
    }
}

impl HasResource for UnknownBXONAsset {}

impl IsBXONAsset for UnknownBXONAsset {}