mod r#box;
mod ball;
mod config;
mod cursor;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_xpbd_2d::prelude::*;
use crate::ball::BallPlugin;
use crate::config::Config;
use crate::cursor::CursorPlugin;
use crate::r#box::BoxPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((CursorPlugin, BoxPlugin, BallPlugin))
        .add_systems(Startup, (setup))
        .init_resource::<Config>()
        .run();
}

#[derive(Component, Debug, Clone)]
pub struct MainCamera;

fn setup(mut commands: Commands, mut debug_render_config: ResMut<PhysicsDebugConfig>, mut narrow_phase_config: ResMut<NarrowPhaseConfig>) {
    commands.spawn((Camera2dBundle {
        projection: OrthographicProjection {
            near: -100.0,
            far: 100.0,
            scaling_mode: ScalingMode::AutoMin {
                min_width: 2.0,
                min_height: 2.0,
            },
            ..Default::default()
        },
        ..Default::default()
    }, MainCamera));
    // This rigid body and its collider and AABB will get rendered
    // commands.spawn((
    //     RigidBody::Dynamic,
    //     Collider::ball(0.025),
    //     // Overwrite default collider color (optional)
    //     DebugRender::default().with_collider_color(Color::RED),
    // ));


    debug_render_config.axis_lengths = Some(Vec2::new(0.01, 0.01));
    //debug_render_config.aabb_color = Some(Color::WHITE);
    //debug_render_config.axis_lengths = None;

    debug_render_config.enabled = false;


    narrow_phase_config.prediction_distance = 0.01;
}