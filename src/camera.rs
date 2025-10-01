use bevy::ecs::{query::QuerySingleError, system::SystemParam};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(SystemParam)]
pub struct CursorPositionQuery<'w, 's> {
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Single<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera2d>>,
}

/// get world position of cursor
/// can fail if cursor is outside app window
impl CursorPositionQuery<'_, '_> {
    pub fn get_world_position(&self) -> Result<Vec2, QuerySingleError> {
        // get single instances of window and camera
        let window = *self.window;
        let (camera, camera_transform) = *self.camera;

        // get cursors position in window
        let Some(cursor_position) = window.cursor_position() else {
            return Err(QuerySingleError::NoEntities(
                "Cannot find window position of cursor!".into(),
            ));
        };

        // Calculate a world position based on the cursor's position.
        let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        else {
            return Err(QuerySingleError::NoEntities(
                "Cannot find world position of cursor!".into(),
            ));
        };

        Ok(world_position)
    }
}
