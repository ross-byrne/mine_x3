//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use crate::{AppSystems, PausableSystems, camera::CursorPositionQuery, demo::player::Player};
use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<ScreenWrap>();

    app.add_systems(
        Update,
        (
            apply_player_movement,
            apply_player_rotation,
            apply_screen_wrap,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

#[derive(Component)]
pub struct ShipSpeed(pub f32);

#[derive(Component, Debug)]
pub struct RotationSpeed(pub f32);

// TODO: update player movement to be closer to this
fn _apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;
        transform.translation += velocity.extend(0.0) * time.delta_secs();
    }
}

/// Applies movement to player. TODO: use movement controller here
fn apply_player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &MovementController,
            &Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &ShipSpeed,
            &RotationSpeed,
        ),
        With<Player>,
    >,
) {
    for (
        _controller,
        transform,
        mut linear_velocity,
        _angular_velocity,
        ship_speed,
        _rotation_speed,
    ) in query.iter_mut()
    {
        let _default_rotation_factor = 0.0;
        let mut movement_factor = 0.0;

        // let velocity = controller.max_speed * controller.intent;

        // if keyboard_input.pressed(KeyCode::KeyA) {
        //     default_rotation_factor += rotation_speed.0;
        // }

        // if keyboard_input.pressed(KeyCode::KeyD) {
        //     default_rotation_factor -= rotation_speed.0;
        // }

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement_factor += 1.0;
        }

        // set rotation factor
        // angular_velocity.0 = default_rotation_factor;

        // get the ship's forward vector by applying the current rotation to the ships initial facing
        // vector
        let movement_direction = transform.rotation * Vec3::Y;
        // get the distance the ship will move based on direction, the ship's movement speed and delta
        // time
        let movement_distance = movement_factor * ship_speed.0;
        // create the change in translation using the new movement direction and distance
        let translation_delta = movement_direction * movement_distance;

        // update the ship translation with our new translation delta
        linear_velocity.x = translation_delta.x;
        linear_velocity.y = translation_delta.y;
    }
}

/// Rotate player towards cursor
fn apply_player_rotation(
    time: Res<Time>,
    input: Res<ButtonInput<MouseButton>>,
    cursor_position: CursorPositionQuery,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    // Only rotate towards cursor while holding button
    if !input.pressed(MouseButton::Right) {
        return;
    }

    // Get the cursor translation in 2D
    let Ok(cursor_translation) = cursor_position.get_world_position() else {
        return; // cursor not in primary window
    };

    // Check how close cursor is to player
    let distance = cursor_translation.distance(player_transform.translation.xy());
    if distance <= 50.0 {
        return; // Player too close to cursor
    }

    // Get the player ship forward vector in 2D (already unit length)
    let player_forward = (player_transform.rotation * Vec3::Y).xy();

    // Get the vector from the player ship to the cursor in 2D and normalize it.
    let to_cursor = (cursor_translation - player_transform.translation.xy()).normalize();

    // Get the dot product between the player forward vector and the direction to the cursor.
    let forward_dot_cursor = player_forward.dot(to_cursor);

    // If the dot product is approximately 1.0 then the player is already facing the cursor and we can early out.
    if (forward_dot_cursor - 1.0).abs() < f32::EPSILON {
        return;
    }

    // Get the right vector of the player ship in 2D (already unit length)
    let player_right = (player_transform.rotation * Vec3::X).xy();

    // Get the dot product of the player right vector and the direction to the cursor.
    // If the dot product is negative them we need to rotate counter clockwise, if it is
    // positive we need to rotate clockwise. Note that `copysign` will still return 1.0 if the
    // dot product is 0.0 (because the cursor is directly behind the player, so perpendicular
    // with the right vector).
    let right_dot_cursor = player_right.dot(to_cursor);

    // Determine the sign of rotation from the right dot cursor. We need to negate the sign
    // here as the 2D bevy co-ordinate system rotates around +Z, which is pointing out of the
    // screen. Due to the right hand rule, positive rotation around +Z is counter clockwise and
    // negative is clockwise.
    let rotation_sign = -f32::copysign(1.0, right_dot_cursor);

    // Limit rotation so we don't overshoot the target. We need to convert our dot product to
    // an angle here so we can get an angle of rotation to clamp against.
    let max_angle = ops::acos(forward_dot_cursor.clamp(-1.0, 1.0)); // Clamp acos for safety

    // Calculate angle of rotation with limit
    let rotation_speed = f32::to_radians(360.0);
    let rotation_angle = rotation_sign * (rotation_speed * time.delta_secs()).min(max_angle);

    // Rotate the player to face the cursor
    player_transform.rotate_z(rotation_angle);
}

/// Wrap objects when they go off screen
fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
