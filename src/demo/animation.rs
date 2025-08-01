//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    prelude::*,
};
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    demo::{
        movement::MovementController,
        player::{PlayerAssets, PlayerShipEngineEffect},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<PlayerAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
                execute_animations,
            )
                .chain()
                .run_if(resource_exists::<PlayerAssets>)
                .in_set(AppSystems::Update),
            (
                start_animation::<PlayerShipEngineEffect>.run_if(input_just_pressed(KeyCode::KeyW)),
                stop_animation::<PlayerShipEngineEffect>.run_if(input_just_released(KeyCode::KeyW)),
            )
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(&MovementController, &mut Sprite, &mut PlayerAnimation)>,
) {
    for (controller, mut sprite, mut animation) in &mut player_query {
        let dx = controller.intent.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }

        let animation_state = if controller.intent == Vec2::ZERO {
            PlayerAnimationState::Idling
        } else {
            PlayerAnimationState::Walking
        };
        animation.update_state(animation_state);
    }
}

/// Update the animation timer.
fn update_animation_timer(
    time: Res<Time>,
    mut query: Query<&mut PlayerAnimation>,
    mut animation_timers: Query<&mut AnimationTimer>,
) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }

    // Update player animation timers
    for mut timer in &mut animation_timers {
        timer.tick(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut step_query: Query<&PlayerAnimation>,
) {
    for animation in &mut step_query {
        if animation.state == PlayerAnimationState::Walking
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            let rng = &mut rand::thread_rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.spawn(sound_effect(random_step));
        }
    }
}

#[derive(Reflect, PartialEq)]
pub enum PlayerAnimationState {
    Idling,
    Walking,
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 2;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 6;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(50);

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
        }
    }

    fn walking() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Walking,
        }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Walking => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Walking => *self = Self::walking(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::Idling => self.frame,
            PlayerAnimationState::Walking => 6 + self.frame,
        }
    }
}

#[derive(Component)]
struct AnimationPlaying;

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);
impl AnimationTimer {
    pub fn with_fps(fps: f32) -> Self {
        Self(Timer::from_seconds(1.0 / fps, TimerMode::Repeating))
    }
}

// Execute playing animations only
fn execute_animations(
    mut query: Query<(&AnimationIndices, &AnimationTimer, &mut Sprite), With<AnimationPlaying>>,
) {
    for (animation_indices, animation_timer, mut sprite) in &mut query {
        if let Some(atlas) = &mut sprite.texture_atlas {
            // If it has been displayed for the user-defined amount of time (fps)
            if animation_timer.just_finished() {
                if atlas.index == animation_indices.last {
                    // if last frame, reset to first
                    atlas.index = animation_indices.first;
                } else {
                    // otherwise, progress to next frame
                    atlas.index += 1;
                }
            }
        }
    }
}

fn start_animation<T: Component>(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Sprite, &mut AnimationTimer, &AnimationIndices),
        (With<T>, Without<AnimationPlaying>),
    >,
) {
    for (entity, mut sprite, mut animation_timer, animation_indices) in query.iter_mut() {
        if let Some(atlas) = &mut sprite.texture_atlas {
            // reset first animation frame
            atlas.index = animation_indices.first;
            animation_timer.reset();

            commands
                .entity(entity)
                .insert((Visibility::Visible, AnimationPlaying));
        }
    }
}

fn stop_animation<T: Component>(
    mut commands: Commands,
    query: Query<Entity, (With<T>, With<AnimationPlaying>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<AnimationPlaying>()
            .insert(Visibility::Hidden);
    }
}
