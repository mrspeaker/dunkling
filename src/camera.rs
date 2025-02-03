use bevy::prelude::*;
//use bevy::core_pipeline::Skybox;
use bevy_atmosphere::prelude::*;

use crate::constants::STONE_RADIUS;
use crate::game::Stone;

pub struct CameraPlugin;

#[derive(Component)]
pub struct TrackingCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, cam_track);
        app.add_plugins(AtmospherePlugin);

    }
}

fn setup(
    mut commands: Commands,
) {
    //let skybox_handle = asset_server.load("cubemap.png");
    // Camera
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        /*Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
            ..default()
        },*/
        Transform::from_xyz(0.0, STONE_RADIUS * 4.0, STONE_RADIUS * 10.0)
            .looking_at(Vec3::new(0.0, STONE_RADIUS / 2.0, 0.0), Dir3::Y),
        TrackingCamera,
    ));
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
        camera.translation.y = STONE_RADIUS * 5.0;
        camera.translation.z = stone_pos.translation.z - move_amount ;//* 10.0 * dt;
        camera.look_at(stone_pos.translation
                       + Vec3::new(0.0, 0.0, STONE_RADIUS * 20.0), Dir3::Y);// + Vec3::new(0.0, 1.0, 0.0), Dir3::Y);
  //  }


}
