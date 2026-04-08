use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use eframe::egui::{
    self, Align, Color32, ColorImage, CornerRadius, Frame, Margin, RichText, Stroke, TextureHandle,
    TextureOptions, Vec2,
};
use image::ImageReader;

use crate::handle_event::{Emotion, PetDisplay};

pub struct PetApp {
    updates: Receiver<PetDisplay>,
    current: PetDisplay,
    texture_cache: HashMap<PathBuf, TextureHandle>,
    image_dir: PathBuf,
}

impl PetApp {
    pub fn new(updates: Receiver<PetDisplay>) -> Self {
        PetApp {
            updates,
            current: PetDisplay::idle(),
            texture_cache: HashMap::new(),
            image_dir: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("Emotion_pngs"),
        }
    }

    fn apply_updates(&mut self) {
        while let Ok(update) = self.updates.try_recv() {
            self.current = update;
        }
    }

    fn current_texture(&mut self, ctx: &egui::Context) -> Option<&TextureHandle> {
        let image_path = self.resolve_image_path(self.current.emotion())?;

        if !self.texture_cache.contains_key(&image_path)
            && let Ok(texture) = load_texture(ctx, &image_path)
        {
            self.texture_cache.insert(image_path.clone(), texture);
        }

        self.texture_cache.get(&image_path)
    }

    fn resolve_image_path(&self, emotion: Emotion) -> Option<PathBuf> {
        let mut first_supported = None;
        let mut happy_image = None;
        let target_key = emotion.asset_key();

        let entries = fs::read_dir(&self.image_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_supported_image(&path) {
                continue;
            }

            if first_supported.is_none() {
                first_supported = Some(path.clone());
            }

            let Some(stem) = path.file_stem() else {
                continue;
            };

            let stem = stem.to_string_lossy().to_ascii_uppercase();
            if stem == target_key {
                return Some(path);
            }

            if stem == Emotion::HAPPY.asset_key() {
                happy_image = Some(path.clone());
            }
        }

        happy_image.or(first_supported)
    }
}

impl eframe::App for PetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_updates();
        ctx.request_repaint_after(Duration::from_millis(100));

        let background = Color32::from_rgb(248, 236, 217);
        let panel = Frame::default()
            .fill(background)
            .inner_margin(Margin::same(16))
            .corner_radius(CornerRadius::same(18));

        egui::CentralPanel::default().frame(panel).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("desktop pet")
                            .size(18.0)
                            .strong()
                            .color(Color32::from_rgb(74, 44, 42)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("x").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });

                ui.add_space(8.0);

                if let Some(texture) = self.current_texture(ctx).cloned() {
                    let image_size = fit_image(texture.size_vec2(), Vec2::new(280.0, 220.0));
                    ui.add(egui::Image::new(&texture).fit_to_exact_size(image_size));
                } else {
                    ui.add_space(60.0);
                    ui.label(
                        RichText::new(self.current.emotion_label())
                            .size(40.0)
                            .strong()
                            .color(Color32::from_rgb(120, 76, 65)),
                    );
                    ui.add_space(60.0);
                }

                ui.add_space(12.0);
                ui.label(
                    RichText::new(self.current.emotion_label())
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(120, 76, 65)),
                );
                ui.add_space(8.0);

                Frame::default()
                    .fill(Color32::from_rgb(255, 250, 242))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(214, 190, 165)))
                    .inner_margin(Margin::same(14))
                    .corner_radius(CornerRadius::same(16))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new(self.current.detail())
                                .size(13.0)
                                .color(Color32::from_rgb(128, 103, 90)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(self.current.message())
                                .size(18.0)
                                .color(Color32::from_rgb(49, 35, 34)),
                        );
                    });
            });
        });
    }
}

fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase()),
        Some(extension) if matches!(extension.as_str(), "jpg" | "jpeg" | "png")
    )
}

fn load_texture(ctx: &egui::Context, path: &Path) -> Result<TextureHandle, image::ImageError> {
    let image = ImageReader::open(path)?.decode()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

    Ok(ctx.load_texture(
        path.display().to_string(),
        color_image,
        TextureOptions::LINEAR,
    ))
}

fn fit_image(original: Vec2, bounds: Vec2) -> Vec2 {
    if original.x <= 0.0 || original.y <= 0.0 {
        return bounds;
    }

    let scale = (bounds.x / original.x).min(bounds.y / original.y).min(1.0);
    Vec2::new(original.x * scale, original.y * scale)
}
