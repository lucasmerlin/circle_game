use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct Config {
    pub drop_height: f32,
    pub small_ball_radius: f32,

    pub grow_time: f32,

    pub max_random_level: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            drop_height: 0.6,
            small_ball_radius: 0.025,
            grow_time: 0.25,

            max_random_level: 4,
        }
    }
}