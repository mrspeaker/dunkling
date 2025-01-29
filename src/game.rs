use avian3d::prelude::*;

use bevy::{
    prelude::*,
    color::palettes::css::*,
    color::palettes::tailwind::*,
    picking::pointer::PointerInteraction,
};

use std::f32::consts::*;

use crate::constants::{
    STONE_RADIUS,
    STONE_DAMPENING,
};
use crate::camera::CameraPlugin;
use crate::sheet::{SheetPlugin, Sheet, TerrainSculpt};
use crate::townsfolk::TownsfolkPlugin;

pub struct GamePlugin;

#[derive(Component)]
pub struct Stone;

#[derive(Component)]
struct Spotty;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MeshPickingPlugin,
//            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()));
        app.add_plugins(CameraPlugin);
        app.add_plugins(SheetPlugin);
        app.add_plugins(TownsfolkPlugin);

        app.add_systems(Startup, setup);
        app.add_systems(Update, (
            terrain_mouse,
            stone_shoot,
            draw_mesh_intersections
        ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // stone
    commands.spawn((
        Stone,
        RigidBody::Dynamic,
        Collider::sphere(STONE_RADIUS),
        LinearDamping(STONE_DAMPENING),
        //Friction::new(10.0),
        //CollisionMargin(0.1),
        //Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, STONE_RADIUS * 2.0, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            range: 100.0,
            radius: 100.0,
            color: BLUE.into(),
            shadows_enabled: true,
            ..default()
        },
        Spotty
    ));

    // Thor
    let texture_handle = asset_server.load("thor.png");
    let aspect = 1.0;//0.25;
    let quad_width = STONE_RADIUS * 10.0;
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        double_sided: true,
        cull_mode: None, //Some(Face::Back)
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(quad_width, quad_width * aspect))),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(STONE_RADIUS * 2.0, 1.0, STONE_RADIUS * 3.0)
            .with_rotation(Quat::from_euler(
                // YXZ = "yaw"/"pitch"/"roll"
                EulerRot::YXZ,
                (180.0_f32).to_radians(),
                (0.0_f32).to_radians(),
                (0.0_f32).to_radians(),
            ))
    ));

    // Lights
    commands.insert_resource(AmbientLight {
        color: ORANGE_RED.into(),
        brightness: 100.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.0),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(100.0, 100.0, 50.0),
            rotation: Quat::from_rotation_x(-PI * 0.9),
            ..default()
        },
    ));

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

fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 5.0, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 5.0, PINK_100);
    }
}
