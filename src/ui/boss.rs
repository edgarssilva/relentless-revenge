use bevy::{
    color::Color,
    math::Vec2,
    prelude::*,
    time::{Time, Timer},
};
use bevy_egui::{
    egui::{self, Color32, Frame, RichText, Stroke},
    EguiContexts,
};

use crate::{boss::Boss, Health};

#[derive(Component)]
pub struct DomainName(pub String, pub Timer);

pub fn draw_domain_name(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut domain: Query<(&mut DomainName, Entity)>,
    time: Res<Time>,
) -> Result {
    if let Ok((mut domain, entity)) = domain.single_mut() {
        let remaining = domain.1.duration().as_secs_f32() - domain.1.elapsed_secs();
        let progress = if remaining > 0.5 { 0.5 } else { remaining };

        let alpha = (progress * 255.0) as u8;
        let transparent_white = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

        egui::TopBottomPanel::top("domain_name")
            .frame(
                Frame::default()
                    .fill(Color32::from_black_alpha(0))
                    .stroke(Stroke::new(0.0, Color32::TRANSPARENT)),
            )
            .show(contexts.ctx_mut()?, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new(domain.0.clone())
                            .size(40.)
                            .color(transparent_white)
                            .strong(),
                    );
                });
            });

        domain.1.tick(time.delta());

        if domain.1.is_finished() {
            commands.entity(entity).despawn();
        }
    }

    Ok(())
}

pub fn draw_boss_health(query: Query<(&Boss, &Health)>, mut gizmos: Gizmos) {
    if let Ok((_boss, health)) = query.single() {
        let health = health.current as f32 / health.max as f32;
        let _health = health.clamp(0.0, 1.0);

        gizmos.line_2d(Vec2::ZERO, Vec2::new(50., 50.), Color::srgb(255., 0., 0.));
    }
}
