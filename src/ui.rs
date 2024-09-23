pub mod boss;
pub mod player;

use bevy::prelude::{AssetServer, Res, Window};
use bevy::prelude::{Query, With};
use bevy_egui::egui::load::SizedTexture;
use bevy_egui::egui::{Image, Pos2, Rect, RichText};
use bevy_egui::{egui, EguiContexts};

use crate::floor::FloorResource;
use crate::player::Player;
use crate::stats::{Damage, Health, Level, MovementSpeed, Revenge, XP};
use crate::Progression;

pub fn draw_hud(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    query: Query<
        (
            &Health,
            &XP,
            &Progression,
            &MovementSpeed,
            &Damage,
            &Level,
            &Revenge,
        ),
        With<Player>,
    >,
    floor: Res<FloorResource>,
) {
    //TODO: Check if calling asset_server.load multiple times is bad
    let health_bar_fill = contexts.add_image(asset_server.load("health_bar_fill.png"));
    let health_bar_border = contexts.add_image(asset_server.load("health_bar_border.png"));

    let mut size = [63. * 5., 10. * 5.];

    if let Ok((health, xp, progression, speed, damage, level, revenge)) = query.get_single() {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(contexts.ctx_mut(), |ui| {
                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(SizedTexture::new(health_bar_border, size)),
                );

                let scale = health.current as f32 / health.max as f32;
                size[0] *= scale;

                ui.put(
                    Rect {
                        min: egui::pos2(16., 8.),
                        max: egui::pos2(16. + size[0], 8. + size[1]),
                    },
                    Image::new(SizedTexture::new(health_bar_fill, size)).uv(Rect {
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
                            ui.label(
                                RichText::new(format!(
                                    "XP: {}/{}",
                                    xp.amount,
                                    progression.xp_to_level_up(level.level)
                                ))
                                .size(20.),
                            );

                            ui.label(
                                RichText::new(format!(
                                    "Revenge: {:.2}/{}",
                                    revenge.amount, revenge.total
                                ))
                                .size(20.),
                            );
                            ui.add_space(2.);
                            ui.label(RichText::new(format!("Damage: {}", damage.amount)).size(20.));
                            ui.add_space(2.);
                            ui.label(RichText::new(format!("Speed: {}", speed.speed)).size(20.));
                        });
                    });
                });
            });
    }

    egui::SidePanel::right("right")
        .frame(egui::Frame::none())
        .resizable(false)
        .show_separator_line(false)
        .exact_width(110.)
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(10.);
            ui.heading(RichText::new(format!("Floor: {}", floor.floor)).size(26.));
            ui.label(RichText::new(format!("Enemies: {}", floor.enemies.len())).size(16.));
        });
}

pub fn draw_xp_bar(
    query: Query<(&XP, &Progression, &Level), With<Player>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
) {
    let painter = contexts.ctx_mut().debug_painter();

    let width = windows.single().width();

    if let Ok((xp, progression, level)) = query.get_single() {
        let start_xp = match level.level {
            1 => 0,
            _ => progression.xp_to_level_up(level.level - 1),
        };

        let scale = (xp.amount - start_xp) as f32 / progression.xp_to_level_up(level.level) as f32;

        painter.rect(
            Rect {
                min: Pos2::new(0., 0.),
                max: Pos2::new(width * scale, 8.),
            },
            0.,
            egui::Color32::DARK_BLUE,
            egui::Stroke::NONE,
        );
    }
}

pub fn draw_revenge_bar(
    query: Query<&Revenge, With<Player>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
) {
    let painter = contexts.ctx_mut().debug_painter();

    let width = windows.single().width();

    if let Ok(revenge) = query.get_single() {
        let scale = revenge.amount / revenge.total;

        painter.rect(
            Rect {
                min: Pos2::new(0., 8.),
                max: Pos2::new(width * scale, 16.),
            },
            0.,
            egui::Color32::DARK_RED,
            egui::Stroke::NONE,
        );
    }
}
