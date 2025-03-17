use std::{io::{Read, Seek}, path::PathBuf};
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::{traits::{HasSystemPath, HasTopBarUI, HasUI, HasWindowTitle, SystemFile}, util::ReadUtilExt};

pub struct PACK {
    path: PathBuf,

    id: [u8; 4],
    version: u32,
    total_size: u32,
    serialized_size: u32,
    resource_size: u32,

    import_count: u32,
    relative_offset_imports: u32,
    offset_imports: u64,
    
    asset_count: u32,
    relative_ofset_assets: u32,
    offset_assets: u64,

    file_count: u32,
    relative_ofset_files: u32,
    offset_files: u64,

    imports: Vec<Import>,
}

impl PACK {
    pub fn new<R: std::io::Read + std::io::Seek>(path: PathBuf, mut reader: R) -> Result<Self, std::io::Error> {
        let mut id: [u8; 4] = [0; 4];
        reader.read(&mut id)?;
        let version = reader.read_u32::<byteorder::LittleEndian>()?;

        let total_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let serialized_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let resource_size = reader.read_u32::<byteorder::LittleEndian>()?;

        let import_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_imports, relative_offset_imports) = reader.read_offsets::<byteorder::LittleEndian>()?;

        let asset_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_assets, relative_ofset_assets) = reader.read_offsets::<byteorder::LittleEndian>()?;

        let file_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_files, relative_ofset_files) = reader.read_offsets::<byteorder::LittleEndian>()?;

        reader.seek(std::io::SeekFrom::Start(offset_imports))?;
        let mut imports = Vec::new();
        for _ in 0..import_count {
            imports.push(Import::new(&mut reader)?);
        }

        Ok(Self {
            path,
            
            id,
            version,
            total_size,
            serialized_size,
            resource_size,

            import_count,
            relative_offset_imports,
            offset_imports,
            
            asset_count,
            relative_ofset_assets,
            offset_assets,

            file_count,
            relative_ofset_files,
            offset_files,

            imports
        })
    }
}

struct Import {
    hash: u32,
    relative_offset: u32,
    offset: u64,
    flags: u32,
    path: String,
}

impl Import {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let hash = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset, relative_offset) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let flags = reader.read_u32::<byteorder::LittleEndian>()?;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let path = reader.read_string()?;
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        Ok(Self {
            hash,
            relative_offset,
            offset,
            flags,
            path
        })
    }
}

impl HasUI for PACK {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label(format!("Version: {}", self.version));
        ui.label(format!("Total Size: {}", self.total_size));
        ui.label(format!("Serialized Size: {}", self.serialized_size));
        ui.label(format!("Resource Size: {}", self.resource_size));

        ui.separator();

        ui.collapsing(egui::RichText::new(format!("{} Imports", egui_phosphor::regular::ARROW_SQUARE_IN)).heading(), |ui| {
            if self.imports.is_empty() {
                ui.label("No imports found.");
            } else {
                egui_extras::TableBuilder::new(ui)
                .id_salt("pack_imports")
                .striped(true)
                .resizable(true)
                .columns(egui_extras::Column::auto(), 2)
                .header(16.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Path");
                    });
                    header.col(|ui| {
                        ui.heading("Hash");
                    });
                })
                .body(|mut body| {
                    for import in self.imports.iter() {
                        body.row(16.0, |mut row| {
                            row.col(|ui| {
                                ui.add(egui::Label::new(format!("{}", import.path)).extend());
                            });
                            row.col(|ui| {
                                ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                                ui.add(egui::Label::new(format!("{:08X}", import.hash)).extend());
                            });
                        });
                    }
                });
            }
        });

        ui.separator();
    }
}

impl HasTopBarUI for PACK {
    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        
    }
}

impl HasSystemPath for PACK {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl HasWindowTitle for PACK {
    fn window_title(&self) -> String {
        format!("{} PACK", egui_phosphor::regular::PACKAGE)
    }
}

impl SystemFile for PACK {}