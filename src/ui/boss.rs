use bevy::{
    prelude::{Commands, Component, Entity, Query, Res},
    time::{Time, Timer},
};
use bevy_egui::{
    egui::{self, Color32, Frame, RichText, Stroke},
    EguiContexts,
};

#[derive(Component)]
pub struct DomainName(pub String, pub Timer);

pub fn draw_domain_name(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut domain: Query<(&mut DomainName, Entity)>,
    time: Res<Time>,
) {
    if let Ok((mut domain, entity)) = domain.get_single_mut() {
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
            .show(contexts.ctx_mut(), |ui| {
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

        if domain.1.finished() {
            commands.entity(entity).despawn();
        }
    }
}
