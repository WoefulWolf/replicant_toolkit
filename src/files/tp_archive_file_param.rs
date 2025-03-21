use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use byteorder::ReadBytesExt;
use eframe::egui;

use crate::traits::*;
use crate::util::ReadUtilExt;

struct TpArchiveFileParam {
    archive_count: u32,
    rel_offset_archives: u32,
    offset_archives: u64,
    file_count: u32,
    rel_offset_files: u32,
    offset_files: u64,
}

impl TpArchiveFileParam {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let archive_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_archives, rel_offset_archives) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let file_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_files, rel_offset_files) = reader.read_offsets::<byteorder::LittleEndian>()?;

        Ok(Self {
            archive_count,
            rel_offset_archives,
            offset_archives,
            file_count,
            rel_offset_files,
            offset_files,
        })
    }
}

pub struct TpArchiveFileParamManager {
    path: PathBuf,
    runtime: tokio::runtime::Handle,

    tp_archive_file_param: TpArchiveFileParam,
    archive_params: Vec<ArchiveParam>,
    file_params: Vec<FileParam>,
    file_params_filter: String,

    archives_cache: Arc<RwLock<HashMap<String, Archive>>>,
    extracted_file_count: Arc<std::sync::RwLock<usize>>,
    failed_extraction_count: Arc<std::sync::RwLock<usize>>
}

impl TpArchiveFileParamManager {
    pub fn new<R: std::io::Read + std::io::Seek>(path: PathBuf, runtime: tokio::runtime::Handle, mut reader: R) -> Result<Self, std::io::Error> {
        let tp_archive_file_param = TpArchiveFileParam::new(&mut reader)?;

        reader.seek(std::io::SeekFrom::Start(tp_archive_file_param.offset_archives))?;
        let mut archive_params = Vec::new();
        for _ in 0..tp_archive_file_param.archive_count {
            archive_params.push(ArchiveParam::new(&mut reader)?);
        }

        reader.seek(std::io::SeekFrom::Start(tp_archive_file_param.offset_files))?;
        let mut file_params = Vec::new();
        for _ in 0..tp_archive_file_param.file_count {
            file_params.push(FileParam::new(&mut reader)?);
        }

        Ok(Self {
            path,
            runtime,

            tp_archive_file_param,
            archive_params,
            file_params: file_params.clone(),
            file_params_filter: String::new(),

            archives_cache: Arc::new(RwLock::new(HashMap::new())),
            extracted_file_count: Arc::new(std::sync::RwLock::new(file_params.len())),
            failed_extraction_count: Arc::new(std::sync::RwLock::new(0))
        })
    }

