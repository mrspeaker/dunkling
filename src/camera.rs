use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_atmosphere::settings::SkyboxCreationMode;
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

use crate::constants::STONE_RADIUS;
use crate::game::GameState;
use crate::stone::Stone;

use std::f32::consts::*;

pub fn camera_plugin(app: &mut App) {
    app.add_plugins((
        AtmospherePlugin,
        PanOrbitCameraPlugin
    ));

    app.add_systems(Startup, setup);
    app.add_systems(OnEnter(GameState::InGame), (add_atmos, reset_cam));
    app.add_systems(Update, cam_track_orbit);
    app.add_systems(OnExit(GameState::InGame), remove_atmos);
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, STONE_RADIUS * 4.0, -STONE_RADIUS * 10.0)
            .looking_at(Vec3::new(0.0, STONE_RADIUS / 2.0, 0.0), Dir3::Y),
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            pan_sensitivity: 0.0, // disable panning
            zoom_lower_limit: 100.0,
            zoom_upper_limit: Some(500.0),
            ..default()
        }
    ));

    commands.insert_resource(AtmosphereSettings {
        skybox_creation_mode: SkyboxCreationMode::FromSpecifiedFar(8000.0),
        ..default()
    });
}

pub fn cam_track_orbit(
    stone: Query<&Transform, With<Stone>>,
    mut camera: Query<&mut PanOrbitCamera>,
    keys: Res<ButtonInput<KeyCode>>,
){
    let Ok(stone_pos) = stone.get_single() else { return; };

    for mut cam in camera.iter_mut() {
        cam.target_focus = stone_pos.translation;
        cam.force_update = true;

        if keys.pressed(KeyCode::ShiftLeft) {
            cam.button_orbit = MouseButton::Left;
            cam.modifier_orbit = Some(KeyCode::ShiftLeft);
        } else {
            cam.button_orbit = MouseButton::Middle;
            cam.modifier_orbit = None;
        }
     }
}

fn add_atmos(
    mut camera: Query<Entity, With<PanOrbitCamera>>,
    mut commands: Commands
) {
    let Ok(e) = camera.get_single_mut() else { return; };
    commands.entity(e).insert(AtmosphereCamera::default());
}

fn remove_atmos(
    camera: Query<Entity, With<PanOrbitCamera>>,
    mut commands: Commands
) {
    let Ok(camera) = camera.get_single() else { return; };
    commands.entity(camera).remove::<AtmosphereCamera>();
}

fn reset_cam(
    mut camera: Query<&mut PanOrbitCamera>,
) {
    let Ok(mut cam) = camera.get_single_mut() else { return; };
    cam.target_yaw = PI;
    cam.target_pitch = 0.3805064; // i copied this from dgb printing... figure it out
    cam.target_radius = 107.7;
    cam.force_update = true;
}
