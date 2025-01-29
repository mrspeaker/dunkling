use avian3d::prelude::*;

use bevy::{
    prelude::*,
    color::palettes::tailwind::*,
    picking::pointer::PointerInteraction,
};

use crate::sheet::{Sheet, TerrainSculpt};
use crate::game::{Stone, Spotty};

use crate::constants::STONE_RADIUS;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            terrain_mouse,
            stone_shoot,
            draw_sheet_intersections
        ));
    }
}

#[derive(Default)]
struct LastMouse {
    idx: usize
}

fn terrain_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    windows: Single<&Window>,
    mut ray_cast: MeshRayCast,
    terrain_query: Query<(Entity, &Mesh3d), With<Sheet>>,
    mut last_mouse: Local<LastMouse>,
    mut commands: Commands,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }
    let is_shift = keys.pressed(KeyCode::ShiftLeft);

    // Cursor to ray
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let filter = |entity| terrain_query.contains(entity);
    // let early_exit_test = |_entity| false;
    let settings = RayCastSettings::default()
        .with_filter(&filter);
    let hits = ray_cast.cast_ray(ray, &settings);
    for (_e, rmh) in hits.iter() {
        if let Some(idx) = rmh.triangle_index {
            if idx != last_mouse.idx {
                commands.trigger(TerrainSculpt { up: !is_shift, idx });
                last_mouse.idx = idx;
            }
        }
    }
}

fn stone_shoot(
    input: Res<ButtonInput<KeyCode>>,
    mut stone: Query<(&mut Transform, &mut LinearVelocity), With<Stone>>,
    mut spotty: Query<&mut Transform, (With<Spotty>, Without<Stone>)>,
){
    let (mut stone_pos, mut vel_vec) = stone.single_mut();
    let power = 0.5;
    if input.pressed(KeyCode::KeyW) {
        vel_vec.z += power;
    }
    if input.pressed(KeyCode::KeyS) {
        vel_vec.z -= power;
    }
    if input.pressed(KeyCode::KeyA) {
        vel_vec.x += power;
    }
    if input.pressed(KeyCode::KeyD) {
        vel_vec.x -= power;
    }

    let mut spot_pos = spotty.single_mut();
    spot_pos.translation = stone_pos.translation + Vec3::new(1.0, STONE_RADIUS * 2.0, 1.0);

    if stone_pos.translation.y < -STONE_RADIUS * 5.0 {
        stone_pos.translation = Vec3::new(0.0, STONE_RADIUS, 0.0);
        vel_vec.x = 0.0;
        vel_vec.y = 0.0;
        vel_vec.z = 0.0;
    }

}

fn draw_sheet_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 5.0, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 5.0, PINK_100);
    }
}
