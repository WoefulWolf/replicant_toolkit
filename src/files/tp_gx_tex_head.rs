use std::{borrow::Cow, io::Write, path::PathBuf};
use byteorder::{ReadBytesExt, WriteBytesExt};
use eframe::egui;

use crate::traits::*;
use crate::util::ReadUtilExt;

#[repr(u32)]
enum XonSurfaceDXGIFormat {
    UNKNOWN(u32),
	R8g8b8a8UnormStraight= 0x00010700,
	R8g8b8a8Unorm = 0x00010800,
    R8Unorm= 0x00010A00,
	R8g8b8a8UnormSrgb = 0x00010B00,
	Bc1Unorm = 0x00010F00,
	Bc1UnormSrgb = 0x00011000,
	Bc2Unorm = 0x00011100,
	Bc2UnormSrgb = 0x00011200,
	Bc3Unorm = 0x00011300,
	Bc3UnormSrgb = 0x00011400,
	Bc4Unorm = 0x00011500,
	Bc5Unorm = 0x00011600,
	Bc7Unorm = 0x00011900,
    Bc1UnormVolume = 0x00021700,
	Bc7UnormSrgb = 0x00021A00,
	R32g32b32a32Uint = 0x00030000,
	Bc6hUf16 = 0x00031700,
}

impl XonSurfaceDXGIFormat {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x00010700 => XonSurfaceDXGIFormat::R8g8b8a8UnormStraight,
            0x00010800 => XonSurfaceDXGIFormat::R8g8b8a8Unorm,
            0x00010A00 => XonSurfaceDXGIFormat::R8Unorm,
            0x00010B00 => XonSurfaceDXGIFormat::R8g8b8a8UnormSrgb,
            0x00010F00 => XonSurfaceDXGIFormat::Bc1Unorm,
            0x00011000 => XonSurfaceDXGIFormat::Bc1UnormSrgb,
            0x00011100 => XonSurfaceDXGIFormat::Bc2Unorm,
            0x00011200 => XonSurfaceDXGIFormat::Bc2UnormSrgb,
            0x00011300 => XonSurfaceDXGIFormat::Bc3Unorm,
            0x00011400 => XonSurfaceDXGIFormat::Bc3UnormSrgb,
            0x00011500 => XonSurfaceDXGIFormat::Bc4Unorm,
            0x00011600 => XonSurfaceDXGIFormat::Bc5Unorm,
            0x00011900 => XonSurfaceDXGIFormat::Bc7Unorm,
            0x00021700 => XonSurfaceDXGIFormat::Bc1UnormVolume,
            0x00021A00 => XonSurfaceDXGIFormat::Bc7UnormSrgb,
            0x00030000 => XonSurfaceDXGIFormat::R32g32b32a32Uint,
            0x00031700 => XonSurfaceDXGIFormat::Bc6hUf16,
            _ => XonSurfaceDXGIFormat::UNKNOWN(value), // In case the value doesn't match any variant
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            XonSurfaceDXGIFormat::UNKNOWN(value) => *value,
            XonSurfaceDXGIFormat::R8g8b8a8UnormStraight => 0x00010700,
            XonSurfaceDXGIFormat::R8g8b8a8Unorm => 0x00010800,
            XonSurfaceDXGIFormat::R8Unorm => 0x00010A00,
            XonSurfaceDXGIFormat::R8g8b8a8UnormSrgb => 0x00010B00,
            XonSurfaceDXGIFormat::Bc1Unorm => 0x00010F00,   
            XonSurfaceDXGIFormat::Bc1UnormSrgb => 0x00011000,
            XonSurfaceDXGIFormat::Bc2Unorm => 0x00011100,
            XonSurfaceDXGIFormat::Bc2UnormSrgb => 0x00011200,
            XonSurfaceDXGIFormat::Bc3Unorm => 0x00011300,
            XonSurfaceDXGIFormat::Bc3UnormSrgb => 0x00011400,
            XonSurfaceDXGIFormat::Bc4Unorm => 0x00011500,
            XonSurfaceDXGIFormat::Bc5Unorm => 0x00011600,
            XonSurfaceDXGIFormat::Bc7Unorm => 0x00011900,
            XonSurfaceDXGIFormat::Bc1UnormVolume => 0x00021700,
            XonSurfaceDXGIFormat::Bc7UnormSrgb => 0x00021A00,
            XonSurfaceDXGIFormat::R32g32b32a32Uint => 0x00030000,   
            XonSurfaceDXGIFormat::Bc6hUf16 => 0x00031700,
        }
    }

    pub fn to_dxgi_format(&self) -> u32 {
        match self {
            XonSurfaceDXGIFormat::UNKNOWN(value) => *value,
            XonSurfaceDXGIFormat::R8g8b8a8UnormStraight => 28,
            XonSurfaceDXGIFormat::R8g8b8a8Unorm => 28,
            XonSurfaceDXGIFormat::R8Unorm => 61,
            XonSurfaceDXGIFormat::R8g8b8a8UnormSrgb => 29,
            XonSurfaceDXGIFormat::Bc1Unorm => 71,
            XonSurfaceDXGIFormat::Bc1UnormSrgb => 72,
            XonSurfaceDXGIFormat::Bc2Unorm => 74,
            XonSurfaceDXGIFormat::Bc2UnormSrgb => 75,
            XonSurfaceDXGIFormat::Bc3Unorm => 77,
            XonSurfaceDXGIFormat::Bc3UnormSrgb => 78,
            XonSurfaceDXGIFormat::Bc4Unorm => 80,
            XonSurfaceDXGIFormat::Bc5Unorm => 83,
            XonSurfaceDXGIFormat::Bc7Unorm => 98,
            XonSurfaceDXGIFormat::Bc1UnormVolume => 71,
            XonSurfaceDXGIFormat::Bc7UnormSrgb => 99,
            XonSurfaceDXGIFormat::R32g32b32a32Uint => 2,
            XonSurfaceDXGIFormat::Bc6hUf16 => 95,
        }
    }

    pub fn get_alpha_mode(&self) -> u32 {
        match self {
            XonSurfaceDXGIFormat::R8g8b8a8UnormStraight => 1,
            _ => 2
        }
    }
}

