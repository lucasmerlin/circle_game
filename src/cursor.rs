use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::MainCamera;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursorPosition>()
            .add_systems(Update, my_cursor_system);
    }
}


/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct WorldCursorPosition(pub(crate) Option<Vec2>);

fn my_cursor_system(
    mut mycoords: ResMut<WorldCursorPosition>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    mycoords.0 = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());
}