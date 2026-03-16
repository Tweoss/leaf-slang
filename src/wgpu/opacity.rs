use eframe::egui_wgpu::RenderState;
use wgpu::{Device, ShaderModule};

use crate::panes::overlay::Overlay;

use super::*;

const LABEL: &str = "opacity module";

pub struct OpacityModule {
    shader: ShaderModule,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    white_bbox_offset: [u32; 2],
    _padding: [u32; 2],
    black_bbox: [f32; 4],
    black_translation: [f32; 2],
    black_rotation: f32,
    _padding1: [u32; 1],
}

impl OpacityModule {
    pub fn new(device: &Device) -> Self {
        Self {
            shader: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(LABEL),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shader_build/opacity.wgsl").into(),
                ),
            }),
        }
    }

    pub fn reload(&mut self, device: &Device) {
        self.shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL),
            source: wgpu::ShaderSource::Wgsl(
                std::fs::read_to_string("shader_build/opacity.wgsl")
                    .unwrap()
                    .into(),
            ),
        });
    }

    pub fn run(
        &self,
        wgpu_render_state: &RenderState,
        target_offset: (u32, u32),
        black_bbox: Rect,
        (target, white, black): (&mut SharedTexture, &SharedTexture, &SharedTexture),
        overlay: &Overlay,
        callback: impl FnOnce() + Send + 'static,
    ) {
        let device = &wgpu_render_state.device;

        let uniforms = Uniforms {
            white_bbox_offset: [target_offset.0, target_offset.1],
            _padding: [0; 2],
            black_bbox: [
                black_bbox.min.x,
                black_bbox.min.y,
                black_bbox.max.x,
                black_bbox.max.y,
            ],
            black_translation: [overlay.dpos.x, overlay.dpos.y],
            black_rotation: overlay.dangle,
            _padding1: [0; 1],
        };
        let uniforms = bytemuck::bytes_of(&uniforms);

        let uniform_buffer = create_buffer_bytes(device, LABEL, uniforms);
        let bind_group = [
            view_to_bind_group(&white.wgpu_view, 0),
            view_to_bind_group(&black.wgpu_view, 1),
            view_to_bind_group(&target.wgpu_view, 2),
            buffer_to_bind_group(&uniform_buffer, 3),
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
