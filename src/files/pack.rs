use std::path::PathBuf;
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::{traits::{HasSystemPath, HasUI, ReplicantFile, ReplicantResourceFile, SystemFile}, util::ReadUtilExt};

use super::{bxon::BXON, UnknownReplicantFile};

pub struct PACK {
    path: PathBuf,

    id: [u8; 4],
    version: u32,
    total_size: u32,
    serialized_size: u32,
    resources_size: u32,

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
    assets: Vec<Asset>,
    files: Vec<File>,

    files_filter: String,
}

impl PACK {
    pub fn new<R: std::io::Read + std::io::Seek>(path: PathBuf, mut reader: R) -> Result<Self, std::io::Error> {
        let mut id: [u8; 4] = [0; 4];
        reader.read(&mut id)?;
        let version = reader.read_u32::<byteorder::LittleEndian>()?;

        let total_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let serialized_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let resources_size = reader.read_u32::<byteorder::LittleEndian>()?;

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

        reader.seek(std::io::SeekFrom::Start(offset_assets))?;
        let mut assets = Vec::new();
        for _ in 0..asset_count {
            assets.push(Asset::new(&mut reader)?);
        }

        reader.seek(std::io::SeekFrom::Start(offset_files))?;
        let mut files = Vec::new();
        for _ in 0..file_count {
            files.push(File::new(&mut reader)?);
        }

        reader.seek(std::io::SeekFrom::Start(serialized_size as u64))?;
        for file in files.iter_mut() {
            let resource_size = file.content.get_resource_size();
            if resource_size > 0 {
                let resource = Resource::new(&mut reader, resource_size as usize, serialized_size as u64)?;
                file.set_resource(resource);
            }
        }

        Ok(Self {
            path,
            
            id,
            version,
            total_size,
            serialized_size,
            resources_size,

            import_count,
            relative_offset_imports,
            offset_imports,
            
            asset_count,
            relative_ofset_assets,
            offset_assets,

            file_count,
            relative_ofset_files,
            offset_files,

            imports,
            assets,
            files,

            files_filter: String::new()
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

struct Asset {
    hash: u32,
    relative_offset_name: u32,
    offset_name: u64,
    size: u32,
    relative_offset_data_start: u32,
    offset_data_start: u64,
    relative_offset_data_end: u32,
    offset_data_end: u64,

    name: String,
    content: Box::<dyn ReplicantFile>
}

impl Asset {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let hash = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_name, relative_offset_name) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let size = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_data_start, relative_offset_data_start) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let (offset_data_end, relative_offset_data_end) = reader.read_offsets::<byteorder::LittleEndian>()?;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_name))?;
        let name = reader.read_string()?;
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_data_start))?;
        let mut content_magic = [0; 4];
        reader.read_exact(&mut content_magic)?;
        reader.seek(std::io::SeekFrom::Start(offset_data_start))?;
        let content: Box<dyn ReplicantFile> = match &content_magic {
            b"BXON" => {
                Box::new(BXON::new(name.clone().into(), &mut reader)?)
            },
            _ => {
                Box::new(UnknownReplicantFile {})
            }
        };
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        Ok(Self {
            hash,
            relative_offset_name,
            offset_name,
            size,
            relative_offset_data_start,
            offset_data_start,
            relative_offset_data_end,
            offset_data_end,
            name,
            content
        })
    }
}

struct File {
    hash: u32,
    relative_offset_name: u32,
    offset_name: u64,
    size: u32,
    relative_offset_data_start: u32,
    offset_data_start: u64,
    unknown: u32,

    name: String,
    content: Box::<dyn ReplicantResourceFile>,
}

impl File {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let hash = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_name, relative_offset_name) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let size = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_data_start, relative_offset_data_start) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let unknown = reader.read_u32::<byteorder::LittleEndian>()?;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_name))?;
        let name = reader.read_string()?;
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_data_start))?;
        let mut content_magic = [0; 4];
        reader.read_exact(&mut content_magic)?;
        reader.seek(std::io::SeekFrom::Start(offset_data_start))?;
        let content: Box<dyn ReplicantResourceFile> = match &content_magic {
            b"BXON" => {
                Box::new(BXON::new(name.clone().into(), &mut reader)?)
            },
            _ => {
                Box::new(UnknownReplicantFile {})
            }
        };
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        Ok(Self {
            hash,
            relative_offset_name,
            offset_name,
            size,
            relative_offset_data_start,
            offset_data_start,
            unknown,
            name,
            content
        })
    }

    fn set_resource(&mut self, resource: Resource) {
        self.content.set_resource(resource.data.clone());
    }
}

struct Resource {
    data: Vec<u8>,
}

impl Resource {
    fn new<R: std::io::Read + std::io::Seek>(mut reader: R, size: usize, offset_resources: u64) -> Result<Self, std::io::Error> {
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;

        let position = reader.stream_position()?;
        if (position - offset_resources) % 32 != 0 {
            let offset = (((position - offset_resources) / 32) + 1) * 32;
            reader.seek(std::io::SeekFrom::Start(offset_resources + offset))?;
        }

        Ok(Self {
            data
        })
    }
}

impl HasUI for PACK {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.label(format!("Version: {}", self.version));
        ui.label(format!("Total Size: {}", self.total_size));
        ui.label(format!("Serialized Size: {}", self.serialized_size));
        ui.label(format!("Resource Size: {}", self.resources_size));

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

        ui.collapsing(egui::RichText::new(format!("{} Assets", egui_phosphor::regular::FOLDER)).heading(), |ui| {
            if self.assets.is_empty() {
                ui.label("No assets found.");
            } else {
                for asset in self.assets.iter_mut() {
                    egui::Frame::window(&ui.style()).show(ui, |ui| {
                        ui.collapsing(egui::RichText::new(format!("{} ({})", asset.name, asset.content.title())).heading(), |ui| {
                            asset.content.paint(ui, toasts);
                        });
                    });
                }
            }
        });

        ui.separator();

        ui.collapsing(egui::RichText::new(format!("{} Files", egui_phosphor::regular::FILES)).heading(), |ui| {
            if self.files.is_empty() {
                ui.label("No files found.");
            } else {
                ui.horizontal(|ui| {
                    ui.label("Filter: ");
                    ui.text_edit_singleline(&mut self.files_filter);
                });
                egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    for file in self.files.iter_mut().filter(|file| self.files_filter.is_empty() || file.name.contains(&self.files_filter)) {
                        egui::Frame::window(&ui.style()).show(ui, |ui| {
                            ui.collapsing(egui::RichText::new(format!("{} ({})", file.name, file.content.title())).heading(), |ui| {
                                file.content.paint(ui, toasts);
                            });
                        });
                    }
                });
            }
        });
    }

    fn title(&self) -> String {
        format!("{} PACK", egui_phosphor::regular::PACKAGE)
    }
}

impl HasSystemPath for PACK {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl SystemFile for PACK {}