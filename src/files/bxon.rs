use std::io::{Read, Seek};
use std::path::PathBuf;
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::files::tp_archive_file_param::TpArchiveFileParamManager;
use crate::traits::*;
use crate::util::ReadUtilExt;

use super::tp_gx_tex_head::TpGxTexHeadManager;

struct Bxon {
    id: [u8; 4],
    version: u32,
    project_id: u32,
    relative_offset_asset_type: u32,
    offset_asset_type: u64,
    relative_offset_asset_data: u32,
    offset_asset_data: u64,
    asset_type: String,
}

impl Bxon {
    pub fn new<R: Read + Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let mut id: [u8; 4] = [0; 4];
        reader.read(&mut id)?;
        let version = reader.read_u32::<byteorder::LittleEndian>()?;
        let project_id = reader.read_u32::<byteorder::LittleEndian>()?;

        let (offset_asset_type, relative_offset_asset_type) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let (offset_asset_data, relative_offset_asset_data) = reader.read_offsets::<byteorder::LittleEndian>()?;

        // Read asset name until first null byte
        reader.seek(std::io::SeekFrom::Start(offset_asset_type))?;
        let asset_type = reader.read_string()?;

        Ok(Self {
            id,
            version,
            project_id,
            relative_offset_asset_type,
            offset_asset_type,
            relative_offset_asset_data,
            offset_asset_data,
            asset_type,
        })
    }
}

pub struct BxonManager {
    path: PathBuf,
    runtime: tokio::runtime::Handle,

    bxon: Bxon,
    contents: Box<dyn ResourceManager>
}

impl BxonManager {
    pub fn new<R: Read + Seek>(path: PathBuf, runtime: tokio::runtime::Handle, mut reader: R) -> Result<Self, std::io::Error> {
        let bxon = Bxon::new(&mut reader)?;

        reader.seek(std::io::SeekFrom::Start(bxon.offset_asset_data))?;
        let contents: Box<dyn ResourceManager> = match bxon.asset_type.as_str() {
            "tpArchiveFileParam" => {
                let asset = TpArchiveFileParamManager::new(path.clone(), runtime.clone(), reader)?;
                Box::new(asset)
            },
            "tpGxTexHead" => {
                let asset = TpGxTexHeadManager::new(path.clone(), runtime.clone(), reader)?;
                Box::new(asset)
            },
            _ => {
                let asset = UnknownBxonAssetManager::new(path.clone(), runtime.clone(), bxon.asset_type.clone())?;
                Box::new(asset)
            }
        };

        Ok(Self {
            path,
            runtime,

            bxon,
            contents
        })
    }
}

impl Manager for BxonManager {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label(format!("Version: {}", self.bxon.version));
        ui.label(format!("Project Id: {}", self.bxon.project_id));
        ui.separator();
        self.contents.paint(ui, toasts);
        self.contents.resource_preview(ui, toasts);
    }

    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.contents.paint_top_bar(ui, toasts);
    }

    fn title(&self) -> String {
        format!("{} BXON", egui_phosphor::regular::CUBE)
    }

    fn paint_floating(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.contents.paint_floating(ui, toasts);
    }
}

impl Resource for BxonManager {
    fn get_resource_size(&self) -> u32 {
        self.contents.get_resource_size()
    }

    fn set_resource(&mut self, resource: Vec<u8>) {
        self.contents.set_resource(resource);
    }
}

impl ResourceManager for BxonManager {}

struct UnknownBxonAssetManager {
    path: PathBuf,
    runtime: tokio::runtime::Handle,

    asset_type: String,
}

impl UnknownBxonAssetManager {
    pub fn new(path: PathBuf, runtime: tokio::runtime::Handle, asset_type: String) -> Result<Self, std::io::Error> {
        Ok(Self {
            path,
            runtime,

            asset_type
        })
    }
}

impl Manager for UnknownBxonAssetManager {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        egui::Frame::window(&ui.style()).show(ui, |ui| {
            ui.label(format!("Unknown BXON asset type: {}", self.asset_type));
        });
    }
}

impl Resource for UnknownBxonAssetManager {}

impl ResourceManager for UnknownBxonAssetManager {}