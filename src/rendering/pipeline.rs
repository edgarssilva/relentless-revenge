use bevy::{
    core_pipeline::FullscreenShader,
    ecs::component::Component,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
struct PixelEffect {
    pub enabled: bool,
}

impl FullscreenShader for PixelEffect {
    fn fragment_shader() -> &'static str {
        //TODO: Change crate name
        "embedded://isometric/rendering/shaders/pipeline.wgsl".into()
    }

    fn node_edges() -> Vec<bevy::render::render_graph::InternedRenderLabel> {
        todo!()
    }
}
