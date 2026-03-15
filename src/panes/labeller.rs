use std::collections::HashMap;

use egui::{Color32, DragValue, Pos2, Rect, Stroke, Vec2};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint, Points, Polygon};
use serde::{Deserialize, Serialize};

use crate::images::{ImageID, ImagePair};

pub struct LabelState {
    pair_index: usize,
    tool: Tool,
    corner_index: usize,
    bbox_index: usize,
    bbox_dragging: bool,
    bbox_white_background: bool,
    focused: bool,
}

impl Default for LabelState {
    fn default() -> Self {
        Self {
            pair_index: Default::default(),
            tool: Tool::Corner,
            corner_index: 0,
            bbox_index: 0,
            bbox_dragging: false,
            bbox_white_background: true,
            focused: false,
        }
    }
}

#[derive(PartialEq)]
enum Tool {
    Corner,
    BBox,
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
) {
    ui.label("Target Pair");
    for (i, p) in image_pairs.iter().enumerate() {
        ui.radio_value(&mut label_state.pair_index, i, &p.0);
    }
    let Some(pair) = image_pairs.get_mut(label_state.pair_index) else {
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Tool");
        ui.radio_value(&mut label_state.tool, Tool::Corner, "Corner");
        ui.radio_value(&mut label_state.tool, Tool::BBox, "BBox");

        ui.label("Corner Index");
        ui.add(DragValue::new(&mut label_state.corner_index).range(0..=4));

        ui.label("BBox Index");
        const MAX_PETALS: i32 = 100;
        ui.add(DragValue::new(&mut label_state.bbox_index).range(0..=MAX_PETALS));

        ui.checkbox(&mut label_state.bbox_white_background, "White Background")
    });

    let pointer = ui.ctx().input(|i| {
        (
            i.pointer.latest_pos(),
            i.pointer.primary_clicked(),
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
        )
    });
    egui::Frame::new()
        .stroke(Stroke::new(
            2.0,
            if label_state.focused {
                Color32::BLUE
            } else {
                Color32::GRAY
            },
        ))
        .show(ui, |ui| {
            let clicked = ui.ctx().input(|i| i.pointer.primary_clicked());
            let plot_clicked = Plot::new("Labelling Plot")
                .legend(Legend::default())
                .data_aspect(1.0)
                .view_aspect(1.0)
                .allow_drag(false)
                .show(ui, |plot_ui| {
                    for (i, t) in pair.1.iter().enumerate() {
                        handle_image(label_state, labels, pointer, plot_ui, i, t);
                    }
                })
                .response
                .clicked();

            if plot_clicked {
                label_state.focused = true
            } else {
                if clicked {
                    label_state.focused = false;
                }
            }
        });

    // Advance counters.
    if label_state.focused && ui.ctx().input(|i| i.key_pressed(egui::Key::Space)) {
        match label_state.tool {
            Tool::Corner => label_state.corner_index = (label_state.corner_index + 1) % 4,
            Tool::BBox => label_state.bbox_index += 1,
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
) {
    let texture = t.texture.expect("loaded texture");
    let width = 1.0;
    let height = texture.size.y / texture.size.x;
    let center_x = if i == 0 { -0.5 } else { 0.5 };
    plot_ui.image(PlotImage::new(
        t.id.to_string(),
        texture.id,
        PlotPoint::new(center_x, 0.0),
        (width, height),
    ));
    // Draw plot points.
    if let Some(label) = labels.get(&t.id) {
        plot_ui.points(
            Points::new(
                "corners ".to_owned() + &t.id.to_string(),
                label.corners.map(|p| [p.x as f64, p.y as f64]).to_vec(),
            )
            .radius(20.0)
            .color(if i == 0 { Color32::RED } else { Color32::BLUE }.gamma_multiply(0.5))
            .shape(egui_plot::MarkerShape::Cross),
        );
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
                    .map(|p| [p.x as f64, p.y as f64])
                    .to_vec(),
                )
                .stroke(Stroke::new(1.0, color)),
            );
        }
    }

    let is_in_bounds = |plot_pos: PlotPoint| {
        (plot_pos.x - center_x).abs() < width as f64 / 2.0
            && (plot_pos.y).abs() < height as f64 / 2.0
    };

    // Handle mouse input.
    if label_state.focused
        && let Some(mouse_pos) = pos
    {
        handle_mouse(
            label_state,
            labels,
            mouse_pos,
            (clicked, pressed, released),
            plot_ui,
            t,
            is_in_bounds,
        );
    }
}

fn handle_mouse(
    label_state: &mut LabelState,
    labels: &mut HashMap<ImageID, Labels>,
    mouse_pos: Pos2,
    (clicked, pressed, released): (bool, bool, bool),
    plot_ui: &mut egui_plot::PlotUi<'_>,
    t: &crate::images::Image,
    is_in_bounds: impl Fn(PlotPoint) -> bool,
) {
    let plot_pos = plot_ui.plot_from_screen(mouse_pos);
    if !is_in_bounds(plot_pos) {
        return;
    }
    let target = labels.entry(t.id.clone()).or_insert(Labels {
        corners: [Pos2::ZERO; 4],
        bounding_boxes: vec![],
    });
    match label_state.tool {
        Tool::Corner => {
            if clicked {
                target.corners[label_state.corner_index] = plot_pos.to_pos2();
            }
        }
        Tool::BBox => {
            if pressed {
                if label_state.bbox_index >= target.bounding_boxes.len() {
                    target
                        .bounding_boxes
                        .resize(label_state.bbox_index + 1, (false, Rect::ZERO));
                }

                target.bounding_boxes[label_state.bbox_index].0 = label_state.bbox_white_background;
                target.bounding_boxes[label_state.bbox_index].1 =
                    Rect::from_min_size(plot_pos.to_pos2(), Vec2::ZERO);

                label_state.bbox_white_background ^= true;
                label_state.bbox_dragging = true;
            }
            if label_state.bbox_dragging {
                let target = labels.entry(t.id.clone()).or_insert(Labels {
                    corners: [Pos2::ZERO; 4],
                    bounding_boxes: vec![],
                });
                match label_state.tool {
                    Tool::BBox => {
                        target.bounding_boxes[label_state.bbox_index].1.max = plot_pos.to_pos2()
                    }
                    Tool::Corner => {}
                }

                if released {
                    label_state.bbox_dragging = false;
                }
            }
        }
    }
}
