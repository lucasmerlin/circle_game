use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_xpbd_2d::prelude::*;
use rand::random;

use crate::config::Config;
use crate::cursor::WorldCursorPosition;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_system,
                aim_system,
                drop_system,
                ball_collision_system,
                merge_into_system,
                grow_system,
            ),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Level(pub usize);

impl Level {

    pub const MAX: Level = Level(11);

    pub fn scale(&self) -> f32 {
        f32::powf(1.25, self.0 as f32)
    }

    pub fn random(max: Level) -> Level {
        Level(random::<usize>() % max.0 + 1)
    }

    // HSL pastel color based on level
    pub fn color(&self) -> Color {
        let hue = (self.0 as f32 / Self::MAX.0 as f32) * 360.0;
        let saturation = 1.0;
        let lightness = 0.77;

        Color::hsl(hue, saturation, lightness)
    }
}

#[derive(Component, Debug, Clone)]
pub struct Ball {
    pub level: Level,
    pub state: BallState,
}

#[derive(Debug, Clone)]
pub enum BallState {
    Aiming,
    Normal,
    MergingInto,
    Growing,
}

#[derive(Component, Debug, Clone)]
pub struct Aiming;

#[derive(Component, Debug, Clone)]
pub struct Dropping;

#[derive(Component, Debug, Clone)]
pub struct MergingInto {
    pub target: Entity,
    pub timer: Timer,
    pub from_position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct Growing {
    pub timer: Timer,
    pub from_level: Level,
}

pub fn spawn_system(
    mut commands: Commands,
    current_aiming_query: Query<&Ball, With<Aiming>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    config: Res<Config>,
    assets: Res<AssetServer>,
    current_max_level_query: Query<&Ball>,
) {
    if current_aiming_query.iter().count() > 0 {
        return;
    }

    let current_max_level = current_max_level_query
        .iter()
        .map(|ball| ball.level)
        .max()
        .unwrap_or(Level(1));

    let max_level = config.max_random_level.min(current_max_level.0);

    let level = Level::random(Level(max_level));

    let cursor_position = q_windows.single().cursor_position();

    let spawn_position = Vec2::new(
        cursor_position.map(|v| v.x).unwrap_or(0.0),
        config.drop_height,
    );

    commands.spawn((
        Ball {
            level,
            state: BallState::Aiming,
        },
        Aiming,
        SpriteBundle {
            texture: assets.load("circle.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    config.small_ball_radius * 2.0,
                    config.small_ball_radius * 2.0,
                )),
                color: level.color(),
                ..Default::default()
            },
            transform: Transform::from_xyz(spawn_position.x, spawn_position.y, 0.0).with_scale(
                Vec3::new(
                    level.scale(),
                    level.scale(),
                    1.0,
                ),
            ),
            ..Default::default()
        },
    ));
}

pub fn aim_system(
    mut commands: Commands,
    mut current_aiming_query: Query<(Entity, &mut Transform), (With<Aiming>)>,
    cursor_position: Res<WorldCursorPosition>,
) {
    if let Ok((entity, mut transform)) = current_aiming_query.get_single_mut() {
        if let Some(cursor_position) = &cursor_position.0 {
            transform.translation.x = cursor_position.x;
        }
    }
}

pub fn drop_system(
    mut commands: Commands,
    mut current_aiming_query: Query<(Entity, &mut Ball, &mut Transform), (With<Aiming>)>,
    mouse_input: Res<Input<MouseButton>>,
    config: Res<Config>,
) {
    if let Ok((entity, mut ball, mut transform)) = current_aiming_query.get_single_mut() {
        if mouse_input.pressed(MouseButton::Left) {
            ball.state = BallState::Normal;
            commands.entity(entity).remove::<Aiming>().insert((
                Dropping,
                RigidBody::Dynamic,
                Collider::ball(config.small_ball_radius),
            ));
        }
    }
}

