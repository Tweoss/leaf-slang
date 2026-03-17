use eframe::Frame;
use eframe::egui_wgpu::RenderState;
use egui::{Color32, DragValue, Image, ImageSource, Pos2, Rect, Vec2, Widget, color_picker};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint};
use serde::{Deserialize, Serialize};

use crate::images::{ImageID, ImagePair, SharedTexture};
use crate::panes::labeller::{LabelState, Labels, plot_bbox};
use crate::wgpu::opacity::OpacityModule;
use crate::wgpu::{create_texture, texture_to_view, texture_view_to_egui_id};
use std::collections::HashMap;
use std::f32;

pub struct OverlayState {
    petal_index: usize,
    pub textures: HashMap<(String, usize), ((u32, u32), SharedTexture)>,
    color: Color32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            petal_index: Default::default(),
            textures: Default::default(),
            color: Color32::WHITE,
        }
    }
}

impl OverlayState {
    pub fn set_dim(&mut self, render_state: &RenderState, target: (String, usize), dim: Rect) {
        if let Some((_, old)) = self.textures.remove(&target) {
            old.destroy(render_state);
        }

        let offset = (dim.min.x.floor() as u32, dim.min.y.floor() as u32);
        let pixel_dim = (
            dim.max.x.floor() as u32 - offset.0,
            dim.max.y.floor() as u32 - offset.1,
        );
        let label = "overlay texture";
        let texture = create_texture(&render_state.device, label, pixel_dim);
        let view = texture_to_view(label, &texture);
        let id = texture_view_to_egui_id(render_state, &view);
        let new_texture = SharedTexture::from_texture_id(texture, view, id);
        self.textures.insert(target, (offset, new_texture));
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Overlay {
    pub dpos: Vec2,
    pub dangle: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn ui(
    ui: &mut egui::Ui,
    opacity_module: &mut OpacityModule,
    image_pairs: &[ImagePair],
    overlay_state: &mut OverlayState,
    label_state: &LabelState,
    labels: &HashMap<ImageID, Labels>,
    overlays: &mut HashMap<(String, usize), Overlay>,
    frame: &mut Frame,
) -> Option<()> {
    ui.heading("Overlay");

    let Some(pair) = image_pairs.get(label_state.pair_index) else {
        ui.label("No image pair");
        return None;
    };
    let directory = &pair.0;
    let images = &pair.1;

    let petal_count = images
        .iter()
        .filter_map(|i| labels.get(&i.id))
        .map(|l| l.bounding_boxes.len())
        .min()
        .unwrap_or(0);

    ui.add(DragValue::new(&mut overlay_state.petal_index).range(0..=petal_count - 1));

    if overlay_state.petal_index >= petal_count {
        ui.label("Petal count lower than index");
        return None;
    }

    if ui.button("Reload shader").clicked() {
        opacity_module.reload(&frame.wgpu_render_state().expect("wgpu state").device);
    }

    let mut iter = images.iter().map(|i| (i, labels.get(&i.id).unwrap()));
    let pair = [iter.next().unwrap(), iter.next().unwrap()].map(|i| {
        let bbox = i.1.bounding_boxes[overlay_state.petal_index];
        Some((i.0.normalized_texture.as_ref()?, bbox.0, bbox.1, i.0))
    });

    let Some(i0) = pair[0] else {
        ui.label("missing normalized images");
        return None;
    };
    let Some(i1) = pair[1] else {
        ui.label("missing normalized images");
        return None;
    };
    // Use white background on bottom.
    let (white, black) = if i0.1 { (i0, i1) } else { (i1, i0) };

    let overlay = overlays
        .entry((directory.clone(), overlay_state.petal_index))
        .or_default();

    let plot = Plot::new("Overlay Plot")
        .legend(Legend::default())
        .data_aspect(1.0)
        .view_aspect(1.0)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            let size = white.0.egui.size;
            plot_bbox(
                plot_ui,
                white.3,
                |p: Pos2| {
                    [
                        p.x as f64 - size.x as f64 / 2.0,
                        p.y as f64 - size.y as f64 / 2.0,
                    ]
                },
                overlay_state.petal_index,
                &white.2,
                Color32::RED,
            );
            plot_ui.image(PlotImage::new(
                directory.clone() + " white",
                white.0.egui.id,
                PlotPoint::new(0.0, 0.0),
                size,
            ));
            plot_ui.image(
                PlotImage::new(
                    directory.clone() + " black",
                    black.0.egui.id,
                    PlotPoint::new(overlay.dpos.x as f64, overlay.dpos.y as f64),
                    size,
                )
                .rotate(overlay.dangle as f64)
                .tint(Color32::from_white_alpha(160)),
            );
            *plot_ui.transform()
        });
    let transform = plot.inner;
    let plot_hovered = plot.response.hovered();

    let mut target = overlay_state
        .textures
        .get_mut(&(white.3.id.directory.clone(), overlay_state.petal_index));
    if plot_hovered {
        let scale = transform.dvalue_dpos();
        let (delta, shift, space) = ui.ctx().input(|i| {
            (
                i.pointer.delta(),
                i.modifiers.shift,
                i.key_down(egui::Key::Space),
            )
        });
        let delta = Vec2::new(delta.x * scale[0] as f32, delta.y * scale[1] as f32);
        if shift {
            let dangle = delta.y / 100.0;
            let relative_pos = black.2.center() - black.0.egui.size / 2.0;
            let dpos = calc_rotate_translation(
                overlay.dangle + relative_pos.y.atan2(relative_pos.x),
                dangle,
                relative_pos.to_vec2().length(),
            );
            overlay.dangle -= dangle;
            overlay.dpos -= dpos;
        }
        if space {
            overlay.dpos += delta;
        }
        if delta != Vec2::ZERO
            && let Some(target) = &mut target
        {
            opacity_module.run(
                frame.wgpu_render_state().expect("wgpu context"),
                target.0,
                black.2,
                (&mut target.1, white.0, black.0),
                overlay,
                || {},
            );
        }

        if ui.ctx().input(|i| i.key_pressed(egui::Key::S)) {
            overlay_state.petal_index = (overlay_state.petal_index + 1) % petal_count;
        }
    }

    ui.heading("Values");
    ui.label(format!("bboxes {:?} {:?}", white.2, black.2));
    ui.label(format!("dangle {} dpos {:?}", overlay.dangle, overlay.dpos));

    ui.label("background color");
    color_picker::color_edit_button_srgba(
        ui,
        &mut overlay_state.color,
        color_picker::Alpha::Opaque,
    );
    if let Some(target) = target {
        ui.allocate_ui(target.1.egui.size, |ui| {
            let pos = ui.next_widget_position();
            checkers::background_checkers(
                ui.painter(),
                Rect::from_min_size(pos, target.1.egui.size),
                overlay_state.color,
            );
            Image::new(ImageSource::Texture(target.1.egui)).ui(ui);
        });
    }

    None
}

// The pivot point at the end of the arm "swings" the center of the image around.
fn calc_rotate_translation(start_angle: f32, dangle: f32, arm_length: f32) -> Vec2 {
    let end_angle = start_angle + dangle;
    arm_length
        * Vec2::new(
            f32::cos(start_angle) - f32::cos(end_angle),
            f32::sin(start_angle) - f32::sin(end_angle),
        )
}

// Stolen from egui.

mod checkers {
    use egui::{Color32, Mesh, Painter, Rect, Shape, Vec2, lerp, pos2};

    pub fn background_checkers(painter: &Painter, rect: Rect, color: Color32) {
        let rect = rect.shrink(0.1); // Small hack to avoid the checkers from peeking through the sides
        if !rect.is_positive() {
            return;
        }

        let dark_color = Color32::from_gray(32);
        let bright_color = color;

        let checker_size = Vec2::new(rect.width() / 4.0, rect.height() / 2.0);
        let n = (rect.width() / checker_size.x).round() as u32;

        let mut mesh = Mesh::default();
        mesh.add_colored_rect(rect, dark_color);

        let mut top = true;
        for i in 0..n {
            let x = lerp(rect.left()..=rect.right(), i as f32 / (n as f32));
            let small_rect = if top {
                Rect::from_min_size(pos2(x, rect.top()), checker_size)
            } else {
                Rect::from_min_size(pos2(x, rect.center().y), checker_size)
            };
            mesh.add_colored_rect(small_rect, bright_color);
            top = !top;
        }
        painter.add(Shape::mesh(mesh));
    }
}
