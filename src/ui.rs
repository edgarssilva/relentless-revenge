use bevy::prelude::{Query, ResMut, With};
use bevy::prelude::{AssetServer, Res};
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::{Image, Rect};

use crate::player::Player;
use crate::stats::Health;

pub fn setup_ui(mut egui_context: ResMut<EguiContext>, asset_server: Res<AssetServer>, query: Query<&Health, With<Player>>) {
    let health_bar_fill = egui_context.add_image(asset_server.load("health_bar_fill.png"));
    let health_bar_border = egui_context.add_image(asset_server.load("health_bar_border.png"));

    let mut size = [63. * 5., 10. * 5.];

    if let Ok(health) = query.get_single() {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_context.ctx_mut(), |ui| {
                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(health_bar_border, size),
                );

                size[0] *= health.current as f32 / health.max as f32;

                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(health_bar_fill, size),
                );

            });

    }
}
