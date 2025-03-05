use avian3d::prelude::*;

use bevy::{
    prelude::*,
    color::palettes::tailwind::*,
    picking::pointer::PointerInteraction,
};

use crate::game::{GamePhase, OnGameScreen, Spotty, BigThor};
use crate::sheet::{Sheet, TerrainSculpt};
use crate::stone::Stone;

use crate::constants::{
    CHUNK_SIZE,
    MIN_SCULT_DIST_FROM_STONE,
    STONE_RADIUS,
    SHEET_PRE_AREA,
    STONE_X,
    STONE_Y,
    STONE_Z, STONE_HURL_POWERUP_TIME,
};

const INIT_X:f32 = STONE_RADIUS * 10.0;

#[derive(Debug, Event)]
pub struct HurlStone {
    pub power: f32,
    pub angle: f32,
}
#[derive(Debug, Event)]
pub struct HurlAimAndPower {
    pub power: f32,
    pub angle: f32,
    pub reset: bool
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
            aim_and_powerup_for_hurl,
        ).run_if(in_state(GamePhase::Aiming)));
        app.add_systems(OnEnter(GamePhase::Aiming), setup_aim);
        app.add_observer(do_powerup_viz);
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
        Transform::from_xyz(INIT_X, STONE_RADIUS * 9.0, -CHUNK_SIZE + SHEET_PRE_AREA * 2.0),
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
    stone_query: Query<&Transform, With<Stone>>,
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

    let stone_pos_opt = stone_query.get_single();

    let filter = |entity| terrain_query.contains(entity);
    // let early_exit_test = |_entity| false;
    let settings = RayCastSettings::default()
        .with_filter(&filter);
    let hits = ray_cast.cast_ray(ray, &settings);
    for (e, rmh) in hits.iter() {

        // Don't sculpt if too close to stone
        if let Ok(pos) = stone_pos_opt {
            let dist_to_stone = pos.translation.distance(rmh.point);
            if dist_to_stone < MIN_SCULT_DIST_FROM_STONE {
                continue;
            }
        }

        if let Some(idx) = rmh.triangle_index {
            let dist_mouse_moved = rmh.point.xz().distance(last_mouse.pos.xz());
            if dist_mouse_moved > 1.0 {
                commands.trigger_targets(
                    TerrainSculpt {
                        up: is_right,
                        idx,
                        p1: rmh.point,
                    },
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
    if x_dist > CHUNK_SIZE || y_dist < -STONE_RADIUS * 12.0 {
        // TODO: this should transition to phase
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
    power_up: bool,
    power: f32,
    angle: f32
}

fn aim_and_powerup_for_hurl(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut aim: Local<Aiming>,
    time: Res<Time>,
    mut commands: Commands
) {
    if keys.pressed(KeyCode::ShiftLeft) {
        return;
    }
    if buttons.just_pressed(MouseButton::Left) {
        aim.power_up = true;
    }

    // Set angle based on cursor position
    let window = windows.single();
    window
        .cursor_position()
        .map(|v| { v / window.size() })
        .and_then(|pos| { Some(aim.angle = pos.x - 0.5) });

    if aim.power_up  {
        aim.power = (aim.power + time.delta_secs()).min(STONE_HURL_POWERUP_TIME);
    }
    let ratio = aim.power / STONE_HURL_POWERUP_TIME;

    let mut fired = false;
    if buttons.just_released(MouseButton::Left) {
        if aim.power > STONE_HURL_POWERUP_TIME * 0.3 {
            fired = true;
        }
        // Reset
        aim.power_up = false;
        aim.power = 0.0;
    }

    // Set Aiming params (or reset if fired)
    commands.trigger(HurlAimAndPower {
        power: if fired { 0.0 } else { ratio },
        angle: if fired { 0.0 } else { aim.angle },
        reset: fired
    });
    if fired {
        commands.trigger(HurlStone { power: ratio, angle: aim.angle });
    }

}


/// Move thor and powerball to show aim and power
fn do_powerup_viz(
    trigger: Trigger<HurlAimAndPower>,
    mut powerball: Query<&mut Transform, With<PowerBall>>,
    mut thor: Query<&mut Transform, (With<BigThor>, Without<PowerBall>)>

) {
    let Ok(mut pball) = powerball.get_single_mut() else { return; };
    let Ok(mut thor) = thor.get_single_mut() else { return; };
    let ev = trigger.event();
    let ratio = ev.power;

    if ev.reset {
        thor.rotation.x = 0.0;
    } else {
        pball.translation.x = INIT_X - ratio * 200.0;
        thor.rotation.x = -(ratio * 0.5).sin();
        thor.rotation.y = ev.angle;
    }
}
