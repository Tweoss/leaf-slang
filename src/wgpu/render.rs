use eframe::egui_wgpu::RenderState;
use nalgebra::Matrix4;
use wgpu::{Device, ShaderModule};

use super::*;

const LABEL: &str = "render module";

pub struct RenderModule {
    combine: ShaderModule,
    shader: ShaderModule,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Petal {
    pos: [f32; 3],
    _p0: u32,
    xaxis: [f32; 3],
    _p1: u32,
    yaxis: [f32; 3],
    _p2: u32,
    texture_offset: [u32; 2],
    texture_size: [u32; 2],
}
impl Petal {
    pub fn new(
        pos: [f32; 3],
        xaxis: [f32; 3],
        yaxis: [f32; 3],
        texture_offset: [u32; 2],
        texture_size: [u32; 2],
    ) -> Self {
        Self {
            pos,
            _p0: 0,
            xaxis,
            _p1: 0,
            yaxis,
            _p2: 0,
            texture_offset,
            texture_size,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    petal_count: u32,
    _p0: [u32; 1],
    cell_size: [u32; 2],
    camera_inv_view_mat: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new(petal_count: u32, cell_size: [u32; 2], camera_inv_view: Matrix4<f32>) -> Self {
        Self {
            petal_count,
            _p0: [0; 1],
            cell_size,
            // Colmajor for both
            camera_inv_view_mat: camera_inv_view.data.0,
        }
    }
}

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
        atlas: &SharedTexture,
        target: &mut SharedTexture,
        petals: &[Petal],
        uniforms: Uniforms,
        callback: impl FnOnce() + Send + 'static,
    ) {
        let device = &wgpu_render_state.device;

        let uniform_buffer = create_buffer_bytes(device, LABEL, bytemuck::bytes_of(&uniforms));
        let petals: Vec<_> = petals.iter().map(bytemuck::bytes_of).collect();
        let petal_buffer = create_buffer_bytes(device, LABEL, &petals.concat());
        let bind_group = [
            view_to_bind_group(&atlas.wgpu_view, 0),
            view_to_bind_group(&target.wgpu_view, 1),
            buffer_to_bind_group(&petal_buffer, 2),
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