pub struct TpGxTexHead {
    path: PathBuf,

    width: u32,
    height: u32,
    depth: u32,
    mip_count: u32,
    size: u32,
    unknown_1: u32,
    format: XonSurfaceDXGIFormat,
    surface_count: u32,
    relative_offset_surfaces: u32,
    offset_surfaces: u64,
    
    surfaces: Vec<Surface>,

    resource: Vec<u8>,
    dds_bytes: Vec<u8>,
    png_images: Vec<Vec<u8>>,
    selected_png_index: usize,
}

impl TpGxTexHead {
    pub fn new<R: std::io::Read + std::io::Seek>(path: PathBuf, mut reader: R) -> Result<Self, std::io::Error> {
        let width = reader.read_u32::<byteorder::LittleEndian>()?;
        let height = reader.read_u32::<byteorder::LittleEndian>()?;
        let depth = reader.read_u32::<byteorder::LittleEndian>()?;
        let mip_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let size = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_1 = reader.read_u32::<byteorder::LittleEndian>()?;
        let format = XonSurfaceDXGIFormat::from_u32(reader.read_u32::<byteorder::LittleEndian>()?);
        let surface_count = reader.read_u32::<byteorder::LittleEndian>()?;
        let (offset_surfaces, relative_offset_surfaces) = reader.read_offsets::<byteorder::LittleEndian>()?;

        reader.seek(std::io::SeekFrom::Start(offset_surfaces))?;
        let mut surfaces = Vec::new();
        for _ in 0..surface_count {
            surfaces.push(Surface::new(&mut reader)?);
        }

        Ok(Self {
            path,
            width,
            height,
            depth,
            mip_count,
            size,
            unknown_1,
            format,
            surface_count: mip_count,
            relative_offset_surfaces,
            offset_surfaces,
            surfaces,
            resource: Vec::new(),
            dds_bytes: Vec::new(),
            png_images: Vec::new(),
            selected_png_index: 0,
        })
    }

