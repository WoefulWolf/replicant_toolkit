use std::{io::{Read, Seek}, path::PathBuf};

use byteorder::ReadBytesExt;

use crate::{bxon::BXON, traits::{HasSystemPath, HasUI, ReplicantFile, SystemFile}};

pub struct ZstdFile {
    path: PathBuf,
    decompressed_file: Box<dyn ReplicantFile>
}

impl ZstdFile {
    pub fn new<R: Read + Seek>(path: PathBuf, reader: R) -> Result<Self, std::io::Error> {
        let mut decoder = zstd::stream::Decoder::new(reader)?;
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let mut reader = std::io::Cursor::new(decompressed_data.clone());

        let mut decompressed_file_magic = &decompressed_data[0..4];
        let decompressed_file = match decompressed_file_magic {
            b"BXON" => {
                Box::new(BXON::new(path.clone(), &mut reader)?)
            },
            _ => {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown file magic {:X}", decompressed_file_magic.read_u32::<byteorder::LittleEndian>().unwrap())))?
            }
        };


        Ok(Self {
            path,
            decompressed_file
        })
    }
}

impl HasUI for ZstdFile {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        self.decompressed_file.paint(ui, toasts);
    }
}

impl HasSystemPath for ZstdFile {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl SystemFile for ZstdFile {}