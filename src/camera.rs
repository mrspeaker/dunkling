use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_atmosphere::settings::SkyboxCreationMode;
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

use crate::constants::{STONE_RADIUS, STONE_Y};
use crate::game::{Stone, GameState};

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
            modifier_orbit: Some(KeyCode::ShiftLeft),
            zoom_lower_limit: 100.0,
            zoom_upper_limit: Some(500.0),
            ..default()
        }
    ));

    commands.insert_resource(AtmosphereSettings {
        skybox_creation_mode: SkyboxCreationMode::FromSpecifiedFar(3000.0),
        ..default()
    });
}

pub fn cam_track(
    //time: Res<Time>,
    stone: Query<&Transform, (With<Stone>, Without<TrackingCamera>)>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
){
    //let dt = time.delta_secs();
    let Ok(stone_pos) = stone.get_single() else { return; };

    let Ok(mut camera) = camera.get_single_mut() else { return; };

    let dist = stone_pos.translation.distance(camera.translation);
/*    if stone_pos.translation.z > -5.0 && dist > STONE_RADIUS * 3.0  {

        //let move_amount = camera.forward();
        //camera.translation += move_amount * 5.0 * dt;
        camera.look_at(
            stone_pos.translation +
                Vec3::new(0.0, STONE_RADIUS * 5.0, STONE_RADIUS * 3.0)
                , Dir3::Y);
    } else {
*/
    let move_amount = STONE_RADIUS * 10.0; //camera.forward() * 10.0;
    camera.translation.y = stone_pos.translation.y + STONE_RADIUS * 2.0;
    camera.translation.z = stone_pos.translation.z - move_amount ;//* 10.0 * dt;
    camera.look_at(stone_pos.translation
                   + Vec3::new(0.0, 0.0, STONE_RADIUS * 20.0), Dir3::Y);// + Vec3::new(0.0, 1.0, 0.0), Dir3::Y);
    //  }


}


pub fn cam_track_orbit(
    //time: Res<Time>,
    stone: Query<&Transform, With<Stone>>,
    mut camera: Query<(&Transform, &mut PanOrbitCamera)>,
){
    //let dt = time.delta_secs();
    let Ok(stone_pos) = stone.get_single() else { return; };

    let Ok((t, mut camera)) = camera.get_single_mut() else { return; };

    camera.target_focus = stone_pos.translation;
    camera.force_update = true;
}

fn add_atmos(
    camera: Query<Entity, With<TrackingCamera>>,
    mut commands: Commands
) {
    let Ok(camera) = camera.get_single() else { return; };
    commands.entity(camera).insert(AtmosphereCamera::default());
}

fn remove_atmos(
    camera: Query<Entity, With<TrackingCamera>>,
    mut commands: Commands
) {
    let Ok(camera) = camera.get_single() else { return; };
    commands.entity(camera).remove::<AtmosphereCamera>();
}