    fn extract_file(&self, file_param: &FileParam) -> Result<(), std::io::Error> {
        let archive_param = &self.archive_params[file_param.archive_index as usize];
        let archive_name = archive_param.name.clone();
        let mut archives_directory = self.path.clone();
        archives_directory.pop();
        let mut archive_path = archives_directory.join(&archive_name);

        if !archive_path.exists() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Archive", &["arc"]).set_title(format!("Locate {}", &archive_name)).set_file_name(&archive_name).pick_file() {
                archive_path = path;
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Archive \"{}\" not found.", &archive_name)));
            }
        }

        let offset = (file_param.archive_offset as u64) << 4;

        let archive_file = std::fs::File::open(archive_path)?;
        let archive = Archive::new(archive_file, archive_param.is_streamed)?;

        let file = archive.get_file(offset, file_param.compressed_size as usize, file_param.uncompressed_size as usize, file_param.buffer_size as usize, file_param.is_compressed)?;

        let file_name = file_param.name.clone();

        let Some(output_folder) = rfd::FileDialog::new().set_title(&file_name).pick_folder() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."));
        };

        let mut output_path = output_folder.join(&file_name);
        let output_dir = output_path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."))?;

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let mut output_file = std::fs::File::create(&mut output_path)?;
        
        output_file.write_all(&file)?;
        output_file.flush()?;

        Ok(())
    }

    fn extract_all_files(&self) -> Result<(), std::io::Error> {
        let Some(output_folder) = rfd::FileDialog::new().set_title("Extract all files").pick_folder() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."));
        };

        {
            let mut extracted_file_count = self.extracted_file_count.write().unwrap();
            *extracted_file_count = 0;
            let mut failed_extraction_count = self.failed_extraction_count.write().unwrap();
            *failed_extraction_count = 0;
        }

        let mut sorted_file_params = self.file_params.clone();
        sorted_file_params.sort_by(|a, b| a.archive_index.cmp(&b.archive_index));

        for file_param in sorted_file_params.iter().cloned() {
            let output_folder = output_folder.clone();
            let mut archives_directory = self.path.clone();
            archives_directory.pop();
            let archive_param = self.archive_params[file_param.archive_index as usize].clone();
            let archives_cache = self.archives_cache.clone();
            let extracted_file_count = self.extracted_file_count.clone();
            let failed_extraction_count = self.failed_extraction_count.clone();

            self.runtime.spawn(async move {            
                match TpArchiveFileParamManager::extract_file_async(archives_directory, archive_param, archives_cache, output_folder, file_param.clone()).await {
                    Ok(_) => {
                        println!("Extracted {}.", file_param.name);
                        let mut extracted_file_count = extracted_file_count.write().unwrap();
                        *extracted_file_count += 1;
                    },
                    Err(e) => {
                        println!("Failed to extract {}: {}", file_param.name, e);
                        let mut failed_extraction_count = failed_extraction_count.write().unwrap();
                        *failed_extraction_count += 1;
                    }
                }
            });
        }

        Ok(())
    }


    async fn extract_file_async(archives_directory: PathBuf, archive_param: ArchiveParam, archives_cache: Arc<RwLock<HashMap<String, Archive>>>, output_folder: PathBuf, file_param: FileParam) -> Result<(), std::io::Error> {
        let archive_name = archive_param.name.clone();
        let archive_path = archives_directory.join(&archive_name);

        if !archive_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Archive \"{}\" not found.", &archive_name)));
        }

        let offset = (file_param.archive_offset as u64) << 4;
        let total_size = (file_param.uncompressed_size + file_param.buffer_size) as usize;

        {
            let is_streamed = archive_param.is_streamed.clone();
            let cache = archives_cache.clone();
            let mut cache_write = cache.write().await;

            if !cache_write.contains_key(&archive_name) {
                println!("Opening {}...", archive_name);
                let archive_file = std::fs::File::open(archive_path).unwrap();
                let archive = Archive::new(archive_file, is_streamed).unwrap();
                cache_write.insert(archive_name.clone(), archive);
            } else {
            }
        }

        let cache = archives_cache.clone();
        let cache_read = cache.read().await;
        let archive = cache_read.get(&archive_name).unwrap();

        let file_name = file_param.name.clone();
        let file = archive.get_file(offset, file_param.compressed_size as usize, file_param.uncompressed_size as usize, file_param.buffer_size as usize, file_param.is_compressed)?;

        let mut output_path = output_folder.join(&file_name);
        let output_dir = output_path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."))?;

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let mut output_file = std::fs::File::create(&mut output_path)?;
        
        output_file.write_all(&file)?;
        output_file.flush()?;
        Ok(())
    }
}

impl Resource for TpArchiveFileParamManager {}
impl ResourceManager for TpArchiveFileParamManager {}

struct Archive {
    data: Vec<u8>,
}

impl Archive {
    fn new<R: Read + Seek>(mut reader: R, streamed: bool) -> Result<Self, std::io::Error> {
        let data = match streamed {
            true => {
                let mut data = Vec::new();
                reader.read_to_end(&mut data)?;
                data
            },
            false => {
                reader.seek(std::io::SeekFrom::Start(0))?;
                let mut header = [0; 64];
                reader.read_exact(&mut header)?;
                let Ok(Some(decompressed_size)) = zstd::zstd_safe::get_frame_content_size(&header) else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid zstd frame header."));
                };
                reader.seek(std::io::SeekFrom::Start(0))?;

                let mut decoder = zstd::stream::Decoder::new(reader)?;
                let mut decompressed_data = vec![0; decompressed_size as usize];
                decoder.read_exact(&mut decompressed_data)?;
                decompressed_data
            }
        };

