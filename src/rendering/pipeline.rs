use bevy::{
    asset::AssetServer,
    core_pipeline::FullscreenShader,
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Res},
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, Operations,
            PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, TextureViewId,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex},
        view::ViewTarget,
    },
    utils::default,
};

const LAYOUT_BIND_GROUP: &str = "pixel_post_process_layout_bind_group";
const BIND_GROUP_LABEL: &str = "pixel_post_process_bind_group";
const PIPELINE_LABEL: &str = "pixel_post_process_pipeline";
const RENDER_PASS_LABEL: &str = "pixel_post_process_render_pass";
//const SHADER_PATH: &str = "embedded://isometric/rendering/shaders/pipeline.wgsl";
const SHADER_PATH: &str = "shaders/post_processing.wgsl";

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(crate) struct PixelPostProcessSettings {
    pub intensity: f32,
}

#[derive(Default)]
pub(crate) struct PostProcessBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

#[derive(Resource)]
pub(crate) struct PostProcessPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_post_process_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    // LAYOUT
    let layout = BindGroupLayoutDescriptor::new(
        LAYOUT_BIND_GROUP,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<PixelPostProcessSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = asset_server.load(SHADER_PATH);

    let vertext_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(PIPELINE_LABEL.into()),
        layout: vec![layout.clone()],
        vertex: vertext_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(PostProcessPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

pub(crate) fn pixel_post_process_system(
    view: ViewQuery<(
        &ViewTarget,
        &PixelPostProcessSettings,
        &DynamicUniformIndex<PixelPostProcessSettings>,
    )>,
    post_process_pipeline: Option<Res<PostProcessPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<PixelPostProcessSettings>>,
    mut cache: Local<PostProcessBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(post_process_pipeline) = post_process_pipeline else {
        return;
    };

    let (view_target, _post_process_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    //Start render pass
    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,

        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                BIND_GROUP_LABEL,
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    post_process.source,
                    &post_process_pipeline.sampler,
                    settings_binding.clone(),
                )),
            );

            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some(RENDER_PASS_LABEL.into()),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

    render_pass.set_pipeline(pipeline);
    render_pass.set_bind_group(0, bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);
}
