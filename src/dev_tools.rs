//! Development tools for the game. This plugin is only enabled in dev builds.

use crate::screens::Screen;
use bevy::{
    dev_tools::{
        fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
        states::log_transitions,
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            refresh_interval: core::time::Duration::from_millis(100),
            enabled: false,
            text_config: TextFont {
                font_size: 20.0,
                ..default()
            },
            frame_time_graph_config: FrameTimeGraphConfig {
                enabled: false,
                // The minimum acceptable fps
                min_fps: 30.0,
                // The target fps
                target_fps: 100.0,
            },
            ..FpsOverlayConfig::default()
        },
    });

    app.add_systems(Update, toggle_debug_ui);
}

fn toggle_debug_ui(input: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<FpsOverlayConfig>) {
    if input.just_released(KeyCode::F11) {
        overlay.frame_time_graph_config.enabled = !overlay.frame_time_graph_config.enabled;
    }

    if input.just_released(KeyCode::F12) {
        overlay.enabled = !overlay.enabled;
    }
}
