use eframe::egui_wgpu::RenderState;
use wgpu::{Device, ShaderModule};

use super::*;

const LABEL: &str = "render module";

pub struct RenderModule {
    combine: ShaderModule,
    shader: ShaderModule,
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct Uniforms {
//     white_bbox_offset: [u32; 2],
//     _padding: [u32; 2],
//     black_bbox: [f32; 4],
//     black_translation: [f32; 2],
//     black_rotation: f32,
//     _padding1: [u32; 1],
// }
//

impl RenderModule {
    pub fn new(device: &Device) -> Self {
        Self {
            shader: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(LABEL),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shader_build/render.wgsl").into(),
                ),
            }),
            combine: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(LABEL),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shader_build/combine.wgsl").into(),
                ),
            }),
        }
    }

    pub fn reload(&mut self, device: &Device) {
        self.shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL),
            source: wgpu::ShaderSource::Wgsl(
                std::fs::read_to_string("shader_build/render.wgsl")
                    .unwrap()
                    .into(),
            ),
        });
        self.combine = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL),
            source: wgpu::ShaderSource::Wgsl(
                std::fs::read_to_string("shader_build/combine.wgsl")
                    .unwrap()
                    .into(),
            ),
        });
    }

    pub fn combine(
        &self,
        wgpu_render_state: &RenderState,
        input: &SharedTexture,
        target: &mut SharedTexture,
        offset: (u32, u32),
        callback: impl FnOnce() + Send + 'static,
    ) {
        let device = &wgpu_render_state.device;

        // Pad out.
        let uniform_buffer = create_buffer_bytes(
            device,
            LABEL,
            bytemuck::bytes_of(&[offset.0, offset.1, 0, 0]),
        );
        let bind_group = [
            view_to_bind_group(&input.wgpu_view, 0),
            view_to_bind_group(&target.wgpu_view, 1),
            buffer_to_bind_group(&uniform_buffer, 2),
        ];

        run(
            wgpu_render_state,
            &self.combine,
            LABEL,
            &bind_group,
            (
                input.wgpu.size().width / 16 + 1,
                input.wgpu.size().height / 16 + 1,
                1,
            ),
            callback,
        );
    }

    pub fn run(
        &self,
        wgpu_render_state: &RenderState,
        target: &mut SharedTexture,
        callback: impl FnOnce() + Send + 'static,
    ) {
        // let device = &wgpu_render_state.device;

        // let uniform_buffer = create_buffer_bytes(device, LABEL, uniforms);
        let bind_group = [
            // view_to_bind_group(&white.wgpu_view, 0),
            // view_to_bind_group(&black.wgpu_view, 1),
            view_to_bind_group(&target.wgpu_view, 2),
            // buffer_to_bind_group(&uniform_buffer, 3),
        ];

        run(
            wgpu_render_state,
            &self.shader,
            LABEL,
            &bind_group,
            (
                target.wgpu.size().width / 16 + 1,
                target.wgpu.size().height / 16 + 1,
                1,
            ),
            callback,
        );
    }
}
