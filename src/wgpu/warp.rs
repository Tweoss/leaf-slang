// Make a thing which warps image. takes a texture, outputs a texture.

use eframe::egui_wgpu::RenderState;
use egui::TextureId;
use nalgebra::{ArrayStorage, U3, U8};
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
                    include_str!("../../shader_build/warp.wgsl").into(),
                ),
            }),
        }
    }

    pub fn reload(&mut self, device: &Device) {
        self.shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL),
            source: wgpu::ShaderSource::Wgsl(
                std::fs::read_to_string("shader_build/warp.wgsl")
                    .unwrap()
                    .into(),
            ),
        });
    }

    pub fn run(
        &self,
        wgpu_render_state: &RenderState,
        input: TextureView,
        quad_points: [[f32; 2]; 4],
        output_size: (u32, u32),
        callback: impl FnOnce() + Send + 'static,
    ) -> (TextureId, Texture) {
        let device = &wgpu_render_state.device;
        let out = create_texture(device, LABEL, output_size);
        let view = texture_to_view(LABEL, &out);
        let id = texture_view_to_egui_id(wgpu_render_state, &view);

        // Rotates so that order is counterclockwise, top left first.
        let p = quad_points;
        let ox = 1.0;
        let oy = 1.0;
        let q = [[0.0, oy], [0.0, 0.0], [ox, 0.0], [ox, oy]];
        let equations = nalgebra::Matrix::<f32, U8, U8, ArrayStorage<f32, 8, 8>>::from_row_iterator(
            (0..4).flat_map(|i| {
                [
                    p[i][0],
                    p[i][1],
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                    -p[i][0] * q[i][0],
                    -p[i][1] * q[i][0],
                    0.0,
                    0.0,
                    0.0,
                    p[i][0],
                    p[i][1],
                    1.0,
                    -p[i][0] * q[i][1],
                    -p[i][1] * q[i][1],
                ]
            }),
        );
        let desired = nalgebra::OVector::<f32, U8>::from_iterator((0..4).flat_map(|i| q[i]));
        let Some(inv) = equations.try_inverse() else {
            return (id, out);
        };
        let coefficients = inv * desired;
        // Calculate coords in source image from uv in dest image.
        let forward_coeffs = nalgebra::OMatrix::<f32, U3, U3>::from_row_iterator(
            coefficients
                .as_slice()
                .iter()
                .copied()
                .chain(std::iter::once(1.0)),
        )
        .try_inverse()
        .unwrap();

        let uniform_buffer = create_buffer(device, LABEL, forward_coeffs.as_slice());
        let bind_group = [
            view_to_bind_group(&input, 0),
            view_to_bind_group(&view, 1),
            buffer_to_bind_group(&uniform_buffer, 2),
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
