use std::collections::HashMap;

use egui::ComboBox;

use crate::{
    images::{ImageID, ImagePair},
    panes::{
        Pane,
        labeller::{LabelState, Labels},
        tree_ui,
    },
    util::image_to_egui_texture,
    wgpu::Custom3d,
};

pub struct App {
    // Example stuff:
    pub custom_3d: Custom3d,
    pub persistent: Persistent,
    pub image_pairs: Vec<ImagePair>,
    pub new_pane_type: Pane,
    pub label_state: LabelState,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Persistent {
    pub tree: egui_tiles::Tree<Pane>,
    pub labels: HashMap<ImageID, Labels>,
}

impl Default for Persistent {
    fn default() -> Self {
        Self {
            tree: egui_tiles::Tree::new_vertical(
                "pane-container",
                vec![Pane::Shader, Pane::Controls, Pane::Labeller],
            ),
            labels: HashMap::new(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, mut image_pairs: Vec<ImagePair>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        let config = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Persistent::default()
        };
        for pair in &mut image_pairs {
            for i in &mut pair.1 {
                i.texture = Some(image_to_egui_texture(
                    &cc.egui_ctx,
                    i.id.to_string(),
                    &i.original_data,
                ));
            }
        }
        Self {
            custom_3d: Custom3d::new(cc).expect("could not construct shader view"),
            persistent: config,
            image_pairs,
            new_pane_type: Pane::default(),
            label_state: LabelState::default(),
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.persistent);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web && ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.new_pane_type))
                    .show_ui(ui, |ui| {
                        let pane_types = [Pane::Shader, Pane::Controls, Pane::Labeller];
                        for pane in pane_types {
                            let text = match pane {
                                Pane::Shader => "Shader",
                                Pane::Controls => "Controls",
                                Pane::Labeller => "Labeller",
                            };
                            ui.selectable_value(&mut self.new_pane_type, pane, text);
                        }
                    });
                if ui.button("Add Pane").clicked() {
                    let tile = self.persistent.tree.tiles.insert_pane(self.new_pane_type);
                    match self.persistent.tree.root {
                        None => self.persistent.tree.root = Some(tile),
                        Some(root) => self
                            .persistent
                            .tree
                            .move_tile_to_container(tile, root, 0, false),
                    }
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            tree_ui(ui, self);
        });
    }
}
