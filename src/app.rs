use std::{io::{Read, Seek}, path::PathBuf};

use eframe::egui;

use crate::{files::{self, generic_file}, traits::*};

pub struct ReplicantToolkit {
    runtime: tokio::runtime::Runtime,
    toasts: egui_notify::Toasts,
    show_open_files: bool,
    open_files: Vec<Box<dyn SystemFile>>,
    selected_file_indices: Vec<usize>,
    files_to_close: Vec<usize>,

    top_layer_id: Option<egui::LayerId>,
}

impl Default for ReplicantToolkit {
    fn default() -> Self {
        Self {
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            toasts: egui_notify::Toasts::default(),
            show_open_files: true,
            open_files: Vec::new(),
            selected_file_indices: Vec::new(),
            files_to_close: Vec::new(),
            top_layer_id: None,
        }
    }
}

impl ReplicantToolkit {
    fn open_file(&mut self, path: PathBuf) {
        if self.open_files.iter().any(|file| file.path() == &path) {
            return;
        }

        let mut file_stream = std::fs::File::open(path.clone()).expect("Failed to open file");
        let mut file_magic = [0; 4];
        file_stream.read_exact(&mut file_magic).expect("Failed to read magic");
        file_stream.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek");

        match &file_magic {
            [0x28, 0xB5, 0x2F, 0xFD] => {
                let zstd_file = match files::zstd::ZstdFile::new(path.clone(), file_stream, self.runtime.handle().clone()) {
                    Ok(zstd_file) => zstd_file,
                    Err(e) => {
                        self.toasts.error(format!("Failed to open file: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        return;
                    }
                };
                self.open_files.push(Box::new(zstd_file));
            },
            b"PACK" => {
                let pack_file = match files::pack::PACK::new(path.clone(), file_stream, self.runtime.handle().clone()) {
                    Ok(pack_file) => pack_file,
                    Err(e) => {
                        self.toasts.error(format!("Failed to open file: {}", e)).duration(Some(std::time::Duration::from_secs(10))).closable(true);
                        return;
                    }
                };
                self.open_files.push(Box::new(pack_file));
            },
            _ => {
                let generic_file = Box::new(generic_file::GenericFile::new(path.clone()));
                self.open_files.push(generic_file);
            }
        };
        
        self.open_files.sort_by(|a, b| a.path().cmp(b.path()));
        // Find the index of the newly opened file
        let index = self.open_files.iter().position(|file| file.path() == &path).unwrap();
        self.selected_file_indices = vec![index];
    }

    fn close_file(&mut self, index: usize) {
        self.selected_file_indices.retain(|i| *i != index);

        self.files_to_close.push(index);
    }

    fn open_folder(&mut self, path: PathBuf) {
        // For each file in the folder, add it to the list of open files
        let Ok(dir) = path.read_dir() else {
            return;
        };
        for entry in dir {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if path.is_file() {
                self.open_file(path);
            }
        }
    }

    fn close_all_files(&mut self) {
        self.selected_file_indices.clear();
        self.files_to_close = self.open_files.iter().enumerate().map(|(i, _)| i).collect();
    }

    fn close_queued_files(&mut self) {
        if self.files_to_close.is_empty() {
            return;
        }

        self.files_to_close.sort_by(|a, b| a.cmp(b));
        while let Some(index) = self.files_to_close.pop() {
            self.selected_file_indices.retain(|i| *i != index);
            self.open_files.remove(index);
            // Decrease the indices of all selected files after the removed file
            for i in self.selected_file_indices.iter_mut() {
                if *i > index {
                    *i -= 1;
                }
            }
        }
    }

    fn get_index_of_top_layer_id(&self) -> Option<usize> {
        let mut index = 0;
        if let Some(top_layer_id) = self.top_layer_id {
            for file in self.open_files.iter() {
                if egui::Id::new(file.path()) == top_layer_id.id {
                    return Some(index);
                }
                index += 1;
            }
        }
        None
    }
}

impl eframe::App for ReplicantToolkit {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.close_queued_files();
        self.toasts.show(ctx);
        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("top_bar")
            // .frame(egui::Frame::default().inner_margin(4))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    ui.menu_button(format!("{} File", egui_phosphor::regular::FILE), |ui| {
                        if ui.button("Open file(s)…").clicked() {
                            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                                for path in paths {
                                    self.open_file(path);
                                }
                            }
                            ui.close_menu();
                        }

                        if ui.button("Open folder…").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.open_folder(path);
                            }
                            ui.close_menu();
                        }

                        if ui.button("Close all").clicked() {
                            self.close_all_files();
                            ui.close_menu();
                        }
                    });

                    if let Some(index) = self.get_index_of_top_layer_id() {
                        self.open_files[index].paint_top_bar(ui, &mut self.toasts);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::widgets::global_theme_preference_switch(ui);
                    });
                });
            });

        egui::SidePanel::left("left_bar")
            .resizable(false)
            .default_width(32.0)
            // .frame(egui::Frame::default().inner_margin(4))
            .show(ctx, |ui| {
                ui.toggle_value(&mut self.show_open_files, egui::RichText::new(egui_phosphor::regular::FILES).size(32.0));
            });

        egui::SidePanel::left("open_files")
            .resizable(true)
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(0))
            .show_animated(ctx, self.show_open_files, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        if self.open_files.is_empty() {
                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                ui.label("Open a file to start!");
                            });
                        } else {
                            for i in 0..self.open_files.len() {
                                let selected = self.selected_file_indices.contains(&i);
                                let background_color = if selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    egui::Color32::TRANSPARENT
                                };

                                egui::Frame::default()
                                    .inner_margin(3)
                                    .fill(background_color)
                                    .show(ui, |ui| {
                                        let response = ui.scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), |ui| {
                                            ui.horizontal(|ui| {
                                                ui.horizontal_wrapped(|ui| {
                                                    let filename = self.open_files[i].path().file_name().unwrap_or_default().to_str().unwrap_or("Unknown");
                                                    ui.style_mut().interaction.selectable_labels = false;
                                                    ui.label(filename);
                                                });
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.visuals_mut().button_frame = false;
                                                    if ui.button(egui_phosphor::regular::X).clicked() {
                                                        self.close_file(i);
                                                    }
                                                });
                                            });
                                        }).response;

                                        if response.clicked() {
                                            if ctx.input(|i| i.modifiers.shift) {
                                                if self.selected_file_indices.contains(&i) {
                                                    self.selected_file_indices.retain(|index| *index != i);
                                                } else { 
                                                    self.selected_file_indices.push(i);
                                                }
                                            } else {
                                                self.selected_file_indices = vec![i];
                                            }
                                        }
                                    });
                            }
                        }
                    });
            });

        egui::TopBottomPanel::bottom("bottom_bar")
            .show(ctx, |ui| {
                if let Some(index) = self.get_index_of_top_layer_id() {
                    ui.label(self.open_files[index].path().to_str().unwrap_or("Unknown"));
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(ctx.style().visuals.window_fill.gamma_multiply(0.8)).inner_margin(0))
            .show(ctx, |ui| {
                self.top_layer_id = ui.ctx().top_layer_id();

                if !self.selected_file_indices.is_empty() {
                    for index in self.selected_file_indices.iter() {
                        let filename = self.open_files[*index].path().file_name().unwrap_or_default().to_str().unwrap_or("Unknown");

                        egui::Window::new(format!("{} - {}", self.open_files[*index].title(), filename))
                        .id(egui::Id::new(self.open_files[*index].path()))
                        .constrain_to(ui.available_rect_before_wrap())
                        .resizable([true, true])
                        .collapsible(true)
                        .movable(true)
                        .show(ui.ctx(), |ui| {
                            egui::ScrollArea::both().show(ui, |ui| {
                                self.open_files[*index].paint(ui, &mut self.toasts);
                            });
                        });

                        self.open_files[*index].paint_floating(ui, &mut self.toasts);
                    }
                } else if !self.open_files.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a file to start!");
                    });
                }
            });
    }
}