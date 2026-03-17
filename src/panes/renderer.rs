use eframe::{Frame, egui_wgpu::RenderState};
use egui::{DragValue, ImageSource, Vec2, Widget};

use crate::{
    images::SharedTexture,
    panes::overlay::OverlayState,
    wgpu::{
        create_texture,
        render::{Petal, RenderModule, Uniforms},
        texture_to_view, texture_view_to_egui_id,
    },
};

pub struct RendererState {
    big_texture: Option<SharedTexture>,
    cell_size: (u32, u32),
    output_texture: Option<SharedTexture>,
    textures: Vec<((u32, u32), Vec2)>,
    slider_value: f32,
}

impl RendererState {
    pub fn new(_: &RenderState) -> Self {
        Self {
            big_texture: None,
            cell_size: (1, 1),
            output_texture: None,
            textures: vec![],
            slider_value: -1.0,
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
        let mut big_texture = SharedTexture::from_texture_id(wgpu, view, id);

        for (r, row) in textures.chunks(per_row as usize).enumerate() {
            for (c, t) in row.iter().enumerate() {
                let offset = (c as u32 * cell_size.0, r as u32 * cell_size.1);
                render_module.combine(render_state, t, &mut big_texture, offset, || {});
                renderer_state.textures.push((offset, t.egui.size));
            }
        }

        renderer_state.big_texture = Some(big_texture);
    }

    if ui.button("reload").clicked() {
        render_module.reload(&render_state.device);
    }

    if let Some(big_texture) = &mut renderer_state.big_texture {
        ui.image(ImageSource::Texture(big_texture.egui));

        DragValue::new(&mut renderer_state.slider_value).ui(ui);

        let output = renderer_state.output_texture.get_or_insert_with(|| {
            let device = &render_state.device;
            let label = "renderer";
            let render_size = (300, 300);
            let wgpu = create_texture(device, label, render_size);
            let view = texture_to_view(label, &wgpu);
            let id = texture_view_to_egui_id(render_state, &view);
            SharedTexture::from_texture_id(wgpu, view, id)
        });

        // if rerender {
        let first_size = renderer_state.textures[0].1;
        let petals = [
            Petal::new(
                [0.0, 0.0, renderer_state.slider_value],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0, 0],
                [first_size.x as u32, first_size.y as u32],
            ),
            Petal::new(
                [1.0, 1.0, -5.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0, 0],
                [first_size.x as u32, first_size.y as u32],
            ),
            Petal::new(
                [1.0, 2.0, -5.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [first_size.x as u32, 0],
                [first_size.x as u32, first_size.y as u32],
            ),
        ];
        render_module.run(
            render_state,
            big_texture,
            output,
            &petals,
            Uniforms::new(petals.len() as u32, renderer_state.cell_size.into()),
            || {},
        );
        // }

        ui.image(ImageSource::Texture(output.egui));
    }
}
