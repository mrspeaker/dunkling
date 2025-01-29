use avian3d::prelude::*;

use bevy::{
    prelude::*,
    core_pipeline::Skybox,
    color::palettes::css::*,
    color::palettes::tailwind::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    picking::pointer::PointerInteraction,
};

use rand::prelude::*;
use std::f32::consts::*;

const STONE_RADIUS: f32 = 10.0;
const STONE_DAMPENING: f32 = 0.08;
const SUBS: u32 = 100;

#[derive(Component)]
struct TrackingCamera;

#[derive(Component)]
struct Stone;

#[derive(Component)]
struct Spotty;

#[derive(Component)]
struct BobX {
    dt: f32
}

#[derive(Component)]
struct Sheet;

#[derive(Debug, Event)]
pub struct TerrainSculpt {
    pub up: bool,
    pub idx: usize,
}


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin,
            MeshPickingPlugin,
//            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::linear_rgb(0.1,0.1, 0.),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            cam_track,
            terrain_mouse,
            stone_shoot,
            bob,
            draw_mesh_intersections
        ))
        .add_observer(terrain_sculpt)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    // sheet
    let width = 800.0;
    let length = 800.0;
    let pre_area = 50.0;

    // start line
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(STONE_RADIUS * 10.0, 0.1, STONE_RADIUS * 0.5))),
        MeshMaterial3d(materials.add(Color::BLACK)),
    ));

    // sheet v2
    let plane = Plane3d::default().mesh().size(length, length).subdivisions(SUBS);

    //let cube_mesh_handle: Handle<Mesh> = meshes.add(create_plane_mesh());
    commands.spawn((
        //Mesh3d(cube_mesh_handle),
        Mesh3d(meshes.add(plane)),
        RigidBody::Static,
        Friction::new(10.0),
        //Collider::cuboid(width, 0.3, length),
        //ColliderConstructor::ConvexHullFromMesh,
        ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
        CollisionMargin(0.05),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2,0.4, 0.))),
        Transform::from_xyz(
            0.0,
            0.0,
            -length / 2.0 + (pre_area / 2.0) )
            .with_rotation(Quat::from_rotation_y(PI / 4.001)),
        Wireframe,
        Sheet
    ));

/*    // jump
    commands.spawn((
        RigidBody::Static,
//        Friction::new(100.0),
        Collider::cuboid(width * 0.6, 0.3, 2.0),
        Mesh3d(meshes.add(Cuboid::new(width * 0.6, 0.3, 2.0))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(-width/2.0, 0.0, -10.0)
            .with_rotation(Quat::from_rotation_x(PI / 8.)),
        BobX{ dt: 0.0 }
    ));*/


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

    //let skybox_handle = asset_server.load("cubemap.png");
    // Camera
    commands.spawn((
        Camera3d::default(),
        /*Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
            ..default()
        },*/
        Transform::from_xyz(0.0, STONE_RADIUS * 4.0, STONE_RADIUS * 10.0)
            .looking_at(Vec3::new(0.0, STONE_RADIUS / 2.0, 0.0), Dir3::Y),
        TrackingCamera
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


    // Lil people
    let mut rng = rand::thread_rng();
    let w = STONE_RADIUS * 10.0;
    for _ in 0..200 {
        let pos = Vec3::new(
            rng.gen_range(-w..w),
            0.1,
            -rng.gen_range(0.0..length));
        commands
            .spawn((
                Name::new("Person1"),
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("person.glb"))),
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
                RigidBody::Dynamic,
//                Collider::cuboid(1.0, 1.0, 1.0),
                Transform::from_xyz(pos.x, pos.y, pos.z)));
                //.with_rotation(Quat::from_rotation_z(PI / 2.))
                 //   .with_scale(Vec3::splat(0.2))));

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
    for (e, rmh) in hits.iter() {
        if let Some(idx) = rmh.triangle_index {
            if idx != last_mouse.idx {
                commands.trigger(TerrainSculpt { up: !is_shift, idx });
                last_mouse.idx = idx;
            }
        }
    }
}

