use std::{fs, path::PathBuf};

use anyhow::Result;
use boop::BoopImage;
use clap::Parser;
use eframe::egui;
use egui::{Image, TextureHandle, Widget};
use image::{DynamicImage, GenericImageView, RgbImage};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    image: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let native_options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Boop Viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, cli.image)))),
    )
    .expect("failed to run");

    Ok(())
}

struct MyEguiApp {
    texture: TextureHandle,
}

const MAX_TEXTURE_SIZE: u32 = 2048;

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, image: PathBuf) -> Self {
        let image = BoopImage::decode(&fs::read(image).unwrap())
            .unwrap()
            .to_dynamic_image()
            .unwrap();

        let (width, height) = image.dimensions();

        // Calculate scaling factor if image is too large
        let scale = if width > MAX_TEXTURE_SIZE || height > MAX_TEXTURE_SIZE {
            let width_scale = MAX_TEXTURE_SIZE as f32 / width as f32;
            let height_scale = MAX_TEXTURE_SIZE as f32 / height as f32;
            width_scale.min(height_scale)
        } else {
            1.0
        };

        // Scale image if necessary
        let image = if scale < 1.0 {
            let new_width = (width as f32 * scale) as u32;
            let new_height = (height as f32 * scale) as u32;
            image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            image
        };

        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        // Create texture from image data
        let texture = cc.egui_ctx.load_texture(
            "image-texture",
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
            egui::TextureOptions::default(),
        );

        Self { texture }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| Image::new(&self.texture).shrink_to_fit().ui(ui));
    }
}
