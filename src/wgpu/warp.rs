// Make a thing which warps image. takes a texture, outputs a texture.

use eframe::egui_wgpu::RenderState;
use egui::TextureId;
use wgpu::{Device, ShaderModule, TextureView};

use super::*;

const LABEL: &str = "warp module";

pub struct WarpModule {
    shader: ShaderModule,
}

impl WarpModule {
    pub fn new(device: &Device) -> Self {
        Self {
            shader: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(LABEL),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shader_build/compute.wgsl").into(),
                ),
            }),
        }
    }

    pub fn run(
        &self,
        wgpu_render_state: &RenderState,
        input: TextureView,
        quad_points: [(f32, f32); 4],
        output_size: (u32, u32),
        callback: impl FnOnce() + Send + 'static,
    ) -> (TextureId, Texture) {
        let device = &wgpu_render_state.device;
        let out = create_texture(device, LABEL, output_size);
        let view = texture_to_view(LABEL, &out);
        let id = texture_view_to_egui_id(wgpu_render_state, &view);
        let floats = [0.0_f32; 4];
        let uniform_buffer = create_buffer(device, LABEL, &floats);
        let bind_group = [
            view_to_bind_group(&view, 0),
            buffer_to_bind_group(&uniform_buffer, 1),
        ];
        run(
            wgpu_render_state,
            &self.shader,
            LABEL,
            &bind_group,
            (16, 16, 1),
            callback,
        );

        (id, out)
    }
}
