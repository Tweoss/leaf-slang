use crate::wgpu::Custom3d;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Pane {
    Shader,
    Controls,
}

pub fn tree_ui(ui: &mut egui::Ui, tree: &mut egui_tiles::Tree<Pane>, custom_3d: &mut Custom3d) {
    let mut behavior = PaneData { custom_3d };
    tree.ui(&mut behavior, ui);
}

struct PaneData<'a> {
    custom_3d: &'a mut Custom3d,
}

impl egui_tiles::Behavior<Pane> for PaneData<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Shader => "Shader".into(),
            Pane::Controls => "Controls".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        let drag_started = ui
            .add(egui::Button::new("Drag me!").sense(egui::Sense::drag()))
            .drag_started();

        match pane {
            Pane::Shader => self.custom_3d.ui(ui),
            Pane::Controls => {
                ui.label("controls");
                // ui.text_edit_singleline(text);
            }
        }

        if drag_started {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}