    fn populate_dds_bytes(&mut self) -> Result<(), std::io::Error> {
        let mut dds_bytes = Vec::new();
        // Header
        dds_bytes.write(b"DDS\x20")?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(124)?;
        let mut flags = 0x1 | 0x2 | 0x4 | 0x1000 | 0x80000;
        if self.surface_count > 1 {
            flags |= 0x20000;
        }
        if self.depth > 1 {
            flags |= 0x800000;
        }
        dds_bytes.write_u32::<byteorder::LittleEndian>(flags)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.height)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.width)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.size)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.depth)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.surface_count)?;
        for i in 0..11 {
            dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        }

        // DDS Pixel Format
        dds_bytes.write_u32::<byteorder::LittleEndian>(32)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(4)?;
        dds_bytes.write(b"DX10")?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        for i in 0..4{
            dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        }
        let mut caps = 0x1000;
        if self.surface_count > 1 {
            caps |= 0x8 | 0x400000;
        }
        dds_bytes.write_u32::<byteorder::LittleEndian>(caps)?;
        caps = 0x0;
        if self.depth > 1 {
            caps |= 0x200000;
        }
        dds_bytes.write_u32::<byteorder::LittleEndian>(caps)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        
        dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;

        // DDS Header DXT10
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.format.to_dxgi_format())?;
        let mut dimension = 3;
        if self.depth > 1 {
            dimension = 4;
        }
        dds_bytes.write_u32::<byteorder::LittleEndian>(dimension)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(0)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(1)?;
        dds_bytes.write_u32::<byteorder::LittleEndian>(self.format.get_alpha_mode())?;
        dds_bytes.write(&self.resource)?;

        self.dds_bytes = dds_bytes;
        Ok(())
    }

    fn populate_png_bytes(&mut self) -> Result<(), std::io::Error> {
        for mip in 0..self.mip_count {
            let img = image_dds::image_from_dds(
                &image_dds::ddsfile::Dds::read(std::io::Cursor::new(self.dds_bytes.clone())).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
                mip
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let mut png_bytes = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            self.png_images.push(png_bytes);
        }
        Ok(())
    }

    fn export_dds(&self) -> Result<(), std::io::Error> {
        if self.dds_bytes.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "DDS bytes are empty."));
        }

        let Some(output_path) = rfd::FileDialog::new().set_title(format!("Export {} as DDS", self.path.to_str().unwrap_or_default())).set_file_name(format!("{}.dds", self.path.file_name().unwrap_or_default().to_str().unwrap_or_default())).save_file() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Output path not found."));
        };

        let output_dir = output_path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."))?;

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let mut output_file = std::fs::File::create(output_path)?;
        output_file.write_all(&self.dds_bytes)?;
        output_file.flush()?;

        Ok(())
    }

    fn export_png(&self) -> Result<(), std::io::Error> {
        if self.selected_png_index >= self.png_images.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "PNG bytes are empty."));
        }

        let Some(output_path) = rfd::FileDialog::new().set_title(format!("Export {} as PNG", self.path.to_str().unwrap_or_default())).set_file_name(format!("{}.png", self.path.file_name().unwrap_or_default().to_str().unwrap_or_default())).save_file() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Output path not found."));
        };

        let output_dir = output_path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Output folder not found."))?;

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let mut output_file = std::fs::File::create(output_path)?;
        output_file.write_all(&self.png_images[self.selected_png_index])?;
        output_file.flush()?;

        Ok(())
    }
}

