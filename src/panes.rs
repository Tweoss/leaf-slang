pub mod labeller;
pub mod overlay;
pub mod renderer;

use std::{collections::HashMap, fmt::Display};

use egui::RichText;
use egui_tiles::{TileId, Tiles};

use crate::{
    App,
    images::{ImageID, ImagePair},
    panes::{
        labeller::{LabelState, LabelTool, Labels},
        overlay::{Overlay, OverlayState},
        renderer::RendererState,
    },
    wgpu::{Custom3d, opacity::OpacityModule, render::RenderModule, warp::WarpModule},
};

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pane {
    #[default]
    Shader,
    Controls,
    Labeller(LabelTool),
    Overlay,
    Renderer,
}

impl Display for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Pane::Shader => "Shader".to_owned(),
            Pane::Controls => "Controls".to_owned(),
            Pane::Labeller(tool) => {
                "Label ".to_owned()
                    + match tool {
                        LabelTool::Corner => "Corner",
                        LabelTool::BBox => "BBox",
                    }
            }
            Pane::Overlay => "Overlay".to_owned(),
            Pane::Renderer => "Renderer".to_owned(),
        })
    }
}

impl Pane {
    pub const ENUM: [Pane; 6] = [
        Pane::Shader,
        Pane::Controls,
        Pane::Labeller(LabelTool::Corner),
        Pane::Labeller(LabelTool::BBox),
        Pane::Overlay,
        Pane::Renderer,
    ];
}

pub fn tree_ui(ui: &mut egui::Ui, app: &mut App, frame: &mut eframe::Frame) {
    let mut behavior = PaneData {
        custom_3d: &mut app.custom_3d,
        image_pairs: &mut app.image_pairs,
        label_state: &mut app.label_state,
        overlay_state: &mut app.overlay_state,
        renderer_state: &mut app.renderer_state,
        labels: &mut app.persistent.labels,
        overlays: &mut app.persistent.overlays,
        warp: &mut app.warp_module,
        opacity: &mut app.opacity_module,
        render: &mut app.render_module,
        frame,
    };
    app.persistent.tree.ui(&mut behavior, ui);
}

struct PaneData<'a> {
    custom_3d: &'a mut Custom3d,
    image_pairs: &'a mut [ImagePair],
    overlay_state: &'a mut OverlayState,
    label_state: &'a mut LabelState,
    renderer_state: &'a mut RendererState,
    labels: &'a mut HashMap<ImageID, Labels>,
    overlays: &'a mut HashMap<(String, usize), Overlay>,
    warp: &'a mut WarpModule,
    opacity: &'a mut OpacityModule,
    render: &'a mut RenderModule,
    frame: &'a mut eframe::Frame,
}

impl egui_tiles::Behavior<Pane> for PaneData<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.to_string().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        let drag_started = ui
            .add(
                egui::Label::new(RichText::new("↔").heading())
                    .selectable(false)
                    .sense(egui::Sense::drag()),
            )
            .drag_started();

        match pane {
            Pane::Shader => self.custom_3d.ui(ui),
            Pane::Controls => {
                ui.label("controls");
            }
            Pane::Labeller(tool) => labeller::ui(
                ui,
                self.image_pairs,
                self.label_state,
                self.labels,
                self.warp,
                self.frame,
                *tool,
                self.overlay_state,
            ),
            Pane::Overlay => {
                overlay::ui(
                    ui,
                    self.opacity,
                    self.image_pairs,
                    self.overlay_state,
                    self.label_state,
                    self.labels,
                    self.overlays,
                    self.frame,
                );
            }
            Pane::Renderer => renderer::ui(
                ui,
                self.frame,
                self.renderer_state,
                self.overlay_state,
                self.render,
            ),
        }

        if drag_started {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }
}
