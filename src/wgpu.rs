pub mod opacity;
pub mod render;
pub mod warp;

use eframe::egui_wgpu::{RenderState, wgpu};
use egui::{Color32, Rect, Sense, TextureId, Vec2, pos2};
use image::RgbaImage;
use wgpu::{
    BindGroupEntry, Buffer, CommandEncoderDescriptor, Device, ShaderModule, Texture, TextureAspect,
    TextureUsages, TextureView, TextureViewDescriptor, util::DeviceExt,
};

use crate::images::SharedTexture;

pub struct Custom3d {
    texture_id: TextureId,
}

impl Custom3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;
        let label = "slang test";

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader_build/compute.wgsl").into()),
        });

        let texture = create_texture(device, label, (100, 100));
        let texture_view = texture_to_view(label, &texture);
        let texture_id = texture_view_to_egui_id(wgpu_render_state, &texture_view);
        let floats = [0.0_f32; 4];
        let uniform_buffer = create_buffer_floats(device, label, &floats);

        let bind_group = [
            view_to_bind_group(&texture_view, 0),
            buffer_to_bind_group(&uniform_buffer, 1),
        ];

        run(
            wgpu_render_state,
            &compute_shader,
            "slang test",
            &bind_group,
            (16, 16, 1),
            || {},
        );
        Some(Self { texture_id })
    }
}

impl Custom3d {
    pub fn ui(&self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.label("Shader running:");

                let uv = Rect {
                    min: pos2(0.0, 0.0),
                    max: pos2(1.0, 1.0),
                };
                let (resp, painter) = ui.allocate_painter(Vec2::new(400.0, 400.0), Sense::empty());
                painter.image(self.texture_id, resp.rect, uv, Color32::WHITE);
            });
        });
    }
}

pub fn texture_from_rgba(
    wgpu_render_state: &RenderState,
    label: &'static str,
    image: &RgbaImage,
) -> SharedTexture {
    let device = &wgpu_render_state.device;
    let dimensions = image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some(label),
        view_formats: &[],
    });
    let view = texture_to_view(label, &texture);
    wgpu_render_state.queue.write_texture(
        wgpu::TexelCopyTextureInfoBase {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    let id = wgpu_render_state.renderer.write().register_native_texture(
        device,
        &texture_to_view(label, &texture),
        wgpu::FilterMode::Linear,
    );

    SharedTexture::from_texture_id(texture, view, id)
}

pub fn create_texture(
    device: &Device,
    label: &'static str,
    (out_width, out_height): (u32, u32),
) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: out_width,
            height: out_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::STORAGE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn texture_to_view(label: &'static str, texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        label: Some(label),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::D2),
        usage: Some(
            TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING,
        ),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
    })
}

pub fn view_to_bind_group(view: &TextureView, index: u32) -> BindGroupEntry<'_> {
    BindGroupEntry {
        binding: index,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

pub fn texture_view_to_egui_id(wgpu_render_state: &RenderState, view: &TextureView) -> TextureId {
    wgpu_render_state.renderer.write().register_native_texture(
        &wgpu_render_state.device,
        view,
        wgpu::FilterMode::Nearest,
    )
}

/// # Warning
/// Ensure floats is aligned properly.
pub fn create_buffer_floats(device: &Device, label: &'static str, floats: &[f32]) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(floats),
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::UNIFORM
            | wgpu::BufferUsages::STORAGE,
    })
}

pub fn create_buffer_bytes(device: &Device, label: &'static str, bytes: &[u8]) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::UNIFORM
            | wgpu::BufferUsages::STORAGE,
    })
}

pub fn buffer_to_bind_group(buffer: &Buffer, index: u32) -> BindGroupEntry<'_> {
    BindGroupEntry {
        binding: index,
        resource: buffer.as_entire_binding(),
    }
}

pub fn run(
    wgpu_render_state: &RenderState,
    shader: &ShaderModule,
    label: &'static str,
    bind_group: &[BindGroupEntry<'_>],
    work_groups: (u32, u32, u32),
    callback: impl FnOnce() + Send + 'static,
) {
    let device = &wgpu_render_state.device;

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: shader,
        entry_point: None,
        compilation_options: Default::default(),
        cache: Default::default(),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &compute_pipeline.get_bind_group_layout(0),
        entries: bind_group,
    });

    let mut encoder =
        device.create_command_encoder(&CommandEncoderDescriptor { label: Some(label) });
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&compute_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(work_groups.0, work_groups.1, work_groups.2);
    }

    wgpu_render_state.queue.submit([encoder.finish()]);
    wgpu_render_state.queue.on_submitted_work_done(callback);
}