        Ok(Self {
            data
        })
    }

    fn get_file(&self, offset: u64, compressed_size: usize, uncompressed_size: usize, buffer_size: usize, compressed: bool) -> Result<Vec<u8>, std::io::Error> {
        match compressed {
            true => {
                let mut reader = std::io::Cursor::new(&self.data);
                let mut buf = vec![0; compressed_size];
                reader.seek(std::io::SeekFrom::Start(offset))?;
                reader.read_exact(&mut buf)?;

                let mut decoder = zstd::stream::Decoder::new(std::io::Cursor::new(buf))?;
                let mut decompressed_data = vec![0; uncompressed_size + buffer_size];
                decoder.read_exact(&mut decompressed_data)?;
                Ok(decompressed_data)
            },
            false => {
                let mut reader = std::io::Cursor::new(&self.data);
                let mut buf = vec![0; uncompressed_size + buffer_size];
                reader.seek(std::io::SeekFrom::Start(offset))?;
                reader.read_exact(&mut buf)?;
                Ok(buf)
            }
        }
    }
}

#[derive(Clone)]
struct ArchiveParam {
    rel_offset_name: u32,
    flags: u32,
    is_streamed: bool,

    name: String,
}

impl ArchiveParam {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        // Align to 4 bytes
        let offset = reader.stream_position()? % 4;
        if offset != 0 {
            reader.seek_relative(4 - offset as i64)?;
        }

        let (offset_name, rel_offset_name) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let flags = reader.read_u32::<byteorder::LittleEndian>()?;
        let is_streamed = reader.read_u8()? != 0;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_name))?;
        let name = reader.read_string()?;
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        Ok(Self {
            rel_offset_name,
            flags,
            is_streamed,

            name
        })
    }
}

#[derive(Clone)]
struct FileParam {
    hash: u32,
    rel_offset_name: u32,
    archive_offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    buffer_size: u32,
    archive_index: u8,
    is_compressed: bool,

    name: String,
}

impl FileParam {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        // Align to 4 bytes
        let offset = reader.stream_position()? % 4;
        if offset != 0 {
            reader.seek_relative(4 - offset as i64)?;
        }

        let hash = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_name, rel_offset_name) = reader.read_offsets::<byteorder::LittleEndian>()?;
        let archive_offset = reader.read_u32::<byteorder::LittleEndian>()?;
        let compressed_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let uncompressed_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let buffer_size = reader.read_u32::<byteorder::LittleEndian>()?;
        let archive_index = reader.read_u8()?;
        let is_compressed = reader.read_u8()? == 1;

        let return_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(offset_name))?;
        let name = reader.read_string()?;
        reader.seek(std::io::SeekFrom::Start(return_pos))?;

        Ok(Self {
            hash,
            rel_offset_name,
            archive_offset,
            compressed_size,
            uncompressed_size,
            buffer_size,
            archive_index,
            is_compressed,

            name
        })
    }
}

impl Manager for TpArchiveFileParamManager {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        egui::Frame::window(&ui.style()).show(ui, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new(format!("{} tpArchiveFileParam", egui_phosphor::regular::DATABASE)).heading())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Archive Count: {}", self.tp_archive_file_param.archive_count));
                    ui.label(format!("File Count: {}", self.tp_archive_file_param.file_count));

                    ui.separator();

