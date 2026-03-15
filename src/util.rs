use std::sync::Arc;

use egui::{ColorImage, Context, TextureOptions, Vec2, load::SizedTexture};
use image::RgbImage;

pub fn image_to_egui_texture(ctx: &Context, name: String, image: &RgbImage) -> SizedTexture {
    let size = [image.width() as usize, image.height() as usize];
    let id = ctx.tex_manager().write().alloc(
        name,
        egui::ImageData::Color(Arc::new(ColorImage::from_rgb(size, image))),
        TextureOptions::default(),
    );
    SizedTexture {
        id,
        size: Vec2::new(size[0] as f32, size[1] as f32),
    }
}
