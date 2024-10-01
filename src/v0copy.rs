use eframe::{egui, epi};
use rfd::FileDialog;
use std::fs::{self, File};
use std::io::{self};
use std::path::Path;
use std::time::Instant;
use zip::{ZipWriter, write::FileOptions, ZipArchive};

struct MyApp {
    source_path: String,
    target_path: String,
    status: String,
    progress: f32,
    is_compression: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            source_path: String::new(),
            target_path: String::new(),
            status: String::new(),
            progress: 0.0,
            is_compression: true,
        }
    }
}

impl MyApp {
    fn compress_file(&mut self) {
        let start = Instant::now();
        let output_path = format!("{}.zip", self.target_path);
        let zip_file = File::create(&output_path).expect("Failed to create zip file");
        let mut zip = ZipWriter::new(zip_file);

        if let Err(e) = zip_folder(&self.source_path, &mut zip) {
            self.status = format!("Error during compression: {}", e);
            return;
        }

        let elapsed = start.elapsed();
        self.status = format!("Compressed successfully in {:?}", elapsed);
        self.progress = 1.0;
    }
    
    fn decompress_file(&mut self) {
        let start = Instant::now();
        let input = File::open(&self.source_path).expect("Failed to open zip file");
        
        let destination = &self.target_path;
        let mut archive = ZipArchive::new(input).expect("Failed to read zip file");

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to get file from archive");
            let outpath = Path::new(destination).join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).expect("Failed to create directory");
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p).expect("Failed to create parent directory");
                }
                let mut output_file = File::create(&outpath).expect("Failed to create output file");
                io::copy(&mut file, &mut output_file).expect("Failed to copy file");
            }
        }

        let elapsed = start.elapsed();
        self.status = format!("Decompressed successfully in {:?}", elapsed);
        self.progress = 1.0;
    }
}

fn zip_folder<P: AsRef<Path>>(source: P, zip: &mut ZipWriter<File>) -> io::Result<()> {
    let source_path = source.as_ref();
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored).unix_permissions(0o755);

    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(source_path).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?; 
            let mut f = File::open(&path)?;
            io::copy(&mut f, zip)?; 
        } else if path.is_dir() {
            zip.add_directory(name.to_string_lossy(), options)?; 
        }
    }

    Ok(())
}

impl epi::App for MyApp {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame) {
        let frame = egui::Frame {
            fill: egui::Color32::from_rgb(240, 248, 255), // AliceBlue
            corner_radius: 10.0,
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 130, 180)), // SteelBlue
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 15.0);
            ui.style_mut().visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 248, 255); // AliceBlue
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
            ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(230, 230, 250); // Lavender
            ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(176, 196, 222); // LightSteelBlue

            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.heading("ZipEasy");
                ui.add_space(30.0);

                let width = ui.available_width().min(400.0);

                ui.group(|ui| {
                    ui.set_width(width);
                    ui.vertical(|ui| {
                        ui.label("Source:");
                        ui.horizontal(|ui| {
                            let text_edit = ui.add_sized([width - 110.0, 30.0], egui::TextEdit::singleline(&mut self.source_path).hint_text("Select source path..."));
                            if ui.add_sized([100.0, 30.0], egui::Button::new("Browse")).clicked() {
                                if self.is_compression {
                                    if let Some(path) = FileDialog::new().pick_folder() {
                                        self.source_path = path.display().to_string();
                                    }
                                } else {
                                    if let Some(path) = FileDialog::new().add_filter("Zip Files", &["zip"]).pick_file() {
                                        self.source_path = path.display().to_string();
                                    }
                                }
                            }
                            text_edit
                        });
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.set_width(width);
                    ui.vertical(|ui| {
                        ui.label("Target:");
                        ui.horizontal(|ui| {
                            let text_edit = ui.add_sized([width - 110.0, 30.0], egui::TextEdit::singleline(&mut self.target_path).hint_text("Select target path..."));
                            if ui.add_sized([100.0, 30.0], egui::Button::new("Save As")).clicked() {
                                if let Some(path) = FileDialog::new().save_file() {
                                    self.target_path = path.display().to_string();
                                }
                            }
                            text_edit
                        });
                    });
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - width) / 2.0);
                    ui.group(|ui| {
                        ui.set_width(width);
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.is_compression, true, "Compress");
                            ui.selectable_value(&mut self.is_compression, false, "Decompress");
                        });
                    });
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - width) / 2.0);
                    if ui.add_sized([width, 40.0], egui::Button::new(
                        if self.is_compression { "Compress" } else { "Decompress" }
                    )).clicked() {
                        if !self.source_path.is_empty() && !self.target_path.is_empty() {
                            if self.is_compression {
                                self.compress_file();
                            } else {
                                self.decompress_file();
                            }
                        } else {
                            self.status = "Please select both source and target paths.".to_string();
                        }
                    }
                });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - width) / 2.0);
                    ui.add_sized([width, 20.0], egui::ProgressBar::new(self.progress).show_percentage());
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - width) / 2.0);
                    ui.add_sized([width, 20.0], egui::Label::new(&self.status));
                });

                ui.add_space(30.0);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label("Created by Marmik Mewada from India <3");
                });
            });
        });
    }

    fn name(&self) -> &str {
        "ZipEasy"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = MyApp::default();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2::new(480.0, 640.0));
    eframe::run_native(Box::new(app), native_options);
}