use super::player::ShipAssets;
use crate::{AppSystems, PausableSystems, screens::Screen};
use avian2d::prelude::*;
use bevy::prelude::*;

const PROJECTILE_SPEED: f32 = 500.0;
const PROJECTILE_FORWARD_SPAWN_SCALAR: f32 = 30.0;
const PROJECTILE_DESPAWN_TIME_SECONDS: f32 = 2.0;
const WEAPON_FIRE_RATE: f32 = 0.16;

#[derive(Event)]
pub struct FireWeapon {
    pub entity: Entity,
}

#[derive(Component, Debug)]
pub struct Weapon {
    pub fire_rate_timer: Timer,
}
impl Weapon {
    pub fn new() -> Self {
        Self {
            fire_rate_timer: Timer::from_seconds(WEAPON_FIRE_RATE, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug)]
pub struct Projectile {
    pub despawn_timer: Timer,
}

pub(super) fn plugin(app: &mut App) {
    app.add_event::<FireWeapon>().add_systems(
        Update,
        (
            (tick_weapon_cooldown, tick_projectile_timers)
                .chain()
                .in_set(AppSystems::TickTimers),
            fire_weapon
                .run_if(resource_exists::<ShipAssets>)
                .in_set(AppSystems::RecordInput),
            despawn_projectile.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// progress timers for tracking weapon cooldown after firing
fn tick_weapon_cooldown(mut weapons: Query<&mut Weapon>, time: Res<Time>) {
    for mut weapon in weapons.iter_mut() {
        weapon.fire_rate_timer.tick(time.delta());
    }
}

/// progress timers for tracking projectile despawning
fn tick_projectile_timers(mut query: Query<&mut Projectile, With<Projectile>>, time: Res<Time>) {
    for mut projectile in query.iter_mut() {
        projectile.despawn_timer.tick(time.delta());
    }
}

fn fire_weapon(
    mut commands: Commands,
    mut weapons: Query<(&Transform, &mut Weapon)>,
    ship_assets: Res<ShipAssets>,
    mut weapon_fired: EventReader<FireWeapon>,
) {
    for event in weapon_fired.read() {
        let trigger_entity = event.entity;

        // find weapon on trigger entity
        let Ok((transform, mut weapon)) = weapons.get_mut(trigger_entity) else {
            return error!("failed to get entity to weapon to fire.");
        };

        // check if weapon timer is finished
        if weapon.fire_rate_timer.finished() {
            // reset timer
            weapon.fire_rate_timer = Timer::from_seconds(WEAPON_FIRE_RATE, TimerMode::Once);

            // fire projectile
            // calculate where to spawn the projectile (in front of player)
            let transform_vec: Vec3 =
                transform.translation + transform.up() * PROJECTILE_FORWARD_SPAWN_SCALAR;
            let linear_velocity: Vec3 = transform.up() * PROJECTILE_SPEED;

            commands.spawn((
                StateScoped(Screen::Gameplay),
                RigidBody::Dynamic,
                LinearVelocity(linear_velocity.xy()),
                Collider::circle(100.0),
                MassPropertiesBundle::from_shape(&Collider::circle(100.0), 1.0),
                Sensor,
                Sprite::from_image(ship_assets.projectile.clone()),
                Transform::from_translation(transform_vec).with_scale(Vec3::splat(0.03)),
                Projectile {
                    despawn_timer: Timer::from_seconds(
                        PROJECTILE_DESPAWN_TIME_SECONDS,
                        TimerMode::Once,
                    ),
                },
            ));
        }
    }
}

/// Handle despawning projectiles
fn despawn_projectile(
    mut commands: Commands,
    mut query: Query<(Entity, &Projectile), With<Projectile>>,
) {
    for (entity, projectile) in query.iter_mut() {
        if projectile.despawn_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
