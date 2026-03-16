use eframe::{Frame, egui_wgpu::RenderState};
use egui::{ImageSource, Vec2};
use wgpu::Device;

use crate::{
    images::SharedTexture,
    panes::overlay::OverlayState,
    wgpu::{create_texture, render::RenderModule, texture_to_view, texture_view_to_egui_id},
};

pub struct RendererState {
    big_texture: Option<SharedTexture>,
    cell_size: (u32, u32),
    output_texture: Option<SharedTexture>,
}

impl RendererState {
    pub fn new(_: &RenderState) -> Self {
        Self {
            big_texture: None,
            cell_size: (1, 1),
            output_texture: None,
        }
    }
}

pub fn ui(
    ui: &mut egui::Ui,
    frame: &Frame,
    renderer_state: &mut RendererState,
    overlay_state: &OverlayState,
    render_module: &mut RenderModule,
) {
    let render_state = frame.wgpu_render_state().expect("wgpu render state");
    let mut rerender = false;
    if ui.button("Generate Texture").clicked() {
        let mut ts: Vec<_> = overlay_state.textures.iter().collect();
        ts.sort_by_key(|ts| ts.0);
        let textures: Vec<_> = ts.into_iter().map(|(_, (_, t))| t).collect();
        let biggest_size = textures
            .iter()
            .map(|t| t.egui.size)
            .reduce(|a, e| a.max(e))
            .unwrap_or(Vec2::new(20.0, 20.0));
        let cell_size = biggest_size.floor();
        let cell_size = (cell_size.x as u32, cell_size.y as u32);
        renderer_state.cell_size = cell_size;
        let count = textures.len() as u32;
        let per_row = 10;
        let rows = count.div_ceil(per_row);

        if let Some(t) = renderer_state.big_texture.take() {
            t.destroy(render_state);
        }
        let device = &render_state.device;
        let label = "renderer";
        let wgpu = create_texture(device, label, (per_row * cell_size.0, rows * cell_size.1));
        let view = texture_to_view(label, &wgpu);
        let id = texture_view_to_egui_id(render_state, &view);
        renderer_state.big_texture = Some(SharedTexture::from_texture_id(wgpu, view, id));
        rerender = true;
    }

    if let Some(big_texture) = &mut renderer_state.big_texture {
        if rerender {
            render_module.run(render_state, big_texture, || {});
        }

        ui.image(ImageSource::Texture(big_texture.egui));
    }
}
