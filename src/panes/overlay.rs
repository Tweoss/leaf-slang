use egui::{Color32, DragValue, Pos2, Vec2};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint};
use serde::{Deserialize, Serialize};

use crate::images::{ImageID, ImagePair};
use crate::panes::labeller::{LabelState, Labels, plot_bbox};
use std::collections::HashMap;
use std::f32;

#[derive(Default)]
pub struct OverlayState {
    petal_index: usize,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Overlay {
    dpos: Vec2,
    dangle: f32,
}

pub fn ui(
    ui: &mut egui::Ui,
    image_pairs: &[ImagePair],
    overlay_state: &mut OverlayState,
    label_state: &LabelState,
    labels: &HashMap<ImageID, Labels>,
    overlays: &mut HashMap<String, Overlay>,
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

    let overlay = overlays.entry(directory.clone()).or_default();

    let plot = Plot::new("Overlay Plot")
        .legend(Legend::default())
        .data_aspect(1.0)
        .view_aspect(1.0)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            let size = white.0.0.size;
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
                white.0.0.id,
                PlotPoint::new(0.0, 0.0),
                size,
            ));
            plot_ui.image(
                PlotImage::new(
                    directory.clone() + " black",
                    black.0.0.id,
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
            let relative_pos = black.2.center() - black.0.0.size / 2.0;
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

        if ui.ctx().input(|i| i.key_pressed(egui::Key::S)) {
            overlay_state.petal_index = (overlay_state.petal_index + 1) % petal_count;
        }
    }

    ui.heading("Values");
    ui.label(format!("White bbox {:?}", white.2));
    ui.label(format!("Black bbox {:?}", black.2));
    ui.label(format!("dangle {} dpos {:?}", overlay.dangle, overlay.dpos));

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
