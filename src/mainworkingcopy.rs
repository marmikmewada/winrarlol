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
    is_compression: bool, // Track mode (compression/decompression)
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            source_path: String::new(),
            target_path: String::new(),
            status: String::new(),
            progress: 0.0,
            is_compression: true, // Default to compression
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
            fill: egui::Color32::from_rgba_unmultiplied(10, 10, 20, 200),
            corner_radius: 15.0,
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 200, 255, 255)),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Zip Easy");
                ui.horizontal(|ui| {
                    ui.label("Source Path:");
                    if ui.button("Browse").clicked() {
                        if self.is_compression {
                            // When compressing, allow selecting any folder
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.source_path = path.display().to_string();
                            }
                        } else {
                            // When decompressing, allow selecting only zip files
                            if let Some(path) = FileDialog::new().add_filter("Zip Files", &["zip"]).pick_file() {
                                self.source_path = path.display().to_string();
                            }
                        }
                    }
                });
                ui.text_edit_singleline(&mut self.source_path);

                ui.horizontal(|ui| {
                    ui.label("Target Path:");
                    if ui.button("Browse").clicked() {
                        if let Some(path) = FileDialog::new().save_file() {
                            self.target_path = path.display().to_string();
                        }
                    }
                });
                ui.text_edit_singleline(&mut self.target_path);

                ui.horizontal(|ui| {
                    if ui.radio(self.is_compression, "Compress").clicked() {
                        self.is_compression = true;
                    }
                    if ui.radio(!self.is_compression, "Decompress").clicked() {
                        self.is_compression = false;
                    }
                });

                if ui.button(if self.is_compression { "Compress" } else { "Decompress" }).clicked() {
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

                ui.label(&self.status);
            });
        });
    }

    fn name(&self) -> &str {
        "Zip Easy"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = MyApp::default();
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}
