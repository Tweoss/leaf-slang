use std::collections::HashMap;

use egui::{Color32, DragValue, Pos2, Rect, Stroke, Vec2};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint, Points, Polygon};
use serde::{Deserialize, Serialize};

use crate::{
    images::{ImageID, ImagePair},
    wgpu::{texture_to_view, warp::WarpModule},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LabelTool {
    Corner,
    BBox,
}

pub struct LabelState {
    pair_index: usize,
    corner_index: usize,
    bbox_index: usize,
    bbox_dragging: bool,
    bbox_white_background: bool,
}

impl Default for LabelState {
    fn default() -> Self {
        Self {
            pair_index: Default::default(),
            corner_index: 0,
            bbox_index: 0,
            bbox_dragging: false,
            bbox_white_background: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Labels {
    corners: [Pos2; 4],
    bounding_boxes: Vec<(bool, Rect)>,
}

pub fn ui(
    ui: &mut egui::Ui,
    image_pairs: &mut [ImagePair],
    label_state: &mut LabelState,
    labels: &mut HashMap<ImageID, Labels>,
    warp: &mut WarpModule,
    frame: &mut eframe::Frame,
    tool: LabelTool,
) {
    ui.label("Target Pair");
    for (i, p) in image_pairs.iter().enumerate() {
        ui.radio_value(&mut label_state.pair_index, i, &p.0);
    }
    let Some(pair) = image_pairs.get_mut(label_state.pair_index) else {
        return;
    };

    ui.horizontal(|ui| match tool {
        LabelTool::Corner => {
            ui.label("Corner Index");
            ui.add(DragValue::new(&mut label_state.corner_index).range(0..=4));
        }
        LabelTool::BBox => {
            ui.label("BBox Index");
            const MAX_PETALS: i32 = 100;
            ui.add(DragValue::new(&mut label_state.bbox_index).range(0..=MAX_PETALS));
            ui.checkbox(&mut label_state.bbox_white_background, "White Background");
        }
    });
    let mut should_warp = if ui.button("reload shader and warp").clicked() {
        let wgpu_render_state = frame.wgpu_render_state().expect("need a wgpu render state");
        warp.reload(&wgpu_render_state.device);
        true
    } else {
        false
    };

    let pointer = ui.ctx().input(|i| {
        (
            i.pointer.latest_pos(),
            i.pointer.primary_clicked(),
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
        )
    });
    let plot_hovered = Plot::new("Labelling Plot")
        .legend(Legend::default())
        .data_aspect(1.0)
        .view_aspect(1.0)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            for (i, t) in pair.1.iter().enumerate() {
                should_warp |= handle_image(label_state, labels, pointer, plot_ui, i, t, tool);
            }
        })
        .response
        .hovered();

    // Advance counters.
    if plot_hovered && ui.ctx().input(|i| i.key_pressed(egui::Key::Space)) {
        match tool {
            LabelTool::Corner => label_state.corner_index = (label_state.corner_index + 1) % 4,
            LabelTool::BBox => label_state.bbox_index += 1,
        }
    }

    if should_warp {
        let wgpu_render_state = frame.wgpu_render_state().expect("need a wgpu render state");
        for i in &mut pair.1 {
            let texture = &i.texture.as_ref().unwrap().1;
            let view = texture_to_view("warp images", texture);
            if let Some(l) = labels.get(&i.id) {
                // Setup warping parameters.
                let points = l
                    .corners
                    .map(|p| [p.x / texture.width() as f32, p.y / texture.height() as f32]);
                let (id, out) = warp.run(wgpu_render_state, view, points, (200, 200), || {});

                if let Some(old_texture) = i.normalized_texture.replace((
                    egui::load::SizedTexture {
                        id,
                        size: Vec2::new(200.0, 200.0),
                    },
                    out,
                )) {
                    old_texture.1.destroy();
                }
            }
        }
    }
}

type MouseInfo = (Option<Pos2>, bool, bool, bool);

fn handle_image(
    label_state: &mut LabelState,
    labels: &mut HashMap<ImageID, Labels>,
    (pos, clicked, pressed, released): MouseInfo,
    plot_ui: &mut egui_plot::PlotUi<'_>,
    i: usize,
    t: &crate::images::Image,
    tool: LabelTool,
) -> bool {
    let to_plot = |c: [f64; 2]| PlotPoint::new(c[0], c[1]);
    let hovered = pos.is_some_and(|pos| {
        plot_ui
            .transform()
            .rect_from_values(
                &to_plot(plot_ui.plot_bounds().min()),
                &to_plot(plot_ui.plot_bounds().max()),
            )
            .contains(pos)
    });

    // Depending on which mode we are in, show original or warped image.
    let texture = match tool {
        LabelTool::Corner => t.texture.as_ref().expect("loaded texture").0,
        LabelTool::BBox => {
            if let Some(t) = &t.normalized_texture {
                t.0
            } else {
                return false;
            }
        }
    };

    let width = 1.0;
    let height = texture.size.y / texture.size.x;
    let center_x = if i == 0 { -0.5 } else { 0.5 };
    plot_ui.image(PlotImage::new(
        t.id.to_string(),
        texture.id,
        PlotPoint::new(center_x, 0.0),
        (width, height),
    ));

    let to_image = |plot_pos: PlotPoint| {
        Pos2::from([
            ((plot_pos.x - center_x) as f32 / width + 0.5) * texture.size.x,
            (plot_pos.y as f32 / height + 0.5) * texture.size.y,
        ])
    };
    let to_plot = |p: Pos2| {
        [
            (p.x as f64 / texture.size.x as f64 - 0.5) * width as f64 + center_x,
            (p.y as f64 / texture.size.y as f64 - 0.5) * height as f64,
        ]
    };

    // Draw plot points for this mode.
    if let Some(label) = labels.get(&t.id) {
        match tool {
            LabelTool::Corner => plot_ui.points(
                Points::new(
                    "corners ".to_owned() + &t.id.to_string(),
                    label.corners.map(to_plot).to_vec(),
                )
                .radius(20.0)
                .color(if i == 0 { Color32::RED } else { Color32::BLUE }.gamma_multiply(0.5))
                .shape(egui_plot::MarkerShape::Cross),
            ),
            LabelTool::BBox => {
                for (i, (white_background, bbox)) in label.bounding_boxes.iter().enumerate() {
                    let color = if *white_background {
                        Color32::DARK_RED
                    } else {
                        Color32::LIGHT_RED
                    }
                    .gamma_multiply(0.8);
                    plot_ui.polygon(
                        Polygon::new(
                            format!("bbox {i} {}", t.id),
                            [
                                bbox.left_top(),
                                bbox.right_top(),
                                bbox.right_bottom(),
                                bbox.left_bottom(),
                            ]
                            .map(to_plot)
                            .to_vec(),
                        )
                        .stroke(Stroke::new(1.0, color)),
                    );
                }
            }
        };
    }

    let is_in_bounds = |plot_pos: PlotPoint| {
        (plot_pos.x - center_x).abs() < width as f64 / 2.0
            && (plot_pos.y).abs() < height as f64 / 2.0
    };

    // Handle mouse input.
    if hovered && let Some(mouse_pos) = pos {
        let should_warp = handle_mouse(
            label_state,
            labels,
            (mouse_pos, clicked, pressed, released),
            plot_ui,
            t,
            (is_in_bounds, to_image),
            tool,
        );
        if should_warp {
            return true;
        }
    }
    false
}

fn handle_mouse(
    label_state: &mut LabelState,
    labels: &mut HashMap<ImageID, Labels>,
    (mouse_pos, clicked, pressed, released): (Pos2, bool, bool, bool),
    plot_ui: &mut egui_plot::PlotUi<'_>,
    t: &crate::images::Image,
    (is_in_bounds, to_image): (impl Fn(PlotPoint) -> bool, impl Fn(PlotPoint) -> Pos2),
    tool: LabelTool,
) -> bool {
    let plot_pos = plot_ui.plot_from_screen(mouse_pos);
    let image_pos = to_image(plot_pos);
    if !is_in_bounds(plot_pos) {
        return false;
    }
    let target = labels.entry(t.id.clone()).or_insert(Labels {
        corners: [Pos2::ZERO; 4],
        bounding_boxes: vec![],
    });
    match tool {
        LabelTool::Corner => {
            if clicked {
                target.corners[label_state.corner_index] = image_pos;
                return true;
            }
        }
        LabelTool::BBox => {
            if pressed {
                if label_state.bbox_index >= target.bounding_boxes.len() {
                    target
                        .bounding_boxes
                        .resize(label_state.bbox_index + 1, (false, Rect::ZERO));
                }

                target.bounding_boxes[label_state.bbox_index].0 = label_state.bbox_white_background;
                target.bounding_boxes[label_state.bbox_index].1 =
                    Rect::from_min_size(image_pos, Vec2::ZERO);

                label_state.bbox_white_background ^= true;
                label_state.bbox_dragging = true;
            }
            if label_state.bbox_dragging {
                let target = labels.entry(t.id.clone()).or_insert(Labels {
                    corners: [Pos2::ZERO; 4],
                    bounding_boxes: vec![],
                });
                target.bounding_boxes[label_state.bbox_index].1.max = image_pos;
                if released {
                    label_state.bbox_dragging = false;
                }
            }
        }
    }
    false
}
