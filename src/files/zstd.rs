use std::{io::{Read, Seek}, path::PathBuf};

use byteorder::ReadBytesExt;

use crate::{files::bxon::BxonManager, traits::*};

struct Zstd {}

pub struct ZstdManager {
    path: PathBuf,
    runtime: tokio::runtime::Handle,

    zstd: Zstd,
    contents: Box<dyn Manager>
}

impl ZstdManager {
    pub fn new<R: Read + Seek>(path: PathBuf, runtime: tokio::runtime::Handle, reader: R) -> Result<Self, std::io::Error> {
        let mut decoder = zstd::stream::Decoder::new(reader)?;
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let mut reader = std::io::Cursor::new(decompressed_data.clone());

        let mut decompressed_file_magic = &decompressed_data[0..4];
        let decompressed_file = match decompressed_file_magic {
            b"BXON" => {
                Box::new(BxonManager::new(path.clone(), runtime.clone(), &mut reader)?)
            },
            _ => {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown file magic {:X}", decompressed_file_magic.read_u32::<byteorder::LittleEndian>().unwrap())))?
            }
        };


        Ok(Self {
            path,
            runtime,
            zstd: Zstd {},
            contents: decompressed_file
        })
    }
}

impl Manager for ZstdManager {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.contents.paint(ui, toasts);
    }

    fn paint_top_bar(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.contents.paint_top_bar(ui, toasts);
    }

    fn title(&self) -> String {
        format!("{} ({} zstd)", self.contents.title(), egui_phosphor::regular::FILE_ARCHIVE)
    }

    fn paint_floating(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.contents.paint_floating(ui, toasts);
    }
}