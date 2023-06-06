use bevy::prelude::{AssetServer, Res};
use bevy::prelude::{Query, With};
use bevy_egui::egui::{Image, Rect, RichText};
use bevy_egui::{egui, EguiContexts};

use crate::level::LevelResource;
use crate::player::Player;
use crate::stats::{Damage, Health, MovementSpeed, XP};

pub fn draw_hud(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    query: Query<(&Health, &XP, &MovementSpeed, &Damage), With<Player>>,
    level: Res<LevelResource>,
) {
    let health_bar_fill = contexts.add_image(asset_server.load("health_bar_fill.png"));
    let health_bar_border = contexts.add_image(asset_server.load("health_bar_border.png"));

    let mut size = [63. * 5., 10. * 5.];

    if let Ok((health, xp, speed, damage)) = query.get_single() {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(contexts.ctx_mut(), |ui| {
                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(health_bar_border, size),
                );

                let scale = health.current as f32 / health.max as f32;
                size[0] *= scale;

                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(health_bar_fill, size).uv(Rect {
                        min: egui::pos2(0., 0.),
                        max: egui::pos2(scale, 1.),
                    }),
                );

                ui.add_space(10.);

                ui.horizontal(|ui| {
                    ui.add_space(25.);

                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.label(RichText::new(format!("Level: {}", level.level)).size(20.));
                            ui.add_space(2.);
                            ui.label(
                                RichText::new(format!("Health: {}/{}", health.current, health.max))
                                    .size(20.),
                            );
                            ui.add_space(2.);
                            ui.label(RichText::new(format!("XP: {}", xp.amount)).size(20.));
                            ui.add_space(2.);
                            ui.label(RichText::new(format!("Damage: {}", damage.amount)).size(20.));
                            ui.add_space(2.);
                            ui.label(RichText::new(format!("Speed: {}", speed.speed)).size(20.));
                        });
                    });
                });
            });
    }
}
