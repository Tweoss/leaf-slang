pub mod labeller;

use std::collections::HashMap;

use egui::RichText;
use egui_tiles::{TileId, Tiles};

use crate::{
    App,
    images::{ImageID, ImagePair},
    panes::labeller::{LabelState, LabelTool, Labels},
    wgpu::{Custom3d, warp::WarpModule},
};

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pane {
    #[default]
    Shader,
    Controls,
    Labeller(LabelTool),
}

pub fn tree_ui(ui: &mut egui::Ui, app: &mut App, frame: &mut eframe::Frame) {
    let mut behavior = PaneData {
        custom_3d: &mut app.custom_3d,
        image_pairs: &mut app.image_pairs,
        label_state: &mut app.label_state,
        labels: &mut app.persistent.labels,
        warp: &mut app.warp_module,
        frame,
    };
    app.persistent.tree.ui(&mut behavior, ui);
}

struct PaneData<'a> {
    custom_3d: &'a mut Custom3d,
    image_pairs: &'a mut [ImagePair],
    label_state: &'a mut LabelState,
    labels: &'a mut HashMap<ImageID, Labels>,
    warp: &'a mut WarpModule,
    frame: &'a mut eframe::Frame,
}

impl egui_tiles::Behavior<Pane> for PaneData<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Shader => "Shader".into(),
            Pane::Controls => "Controls".into(),
            Pane::Labeller(tool) => ("Label ".to_owned()
                + match tool {
                    LabelTool::Corner => "Corner",
                    LabelTool::BBox => "BBox",
                })
            .into(),
        }
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
