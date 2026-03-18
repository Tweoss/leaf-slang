use std::collections::HashMap;

use eframe::egui_wgpu::RenderState;
use egui::{Color32, DragValue, Pos2, Rect, Stroke, Vec2};
use egui_plot::{Legend, Plot, PlotImage, PlotPoint, Points, Polygon};
use serde::{Deserialize, Serialize};

use crate::{
    images::{ImageID, ImagePair, SharedTexture},
    panes::overlay::OverlayState,
    wgpu::{texture_to_view, warp::WarpModule},
};

// 8.5 x 11
const SCALE: u32 = 8;
const WARP_OUTPUT_SIZE: (u32, u32) = (85 * SCALE, 110 * SCALE);

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LabelTool {
    Corner,
    BBox,
}

pub struct LabelState {
    pub pair_index: usize,
    pub corner_index: usize,
    pub bbox_index: usize,
    pub bbox_dragging: bool,
    pub bbox_white_background: bool,
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
    pub corners: [Pos2; 4],
    // White background or not.
    pub bounding_boxes: Vec<(bool, Rect)>,
}

// Just a struct for holding arguments.
struct ImageContext<'a> {
    label_state: &'a mut LabelState,
    labels: &'a mut HashMap<ImageID, Labels>,
    tool: LabelTool,
    image_index: usize,
    image: &'a crate::images::Image,
    frame: &'a mut eframe::Frame,
    overlay_state: &'a mut OverlayState,
}

