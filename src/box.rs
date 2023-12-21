use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

pub struct BoxPlugin;

impl Plugin for BoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (trigger_shake, shake_system));
    }
}

#[derive(Component, Debug, Clone)]
pub struct Box;

#[derive(Component, Debug, Clone)]
pub struct Shaking {
    timer: Timer,
}


pub fn setup(mut commands: Commands) {
    let width = 1.0;
    let height = 2.0;

    let wall_thickness = 0.1;

    // Box shaped like I_I

    commands.spawn((
        Box,
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
    )).with_children(|parent| {
        parent.spawn((
            SpatialBundle::from_transform(Transform::from_xyz(-width / 2.0, 0.0, 0.0)),
            RigidBody::Kinematic,
            Collider::cuboid(wall_thickness, height),
        ));
        parent.spawn((
            SpatialBundle::from_transform(Transform::from_xyz(width / 2.0, 0.0, 0.0)),
            RigidBody::Kinematic,
            Collider::cuboid(wall_thickness, height),
        ));
        parent.spawn((
            SpatialBundle::from_transform(Transform::from_xyz(0.0, -height / 2.0, 0.0)),
            RigidBody::Kinematic,
            Collider::cuboid(width, wall_thickness),
        ));
    });
}

pub fn trigger_shake(mut commands: Commands, input: Res<Input<KeyCode>>, mut query: Query<Entity, With<Box>>) {
    if input.just_pressed(KeyCode::Space) {
        for entity in query.iter_mut() {
            commands.entity(entity).insert(Shaking {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
            });
        }
    }
}

// Sinus like shake function that starts and ends at 0 and has a peak of 1 at t=0.5
// Should shake back and forth three times and slowly ramp up and down the magnitude
pub fn shake_easing(t: f32) -> f32 {
    (t * std::f32::consts::PI * 4.0).sin()
}

pub fn shake_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut Shaking)>) {
    for (mut transform, mut shaking) in query.iter_mut() {

        shaking.timer.tick(time.delta());

        let t = shaking.timer.percent();

        let x = shake_easing(t) * 0.02;

        transform.translation.x = x;

    }
}