fn terrain_sculpt(
    trigger: Trigger<TerrainSculpt>,
    mesh_query: Query<(Entity, &Mesh3d), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let (e, mesh_handle) = mesh_query.get_single().expect("Query not successful");
    let mesh = meshes.get_mut(mesh_handle).unwrap();
    let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let v = trigger.event().idx;
    let up = trigger.event().up;

    let amount = STONE_RADIUS * 0.1 * if up { 1.0 } else { -1.0 };
    let r1 = (SUBS + 2) as usize;

    vert_pos[v][1] = (vert_pos[v][1] + amount).max(0.0);
    if v > 0 {
        vert_pos[v-1][1] = (vert_pos[v-1][1] + amount / 2.0).max(0.0);
    }
    vert_pos[v+1][1] = (vert_pos[v+1][1] + amount / 2.0).max(0.0);
    if v > r1 {
        vert_pos[v-r1][1] = (vert_pos[v-r1][1]  + amount / 2.0).max(0.0);
    }
    vert_pos[v+r1][1] = (vert_pos[v+r1][1] + amount / 2.0).max(0.0);

    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}


fn cam_track(
    time: Res<Time>,
    stone: Query<&Transform, (With<Stone>, Without<TrackingCamera>)>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
){
    let dt = time.delta_secs();
    let stone_pos = stone.single();

    let mut camera = camera.single_mut();

    let dist = stone_pos.translation.distance(camera.translation);
    if stone_pos.translation.z > -5.0 && dist > STONE_RADIUS * 3.0  {

        //let move_amount = camera.forward();
        //camera.translation += move_amount * 5.0 * dt;
        camera.look_at(
            stone_pos.translation +
                Vec3::new(0.0, STONE_RADIUS * 2.0, STONE_RADIUS * 3.0)
                , Dir3::Y);
    } else {

        let move_amount = STONE_RADIUS * 40.0; //camera.forward() * 10.0;
        camera.translation.y = STONE_RADIUS * 9.0;
        camera.translation.z = stone_pos.translation.z - move_amount ;//* 10.0 * dt;
        camera.look_at(stone_pos.translation
                       + Vec3::new(0.0, 0.0, -STONE_RADIUS * 20.0), Dir3::Y);// + Vec3::new(0.0, 1.0, 0.0), Dir3::Y);
    }


}

fn stone_shoot(
    input: Res<ButtonInput<KeyCode>>,
    mut stone: Query<(&mut Transform, &mut LinearVelocity), With<Stone>>,
    mut spotty: Query<&mut Transform, (With<Spotty>, Without<Stone>)>,
    mesh_query: Query<(Entity, &Mesh3d), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
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

    if input.just_pressed(KeyCode::Space) {
        let (e, mesh_handle) = mesh_query.get_single().expect("Query not successful");
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        toggle_texture(mesh);
        commands.entity(e).remove::<Collider>();
        commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));
    }

}

fn bob(
    time: Res<Time>,
    mut t: Query<(&mut Transform, &mut BobX)>,
){
    let dt = time.delta_secs();

    for (mut t, mut b) in t.iter_mut() {
    let move_amount = t.right() * b.dt.sin() * 10.0;
        t.translation += move_amount * dt;
        b.dt += dt * 4.0;
    }
}

#[rustfmt::skip]
fn create_plane_mesh() -> Mesh {
    let w = 5.0 / 2.0;
    let h = 50.0 / 2.0;

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-w, 0.0, -h],
            [w, 0.0, -h],
            [w, 0.0, h],
            [-w, 0.0, h],

            [w+w*2.0, 0.0, -h],
            [w+w*2.0, 0.0, h],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
            [0.0, 1.0], [0.0, 0.0],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],

            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        0,3,1, 1,3,2, // triangles making up the top (+y) facing side.
        1,2,4, 4,2,5
    ]))

}

fn toggle_texture(mesh_to_change: &mut Mesh) {
    let uv_attribute = mesh_to_change.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };


    let mut rng = rand::thread_rng();
    let v = rng.gen_range(0..(SUBS*SUBS)) as usize;
    let r1 = (SUBS + 2) as usize;

    let amount = STONE_RADIUS * 1.0;

    vert_pos[v][1] -= amount;
    if v > 0 { vert_pos[v-1][1] -= amount / 2.0; }
    vert_pos[v+1][1] -= amount / 2.0;
    if v > r1 { vert_pos[v-r1][1] -= amount / 2.0; }
    vert_pos[v+r1][1] -= amount / 2.0;

    let v = rng.gen_range(0..(SUBS*SUBS)) as usize;
    let r1 = (SUBS + 2) as usize;

    vert_pos[v][1] += amount;
    if v > 0 { vert_pos[v-1][1] += amount / 2.0; }
    vert_pos[v+1][1] += amount / 2.0;
    if v > r1 { vert_pos[v-r1][1] += amount / 2.0; }
    vert_pos[v+r1][1] += amount / 2.0;


    // let mut idx = 0;
    //for pos in vert_pos.iter_mut() {
//        pos[1] += rng.gen_range(-0.5..0.5);
    //    idx += 1;
    //}
}


fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}