                    ui.collapsing(egui::RichText::new(format!("{} Archives", egui_phosphor::regular::ARCHIVE)).heading(), |ui| {
                        egui_extras::TableBuilder::new(ui)
                        .id_salt("archive_params")
                        .striped(true)
                        .resizable(true)
                        .columns(egui_extras::Column::auto(), 3)
                        .header(16.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Name");
                            });
                            header.col(|ui| {
                                ui.heading("Flags");
                            });
                            header.col(|ui| {
                                ui.heading("Streamed");
                            });
                        })
                        .body(|mut body| {
                            for archive_param in self.archive_params.iter() {
                                body.row(16.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!("{}", archive_param.name));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", archive_param.flags));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", archive_param.is_streamed));
                                    });
                                });
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    ui.collapsing(egui::RichText::new(format!("{} Files", egui_phosphor::regular::FILES)).heading(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Filter:");
                            ui.text_edit_singleline(&mut self.file_params_filter);
                        });

                        egui_extras::TableBuilder::new(ui)
                        .id_salt("file_params")
                        .striped(true)
                        .resizable(true)
                        .columns(egui_extras::Column::auto(), 7)
                        .header(16.0, |mut header: egui_extras::TableRow<'_, '_>| {
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Archive").clicked() {
                                    self.file_params.sort_by(|a, b| a.archive_index.cmp(&b.archive_index));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Path").clicked() {
                                    self.file_params.sort_by(|a, b| a.name.cmp(&b.name));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Hash").clicked() {
                                    self.file_params.sort_by(|a, b| a.hash.cmp(&b.hash));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Compressed Size").clicked() {
                                    self.file_params.sort_by(|a, b| a.compressed_size.cmp(&b.compressed_size));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Uncompressed Size").clicked() {
                                    self.file_params.sort_by(|a, b| a.uncompressed_size.cmp(&b.uncompressed_size));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                if ui.heading("Compressed").clicked() {
                                    self.file_params.sort_by(|a, b| a.is_compressed.cmp(&b.is_compressed));
                                }
                            });
                            header.col(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                ui.heading("Extract");
                            });
                        })
                        .body(|body| {
                            let filtered_file_params = &self.file_params.iter().filter(|file_param| self.file_params_filter.is_empty() || file_param.name.contains(&self.file_params_filter)).cloned().collect::<Vec<_>>();
                            body.rows(16.0, filtered_file_params.len(), |mut row| {
                                let file_param = &filtered_file_params[row.index()];
                                let archive_param = &self.archive_params[file_param.archive_index as usize];

                                row.col(|ui| {
                                    ui.add(egui::Label::new(format!("{}", archive_param.name)).wrap_mode(egui::TextWrapMode::Extend));
                                });
                                row.col(|ui| {
                                    ui.add(egui::Label::new(format!("{}", file_param.name)).wrap_mode(egui::TextWrapMode::Extend));
                                });
                                row.col(|ui| {
                                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                                    ui.add(egui::Label::new(format!("{:08X}", file_param.hash)).wrap_mode(egui::TextWrapMode::Extend));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", file_param.compressed_size));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", file_param.uncompressed_size));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", file_param.is_compressed));
                                });
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui.button("Extract").clicked() {
                                            match self.extract_file(&file_param) {
                                                Ok(_) => {
                                                    toasts.success(format!("File extracted successfully.")).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                                                },
                                                Err(e) => {
                                                    toasts.error(format!("Failed to extract file: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                        });
                    });
                });
            });
    }

    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.menu_button(format!("{} Extract", egui_phosphor::regular::FOLDER_OPEN), |ui| {
            if ui.button("All files…").clicked() {
                match self.extract_all_files() {
                    Ok(_) => {},
                    Err(e) => {
                        toasts.error(format!("Failed to extract all files: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                    }
                }
                ui.close_menu();
            }
        });
    }

    fn paint_floating(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        let extracted_file_count = self.extracted_file_count.read().unwrap();
        let failed_extraction_count = self.failed_extraction_count.read().unwrap();
        if *extracted_file_count < self.file_params.len() {
            egui::Window::new(format!("{} Extracted {}/{} files...", egui_phosphor::regular::TRAY_ARROW_UP, extracted_file_count, self.file_params.len()))
            .id(egui::Id::new("archive_extract_progress"))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.add(egui::ProgressBar::new(*extracted_file_count as f32/self.file_params.len() as f32));
                if *failed_extraction_count > 0 {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("Failed to extract {} files.", failed_extraction_count));
                    });
                }
            });
            ui.ctx().request_repaint();
        }
    }
}