fn ball_collision_system(
    mut commands: Commands,
    mut events: EventReader<CollisionStarted>,
    mut balls: Query<(Entity, &mut Ball, &Transform, &mut LinearVelocity, &mut AngularVelocity), (Without<Aiming>)>,
    config: Res<Config>,
) {
    for CollisionStarted(a, b) in events.read() {
        let res = balls.get_many_mut([*a, *b]);

        if let Ok([a, b]) = res {
            let mut merge = |(from_entity, mut from_ball, from_transform, mut from_velocity, mut from_ang_velocity): (
                Entity,
                Mut<Ball>,
                &Transform,
                Mut<LinearVelocity>,
                Mut<AngularVelocity>,
            ),
                             (into_entity, mut into_ball, into_transform, mut into_velocity, mut into_ang_velocity): (
                Entity,
                Mut<Ball>,
                &Transform,
                Mut<LinearVelocity>,
                Mut<AngularVelocity>,
            )| {

                *from_velocity = LinearVelocity::ZERO;
                *into_velocity = LinearVelocity::ZERO;

                *from_ang_velocity = AngularVelocity::ZERO;
                *into_ang_velocity = AngularVelocity::ZERO;

                from_ball.state = BallState::MergingInto;
                commands
                    .entity(from_entity)
                    .remove::<RigidBody>()
                    .remove::<Collider>()
                    .remove::<LinearVelocity>()
                    .insert(MergingInto {
                        target: into_entity,
                        timer: Timer::new(Duration::from_secs_f32(config.grow_time), TimerMode::Once),
                        from_position: from_transform.translation.truncate(),
                    });

                into_ball.state = BallState::Growing;
                commands.entity(into_entity).insert(Growing {
                    timer: Timer::new(Duration::from_secs_f32(config.grow_time), TimerMode::Once),
                    from_level: into_ball.level,
                });

                into_ball.level = Level(into_ball.level.0 + 1);
            };

            if a.1.level == b.1.level {
                if a.3.length() > b.3.length() {
                    merge(b, a);
                } else {
                    merge(a, b);
                }
            }
        }
    }
}

fn merge_into_system(
    mut commands: Commands,
    time: Res<Time>,
    mut merge_query: Query<(Entity, &mut MergingInto, &mut Transform), (Without<Aiming>)>,
    into_query: Query<&Transform, (Without<Aiming>, Without<MergingInto>)>,
) {
    for (entity, mut merging_into, mut transform) in merge_query.iter_mut() {
        merging_into.timer.tick(time.delta());
        if let Ok(into_transform) = into_query.get(merging_into.target) {
            let t = merging_into.timer.percent();
            let from_position = merging_into.from_position;
            let into_position = into_transform.translation.truncate();

            let current_pos = from_position.lerp(into_position, t);

            transform.translation.x = current_pos.x;
            transform.translation.y = current_pos.y;

            if merging_into.timer.just_finished() {
                commands.entity(entity).despawn_recursive();
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn grow_system(
    mut commands: Commands,
    time: Res<Time>,
    mut grow_query: Query<(Entity, &Ball, &mut Growing, &mut Transform, &mut Sprite), (Without<Aiming>)>,
) {
    for (entity, ball, mut growing, mut transform, mut sprite) in grow_query.iter_mut() {
        growing.timer.tick(time.delta());
        let t = growing.timer.percent();

        let target_scale = ball.level.scale();
        let from_scale = growing.from_level.scale();

        let current_scale = lerp(from_scale, target_scale, t);

        transform.scale.x = current_scale;
        transform.scale.y = current_scale;

        let target_color = ball.level.color();
        let from_color = growing.from_level.color();

        let current_color = Color::rgb(
            lerp(from_color.r(), target_color.r(), t),
            lerp(from_color.g(), target_color.g(), t),
            lerp(from_color.b(), target_color.b(), t),
        );

        sprite.color = current_color;

        if growing.timer.just_finished() {
            commands.entity(entity).remove::<Growing>();
        }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
