use avian3d::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy::{
    prelude::*,
    color::palettes::tailwind::*,
    picking::pointer::PointerInteraction,
};

use crate::sheet::{Sheet, TerrainSculpt};
use crate::game::{Stone, Spotty, BigThor};

use crate::constants::{
    STONE_RADIUS,
    SHEET_LENGTH,
    SHEET_PRE_AREA,
    STONE_X,
    STONE_Y,
    STONE_Z, SHEET_WIDTH,
};
use crate::game::{GameState, GamePhase, OnGameScreen};

const INIT_X:f32 = STONE_RADIUS * 10.0;

#[derive(Debug, Event)]
pub struct HurlStone {
    pub power: f32,
    pub angle: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            terrain_mouse,
            stone_update,
            draw_sheet_intersections
        ).run_if(in_state(GamePhase::Sculpting)));
        app.add_systems(Update, (
            aim_mouse,
        ).run_if(in_state(GamePhase::Aiming)));
        app.add_systems(OnEnter(GamePhase::Aiming), setup_aim);
    }
}

#[derive(Component)]
struct PowerBall;

fn setup_aim(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    commands.spawn((
        OnGameScreen,
        PowerBall,
        LinearVelocity(Vec3::new(0.0, 0.0, 160.0)),
        AngularVelocity(Vec3::new( 10.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS*0.5))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(INIT_X, STONE_RADIUS * 9.0, -SHEET_LENGTH + SHEET_PRE_AREA * 2.0),
    ));

}

#[derive(Default)]
struct LastMouse {
    idx: usize,
    pos: Vec3,
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
    let is_left = buttons.pressed(MouseButton::Left);
    let is_right = buttons.pressed(MouseButton::Right);
    let is_shift = keys.pressed(KeyCode::ShiftLeft);

    if is_shift || !(is_left || is_right) {
        return;
    }


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
    for (e, rmh) in hits.iter() {
        if let Some(idx) = rmh.triangle_index {
            let dist = rmh.point.xz().distance(last_mouse.pos.xz());
            if dist > 1.0 {
                commands.trigger_targets(
                    TerrainSculpt { up: is_right, idx, p1: last_mouse.pos, p2: rmh.point },
                    e.clone()
                );
                last_mouse.pos = rmh.point;
            }
            if idx != last_mouse.idx {
                last_mouse.idx = idx;
            }
        }
    }
}

fn stone_update(
    input: Res<ButtonInput<KeyCode>>,
    mut stone: Query<(&mut Transform, &mut LinearVelocity), With<Stone>>,
    mut spotty: Query<&mut Transform, (With<Spotty>, Without<Stone>)>,
){
    let Ok((mut stone_pos, mut vel_vec)) = stone.get_single_mut() else { return; };

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

    // update spot light
    let Ok(mut spot_pos) = spotty.get_single_mut() else { return; };
    spot_pos.translation = stone_pos.translation + Vec3::new(1.0, STONE_RADIUS * 2.0, 1.0);

    let x_dist = stone_pos.translation.x.abs();
    let y_dist = stone_pos.translation.y;
    if x_dist > SHEET_WIDTH || y_dist < -STONE_RADIUS * 12.0 {
        stone_pos.translation = Vec3::new(STONE_X, STONE_Y, STONE_Z);
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

#[derive(Default, Debug)]
struct Aiming {
    fired: bool,
    power_up: bool,
    power: f32,
    angle: f32
}

fn aim_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut aim: Local<Aiming>,
    time: Res<Time>,
    mut powerball: Query<&mut Transform, With<PowerBall>>,
    mut thor: Query<&mut Transform, (With<BigThor>, Without<PowerBall>)>,
    mut commands: Commands
) {
    if aim.fired {
        return;
    }

    let Ok(mut t) = powerball.get_single_mut() else { return; };

    for ev in motion.read() {
        aim.angle += ev.delta.x / 300.0;
        thor.get_single_mut().ok().map(|mut th| { th.rotation.y = aim.angle; });

        println!("Mouse moved: X: {} px, rot: {}", ev.delta.x, aim.angle);
    }

    if buttons.just_pressed(MouseButton::Left) {
        aim.power_up = true;
    }

    if aim.power_up  {
        aim.power += time.delta_secs();
        t.translation.x -= time.delta_secs() * 50.0;
    }

    if aim.power_up && buttons.just_released(MouseButton::Left) {
        if aim.power > 1.0 {
            // trigger fire!
            commands.trigger(HurlStone { power: aim.power, angle: aim.angle });
        }
        aim.power_up = false;
        aim.power = 0.0;
        t.translation.x = INIT_X;

    }
}
