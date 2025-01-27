use avian3d::prelude::*;

use bevy::{
    prelude::*,
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

#[derive(Component)]
struct TrackingCamera;

#[derive(Component)]
struct Stone;

#[derive(Component)]
struct BobX {
    dt: f32
}

#[derive(Component)]
struct CustomMesh;

const SUBS: u32 = 30;

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
            MeshPickingPlugin            ,
//            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })        .add_systems(Startup, setup)
        .add_systems(Update, (cam_track, stone_shoot, bob, draw_mesh_intersections))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // sheet
    let width = 5.0;
    let length = 50.0;
    let pre_area = 5.0;
    let post_area = 10.0;

    // start line
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(width-0.01, 0.45, 0.2))),
        MeshMaterial3d(materials.add(Color::BLACK)),
    ));

    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_plane_mesh());
    commands.spawn((
        //Mesh3d(cube_mesh_handle),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(length, length).subdivisions(SUBS))),
        RigidBody::Static,
        Friction::new(10.0),
        //Collider::cuboid(width, 0.3, length),
        //ColliderConstructor::ConvexHullFromMesh,
        ColliderConstructor::TrimeshFromMesh,
        CollisionMargin(0.05),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, -length / 2.0 + (pre_area / 2.0) ).with_rotation(Quat::from_rotation_y(PI / 4.001)),
        Wireframe,
        CustomMesh
    ));

    // sheet
/*    commands.spawn((
        RigidBody::Static,
        Friction::new(0.05),
        Collider::cuboid(width, 0.3, length),
        Mesh3d(meshes.add(Cuboid::new(width, 0.3, length))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, -length / 2.0 + (pre_area / 2.0) ),
    ));
*/
    // jump
    commands.spawn((
        RigidBody::Static,
//        Friction::new(100.0),
        Collider::cuboid(width * 0.6, 0.3, 2.0),
        Mesh3d(meshes.add(Cuboid::new(width * 0.6, 0.3, 2.0))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(-width/2.0, 0.0, -10.0)
            .with_rotation(Quat::from_rotation_x(PI / 8.)),
        BobX{ dt: 0.0 }
    ));


    // stone
    let radius = 0.3;
    let height = 0.15;
    let weight = 20.0;
    commands.spawn((
        Stone,
        RigidBody::Dynamic,
        //Collider::cylinder(radius, height),
        Collider::sphere(radius),
        //LinearDamping(0.8),
        //Friction::new(10.0),
        Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
        //Mesh3d(meshes.add(Cylinder::new(radius, height))),
        Mesh3d(meshes.add(Sphere::new(radius))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 10.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Dir3::Y),
        TrackingCamera
    ));
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
    if stone_pos.translation.z > -5.0 && dist >5.0  {

        //let move_amount = camera.forward();
        //camera.translation += move_amount * 5.0 * dt;
        camera.look_at(stone_pos.translation + Vec3::new(0.0, 1.0, 1.0), Dir3::Y);
    } else {

        let move_amount = 25.0;//camera.forward() * 10.0;
        camera.translation.y = 10.0;
    camera.translation.z = stone_pos.translation.z - move_amount ;//* 10.0 * dt;
        camera.look_at(stone_pos.translation + Vec3::new(0.0, 0.3, -10.0), Dir3::Y);// + Vec3::new(0.0, 1.0, 0.0), Dir3::Y);
    }


}

fn stone_shoot(
    input: Res<ButtonInput<KeyCode>>,
    mut stone: Query<(&mut Transform, &mut LinearVelocity), With<Stone>>,
    mesh_query: Query<(Entity, &Mesh3d), With<CustomMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
){
    let (mut stone_pos, mut vel_vec) = stone.single_mut();
    if input.pressed(KeyCode::KeyW) {
        vel_vec.z = -5.0;
    }

    if stone_pos.translation.distance(Vec3::ZERO) > 64.0 {
        stone_pos.translation = Vec3::new(0.0, 0.5, 0.0);
        vel_vec.x = 0.0;
        vel_vec.y = 0.0;
        vel_vec.z = 0.0;
    }

    if input.just_pressed(KeyCode::Space) {
        let (e, mesh_handle) = mesh_query.get_single().expect("Query not successful");
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        toggle_texture(mesh);
        commands.entity(e).remove::<Collider>();
        commands.entity(e).insert(ColliderConstructor::TrimeshFromMesh);
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

    vert_pos[v][1] -= 1.0;
    if v > 0 { vert_pos[v-1][1] -= 0.5; }
    vert_pos[v+1][1] -= 0.5;
    if v > r1 { vert_pos[v-r1][1] -= 0.5; }
    vert_pos[v+r1][1] -= 0.5;


    let v = rng.gen_range(0..(SUBS*SUBS)) as usize;
    let r1 = (SUBS + 2) as usize;

    vert_pos[v][1] += 1.0;
    if v > 0 { vert_pos[v-1][1] += 0.5; }
    vert_pos[v+1][1] += 0.5;
    if v > r1 { vert_pos[v-r1][1] += 0.5; }
    vert_pos[v+r1][1] += 0.5;


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
