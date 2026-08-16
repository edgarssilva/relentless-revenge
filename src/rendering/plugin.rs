use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{embedded_asset, Assets},
    camera::{
        visibility::RenderLayers, Camera2d, Camera3d, OrthographicProjection, Projection,
        RenderTarget,
    },
    core_pipeline::{Core3d, Core3dSystems},
    ecs::{
        component::Component,
        message::MessageReader,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    math::{Vec2, Vec3},
    render::{
        extract_component::ExtractComponentPlugin, render_resource::TextureFormat,
        uniform::UniformComponentPlugin, RenderApp, RenderStartup,
    },
    sprite::Sprite,
    transform::components::Transform,
    utils::default,
    window::WindowResized,
};

use crate::{
    helper::IsometricCameraFollow,
    rendering::pipeline::{
        init_post_process_pipeline, pixel_post_process_system, PixelPostProcessSettings,
    },
};

const TARGET_WIDTH: u32 = 480;
const TARGET_HEIGHT: u32 = 270;

#[derive(Resource)]
pub struct PixelCamera {
    pub target_width: u32,
    pub target_height: u32,

    pub world_layer: RenderLayers,
    pub camera_layer: RenderLayers,
}

#[derive(Component)]
struct PixelTexture;

pub struct PixelRenderPlugin;

impl Plugin for PixelRenderPlugin {
    fn build(&self, app: &mut App) {
        //TODO: Should propable handle this better, and with a loading state
        //embedded_asset!(app, "shaders/pixel_pipeline.wgsl");
        embedded_asset!(app, "shaders/post_processing.wgsl");
        app.insert_resource(PixelCamera {
            target_width: TARGET_WIDTH,
            target_height: TARGET_HEIGHT,
            world_layer: RenderLayers::layer(0),
            camera_layer: RenderLayers::layer(1),
        })
        .add_systems(Startup, setup_pixel_camera)
        .add_systems(Update, on_window_resize)
        .add_plugins((
            ExtractComponentPlugin::<PixelPostProcessSettings>::default(),
            UniformComponentPlugin::<PixelPostProcessSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_post_process_pipeline);
        render_app.add_systems(
            Core3d,
            pixel_post_process_system.in_set(Core3dSystems::PostProcess),
        );
    }
}

fn setup_pixel_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    pixel_camera: Res<PixelCamera>,
) {
    let image = Image::new_target_texture(
        pixel_camera.target_width,
        pixel_camera.target_height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    let image_handle = images.add(image);

    commands.spawn((
        IsometricCameraFollow {
            offset: Vec3::new(10.0, -10.0, 10.0),
            smoothness: 9.0,
        },
        RenderTarget::Image(image_handle.clone().into()),
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            /*scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 30.0, // Clean zoom level
            },*/
            scale: 0.4,
            near: -1000.0, // Expands the front clipping plane
            far: 1000.0,   // Expands the back clipping plane
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(100.0, -100.0, 100.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
        PixelPostProcessSettings { intensity: 0.02 },
    ));

    commands.spawn((
        PixelTexture,
        Transform::default(),
        Sprite {
            image: image_handle.clone(),
            //custom_size: Some(Vec2::new(1080., 720.)),
            image_mode: bevy::sprite::SpriteImageMode::Scale(
                bevy::sprite::SpriteScalingMode::FitCenter,
            ),
            ..default()
        },
        pixel_camera.camera_layer.clone(),
    ));

    commands.spawn((Camera2d::default(), pixel_camera.camera_layer.clone()));
}

fn on_window_resize(
    mut query: Query<&mut Sprite, With<PixelTexture>>,
    mut resize_events: MessageReader<WindowResized>,
) {
    if let Some(event) = resize_events.read().last() {
        if let Ok(mut sprite) = query.single_mut() {
            sprite.custom_size = Some(Vec2::new(event.width, event.height));
        }
    }
}
