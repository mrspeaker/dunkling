use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_atmosphere::settings::SkyboxCreationMode;
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

use crate::constants::STONE_RADIUS;
use crate::game::GameState;
use crate::stone::Stone;

use std::f32::consts::*;

pub struct CameraPlugin;

#[derive(Component)]
pub struct TrackingCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, cam_track_orbit);
        app.add_plugins(AtmospherePlugin);
        app.add_plugins(PanOrbitCameraPlugin);

        app.add_systems(OnEnter(GameState::InGame), add_atmos);
        app.add_systems(OnExit(GameState::InGame), remove_atmos);
    }
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, STONE_RADIUS * 4.0, -STONE_RADIUS * 10.0)
            .looking_at(Vec3::new(0.0, STONE_RADIUS / 2.0, 0.0), Dir3::Y),
        TrackingCamera,
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            //modifier_orbit: Some(KeyCode::ShiftLeft),
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
){
    let Ok(stone_pos) = stone.get_single() else { return; };

    for mut camera in camera.iter_mut() {
        camera.target_focus = stone_pos.translation;
        camera.force_update = true;
     }
}

fn add_atmos(
    mut camera: Query<(Entity, &mut PanOrbitCamera), With<PanOrbitCamera>>,
    mut commands: Commands
) {
    let Ok((e, mut cam)) = camera.get_single_mut() else { return; };
    commands.entity(e).insert(AtmosphereCamera::default());

    cam.target_yaw = PI;
    cam.target_pitch = 0.3805064; // i copied this from dgb printing... figure it out

    cam.force_update = true;
}


fn remove_atmos(
    camera: Query<Entity, With<PanOrbitCamera>>,
    mut commands: Commands
) {
    let Ok(camera) = camera.get_single() else { return; };
    commands.entity(camera).remove::<AtmosphereCamera>();
}