struct Surface {
    offset: u32,
    unknown_0: u32,
    unknown_1: u32,
    unknown_2: u32,
    size: u32,
    unknown_3: u32,
    width: u32,
    height: u32,
    unknown_6: u32,
    unknown_7: u32,
}

impl Surface {
    pub fn new<R: std::io::Read + std::io::Seek>(mut reader: R) -> Result<Self, std::io::Error> {
        let offset = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_0 = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_1 = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_2 = reader.read_u32::<byteorder::LittleEndian>()?;
        let size = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_3 = reader.read_u32::<byteorder::LittleEndian>()?;
        let width = reader.read_u32::<byteorder::LittleEndian>()?;
        let height = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_6 = reader.read_u32::<byteorder::LittleEndian>()?;
        let unknown_7 = reader.read_u32::<byteorder::LittleEndian>()?;

        Ok(Self {
            offset,
            unknown_0,
            unknown_1,
            unknown_2,
            size,
            unknown_3,
            width,
            height,
            unknown_6,
            unknown_7
        })
    }
}

impl HasResource for TpGxTexHead {
    fn get_resource_size(&self) -> u32 {
        self.size
    }

    fn set_resource(&mut self, resource: Vec<u8>) {
        self.resource = resource;
        match self.populate_dds_bytes() {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to populate dds bytes: {}", e);
                self.dds_bytes = Vec::new();
            }
        }
        if !self.dds_bytes.is_empty() {
            match self.populate_png_bytes() {
                Ok(_) => {},
                Err(e) => {
                    println!("Failed to populate png bytes: {}", e);
                    self.png_images = Vec::new();
                }
            }
        }
    }
    
    fn resource_preview(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        ui.horizontal(|ui| {
            ui.label("Mip:");
            ui.add(egui::Slider::new(&mut self.selected_png_index, 0..=(self.png_images.len() - 1)).show_value(true));
        });
        if !self.png_images.is_empty() && self.selected_png_index < self.png_images.len() {
            let uri = Cow::from(format!("bytes://{}.{}.png", self.path.to_str().unwrap_or_default(), self.selected_png_index));
            ui.add(egui::Image::new(egui::ImageSource::Bytes { uri, bytes: self.png_images[self.selected_png_index].clone().into() }).texture_options(egui::TextureOptions::NEAREST).show_loading_spinner(true).maintain_aspect_ratio(true).fit_to_exact_size(egui::Vec2::new(512.0, 512.0)));
            // Export buttons
            ui.horizontal(|ui| {
                if ui.button("Export DDS").clicked() {
                    match self.export_dds() {
                        Ok(_) => {
                            toasts.success(format!("DDS exported successfully.")).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        },
                        Err(e) => {
                            toasts.error(format!("Failed to export DDS: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        }
                    }
                }

                if ui.button("Export PNG").clicked() {
                    match self.export_png() {
                        Ok(_) => {
                            toasts.success(format!("PNG exported successfully.")).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        },
                        Err(e) => {
                            toasts.error(format!("Failed to export PNG: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        }
                    }
                }
            });
        }
    }
}

impl HasUI for TpGxTexHead {
    fn paint(&mut self, ui: &mut eframe::egui::Ui, toasts: &mut egui_notify::Toasts) {
        egui::Frame::window(&ui.style()).show(ui, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new(self.title()).heading())
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(format!("Width: {}", self.width));
                    ui.label(format!("Height: {}", self.height));
                    ui.label(format!("Depth: {}", self.depth));
                    ui.label(format!("Size: {}", self.size));
                    ui.label(format!("Format: {:X}", self.format.to_u32()));
                    ui.label(format!("Mip Count: {}", self.mip_count));
                    ui.label(format!("Surface Count: {}", self.surface_count));
                });
            });
    }

    fn title(&self) -> String {
        format!("{} tpGxTexHead", egui_phosphor::regular::IMAGE)
    }
}

impl IsBXONAsset for TpGxTexHead {}