// Reset
// - all the warped images
// - all the texture cut out dimensions (use on load).
pub fn init(
    warp: &mut WarpModule,
    overlay_state: &mut OverlayState,
    render_state: &RenderState,
    labels: &mut HashMap<ImageID, Labels>,
    image_pairs: &mut [ImagePair],
) {
    for pair in image_pairs {
        for i in &mut pair.1 {
            let texture = &i.texture.as_ref().unwrap().wgpu;
            let view = texture_to_view("warp images", texture);
            if let Some(l) = labels.get(&i.id) {
                // Setup warping parameters.
                let points = l
                    .corners
                    .map(|p| [p.x / texture.width() as f32, p.y / texture.height() as f32]);
                let (id, out, view) = warp.run(render_state, view, points, WARP_OUTPUT_SIZE, || {});

                if let Some(old_texture) = i
                    .normalized_texture
                    .replace(SharedTexture::from_texture_id(out, view, id))
                {
                    old_texture.destroy(render_state);
                }
            }
        }

        for image in &pair.1 {
            let Some(labels) = labels.get(&image.id) else {
                continue;
            };
            for (i, (white_bg, rect)) in labels.bounding_boxes.iter().enumerate() {
                println!("handling labels for {} {i}", image.id);
                // Only if white background
                if *white_bg {
                    overlay_state.set_dim(render_state, (image.id.directory.clone(), i), *rect);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ui(
    ui: &mut egui::Ui,
    image_pairs: &mut [ImagePair],
    label_state: &mut LabelState,
    labels: &mut HashMap<ImageID, Labels>,
    warp: &mut WarpModule,
    frame: &mut eframe::Frame,
    tool: LabelTool,
    overlay_state: &mut OverlayState,
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

    let input = ui.ctx().input(|i| {
        (
            i.pointer.latest_pos(),
            i.pointer.primary_clicked(),
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            i.modifiers.shift,
        )
    });
    let plot_hovered = Plot::new("Labelling Plot")
        .legend(Legend::default())
        .data_aspect(1.0)
        .view_aspect(1.0)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            for (image_index, image) in pair.1.iter().enumerate() {
                let ctx = ImageContext {
                    label_state,
                    labels,
                    tool,
                    image_index,
                    image,
                    overlay_state,
                    frame,
                };
                should_warp |= handle_image(ctx, input, plot_ui);
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
            let texture = &i.texture.as_ref().unwrap().wgpu;
            let view = texture_to_view("warp images", texture);
            if let Some(l) = labels.get(&i.id) {
                // Setup warping parameters.
                let points = l
                    .corners
                    .map(|p| [p.x / texture.width() as f32, p.y / texture.height() as f32]);
                let (id, out, view) =
                    warp.run(wgpu_render_state, view, points, WARP_OUTPUT_SIZE, || {});

                if let Some(old_texture) = i
                    .normalized_texture
                    .replace(SharedTexture::from_texture_id(out, view, id))
                {
                    old_texture.destroy(wgpu_render_state);
                }
            }
        }
    }
}

type InputInfo = (Option<Pos2>, bool, bool, bool, bool);

fn handle_image(
    ctx: ImageContext<'_>,
    (pos, clicked, pressed, released, shift): InputInfo,
    plot_ui: &mut egui_plot::PlotUi<'_>,
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
    let texture = match ctx.tool {
        LabelTool::Corner => ctx.image.texture.as_ref().expect("loaded texture").egui,
        LabelTool::BBox => {
            if let Some(t) = &ctx.image.normalized_texture {
                t.egui
            } else {
                return false;
            }
        }
    };

    let width = 1.0;
    let height = texture.size.y / texture.size.x;
    let center_x = if ctx.image_index == 0 { -0.5 } else { 0.5 };
    plot_ui.image(PlotImage::new(
        ctx.image.id.to_string(),
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
    if let Some(label) = ctx.labels.get(&ctx.image.id) {
        match ctx.tool {
            LabelTool::Corner => plot_ui.points(
                Points::new(
                    "corners ".to_owned() + &ctx.image.id.to_string(),
                    label.corners.map(to_plot).to_vec(),
                )
                .radius(20.0)
                .color(
                    if ctx.image_index == 0 {
                        Color32::RED
                    } else {
                        Color32::BLUE
                    }
                    .gamma_multiply(0.5),
                )
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
                    plot_bbox(plot_ui, ctx.image, to_plot, i, bbox, color);
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
            ctx,
            (mouse_pos, clicked, pressed, released, shift),
            plot_ui,
            (is_in_bounds, to_image),
        );
        if should_warp {
            return true;
        }
    }
    false
}

pub fn plot_bbox(
    plot_ui: &mut egui_plot::PlotUi<'_>,
    t: &crate::images::Image,
    to_plot: impl Fn(Pos2) -> [f64; 2],
    i: usize,
    bbox: &Rect,
    color: Color32,
) {
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

fn handle_mouse(
    ctx: ImageContext<'_>,
    (mouse_pos, clicked, pressed, released, shift): (Pos2, bool, bool, bool, bool),
    plot_ui: &mut egui_plot::PlotUi<'_>,
    (is_in_bounds, to_image): (impl Fn(PlotPoint) -> bool, impl Fn(PlotPoint) -> Pos2),
) -> bool {
    let plot_pos = plot_ui.plot_from_screen(mouse_pos);
    let image_pos = to_image(plot_pos);
    if !is_in_bounds(plot_pos) {
        return false;
    }
    let target = ctx.labels.entry(ctx.image.id.clone()).or_insert(Labels {
        corners: [Pos2::ZERO; 4],
        bounding_boxes: vec![],
    });
    match ctx.tool {
        LabelTool::Corner => {
            if clicked {
                target.corners[ctx.label_state.corner_index] = image_pos;
                return true;
            }
        }
        LabelTool::BBox => {
            if pressed {
                if ctx.label_state.bbox_index >= target.bounding_boxes.len() {
                    target
                        .bounding_boxes
                        .resize(ctx.label_state.bbox_index + 1, (false, Rect::ZERO));
                }

                ctx.label_state.bbox_white_background = shift;
                target.bounding_boxes[ctx.label_state.bbox_index].0 =
                    ctx.label_state.bbox_white_background;
                target.bounding_boxes[ctx.label_state.bbox_index].1 =
                    Rect::from_min_size(image_pos, Vec2::ZERO);
                ctx.label_state.bbox_dragging = true;
            }
            if ctx.label_state.bbox_dragging {
                let target = ctx.labels.entry(ctx.image.id.clone()).or_insert(Labels {
                    corners: [Pos2::ZERO; 4],
                    bounding_boxes: vec![],
                });
                target.bounding_boxes[ctx.label_state.bbox_index].1.max = image_pos;
                if released {
                    ctx.label_state.bbox_dragging = false;
                    // Make sure even if we were dragging the wrong way
                    // that the min and max are set correctly.
                    let rect = &mut target.bounding_boxes[ctx.label_state.bbox_index].1;
                    let min = rect.min;
                    let max = rect.max;
                    rect.extend_with(min);
                    rect.extend_with(max);

                    if ctx.label_state.bbox_white_background {
                        ctx.overlay_state.set_dim(
                            ctx.frame.wgpu_render_state().expect("wgpu context"),
                            (ctx.image.id.directory.clone(), ctx.label_state.bbox_index),
                            *rect,
                        );
                    }
                }
            }
        }
    }
    false
}
