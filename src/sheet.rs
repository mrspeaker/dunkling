use avian3d::prelude::*;
use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
//    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::{
        mesh::{Indices, VertexAttributeValues, PlaneMeshBuilder},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    }
};
use perlin_noise::PerlinNoise;
use std::f32::consts::*;

use rand::prelude::*;

use crate::constants::{
    CELL_LENGTH, CELL_WIDTH, MAX_TERRAIN_HEIGHT, SHEET_LENGTH, SHEET_PRE_AREA, SHEET_RATIO, SHEET_WIDTH, STONE_RADIUS
};

use crate::game::{GameState, OnGameScreen};

#[derive(Component)]
pub struct Sheet;

#[derive(Debug, Event)]
pub struct TerrainSculpt {
    pub up: bool,
    pub idx: usize,
    pub p1: Vec3,
    pub p2: Vec3,
}

struct SpawnTerrain {
    pos: IVec2,
    bumpiness: f32,
}

impl Command for SpawnTerrain {
    fn apply(self, world: &mut World) {

        let mut hm2 = HeightMap::new(SHEET_WIDTH, SHEET_LENGTH, CELL_WIDTH, CELL_LENGTH);

        let mut plane2 = build_plane(Plane3d::default()
                                     .mesh()
                                     .size(SHEET_WIDTH, SHEET_LENGTH)
        );
        terraform(&mut plane2, &mut hm2, self.bumpiness);

        let mesh = world
            .get_resource_mut::<Assets<Mesh>>()
            .expect("mesh Assets to be exist")
            .add(plane2);

        let mat = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("StandardMaterial Assets to exist")
            .add(Color::WHITE);

        let mut ent = world.spawn((
            OnGameScreen,
            Mesh3d(mesh),
            RigidBody::Static,
            Friction::new(10.0),
            ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
            CollisionMargin(0.05),
            MeshMaterial3d(mat),
            Transform::from_xyz(
                self.pos.x as f32 * SHEET_WIDTH,
                0.,
                self.pos.y as f32 * SHEET_LENGTH,
            ),
            //Wireframe,
        ));
        if self.pos.y != 5 {
            ent.insert(Sheet);
        }

        world.spawn((
            OnGameScreen,
           // Mesh3d(meshes.add(Cuboid::default())),
            RigidBody::Static,
            ColliderConstructor::Cuboid {
                x_length: SHEET_WIDTH,
                y_length: 50.0,
                z_length: SHEET_LENGTH
            },
            Transform::from_xyz(
                self.pos.x as f32 * SHEET_WIDTH,
                -25.0,
                self.pos.y as f32 * SHEET_LENGTH,
            ),
        ));

    }
}

#[derive(Resource, Clone, Debug)]
pub struct HeightMap {
    pub w: f32,
    pub h: f32,
    pub cell_w: usize,
    pub cell_h: usize,
    rat_w: f32,
    rat_h: f32,
    pub map: Vec<Vec<f32>>,
}

impl HeightMap {
    pub fn new(w: f32, h: f32, cell_w: usize, cell_h: usize) -> Self {
        let rat_w = w / cell_w as f32;
        let rat_h = h / cell_h as f32;
        let map = vec![vec![0.0; cell_w]; cell_h];
        HeightMap {
            w,
            h,
            cell_w,
            cell_h,
            rat_w,
            rat_h,
            map,
        }
    }

    /// Given a SHEET x and y coordinate,
    /// return the corresponding CELL position.
    pub fn get_cell_from_pos(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        //Calculate the cell coordinates
        let cell_x = (x / self.rat_w).floor() as usize;
        let cell_y = (y / self.rat_h).floor() as usize;

        // Check if cell position is out of map bounds
        if cell_x >= self.cell_w || cell_y >= self.cell_h {
            dbg!(cell_x, self.cell_w, cell_y, self.cell_h);
            None // out of bound
        } else {
            Some((cell_x, cell_y))
        }
    }
    pub fn pos_to_height(&self, x:f32, y:f32) -> Option<f32> {
        let cell_pos = self.get_cell_from_pos(x, y);
        match cell_pos {
            Some((x, y)) => Some(self.map[y][x]),
            _ => None
        }
    }

    // Return a random cell x/y from the height map
    pub fn get_random_cell(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        let cell_x = rng.gen_range(0..self.cell_w);
        let cell_y = rng.gen_range(0..self.cell_h);
        (cell_x, cell_y)
    }

    /// Bilinear interpolation to get height
    fn get_height_at_pos(&self, x: f32, y:f32) -> Option<f32> {
        // A-----B
        // |--x,y|
        // |  |  |
        // C-----D
        //
        // p1 = Ax + x * (Bx - Ax)
        // p2 = c + x * (d - c)
        // h = p1 + y * (p2 - p1)

        /*
    v1 = original_img[x_floor, y_floor, :]
    v2 = original_img[x_ceil, y_floor, :]
    v3 = original_img[x_floor, y_ceil, :]
        v4 = original_img[x_ceil, y_ceil, :]
        #Estimate the pixel value q using pixel values of neighbours
    q1 = v1 * (x_ceil - x) + v2 * (x - x_floor)
    q2 = v3 * (x_ceil - x) + v4 * (x - x_floor)
    q = q1 * (y_ceil - y) + q2 * (y - y_floor)
    resized[i,j,:] = q
        return resizde
         */

        let xo = x % self.rat_w;
        let yo = x / self.rat_h;

        // 1. Find the cells that surround (x, y)
        let Some((cell_x, cell_y)) = self.get_cell_from_pos(x, y) else {
            return None;
        };
        let a = self.map[cell_y][cell_x];
        let b = self.map[cell_y][cell_x + 1];
        let c = self.map[cell_y + 1][cell_x];
        let d = self.map[cell_y + 1][cell_x + 1];

//        let q1 = a * (

        // 2. interpolate.
        None
    }

}

pub struct SheetPlugin;

impl Plugin for SheetPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(WireframePlugin);
        /*app.insert_resource(WireframeConfig {
            global: false,
            default_color: Color::linear_rgb(0.1,0.1, 0.),
        });*/
        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_observer(terrain_sculpt);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut height_map = HeightMap::new(SHEET_WIDTH, SHEET_LENGTH, CELL_WIDTH, CELL_LENGTH);

    let mut plane = build_plane(Plane3d::default()
        .mesh()
        .size(SHEET_WIDTH, SHEET_LENGTH)
    );
    terraform(&mut plane, &mut height_map, 0.1);

    commands.insert_resource(height_map);

    let texture_handle = asset_server.load_with_settings(
        "textures/Ground037_2K-JPG_Color.jpg",
        |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        });
    // this material renders the texture normally
    let uv_x = 2.0;
    let uv_y = uv_x * SHEET_RATIO;
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.8,
        uv_transform: Affine2::from_scale(Vec2::new(uv_x, uv_y)),
        ..default()
    });

/*    let mat = StandardMaterial {
        base_color: Color::linear_rgb(0.36,0.7, 0.219),
        perceptual_roughness: 0.5,
        ..default()
};*/
    let mat = Color::WHITE;

    commands.spawn((
        OnGameScreen,
        Mesh3d(meshes.add(plane)),
        RigidBody::Static,
        Friction::new(10.0),
        ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
        CollisionMargin(0.05),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(
            0.0,
            -2.0,
            -SHEET_LENGTH / 2.0 + (SHEET_PRE_AREA / 2.0) )
            .with_rotation(Quat::from_rotation_x(0.2)),
        //Wireframe,
        Sheet
    ));

    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 0), bumpiness: 0.2 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(-1, 0), bumpiness: 1.0 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 1), bumpiness: 0.3 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 2), bumpiness: 0.8 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(1, 2), bumpiness: 1.8 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 3), bumpiness: 1.0 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 4), bumpiness: 1.0 });
    commands.queue(SpawnTerrain{ pos: IVec2::new(0, 5), bumpiness: 0.0 });

    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let pos = Vec3::new(
            rng.gen_range((-SHEET_WIDTH / 4.0)..(SHEET_WIDTH/ 4.0)),
            0.0,
            rng.gen_range((-SHEET_LENGTH)..0.0)
        );

        commands
            .spawn((
                Name::new("Tree"),
                OnGameScreen,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/tree.glb"))),
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.7),
                Transform::from_xyz(pos.x, pos.y, pos.z)));
    }

}

fn set_height(hm_x: usize, hm_y: usize, value: f32, height_map: &mut HeightMap, verts: &mut Vec<[f32; 3]>) {
    let map = &mut height_map.map;
    (*map)[hm_y][hm_x] = value;
    verts[hm_y * CELL_WIDTH + hm_x][1] = value;
}

fn add_height(hm_x: usize, hm_y: usize, value: f32, height_map: &mut HeightMap, verts: &mut Vec<[f32; 3]>) {
    let map = &mut height_map.map;
    if hm_x >= height_map.cell_w ||
        hm_y >= height_map.cell_h {
           return;
        }
    let cur = (*map)[hm_y][hm_x];
    let next = (cur + value).max(0.0);
    (*map)[hm_y][hm_x] = next;
    verts[hm_y * CELL_WIDTH + hm_x][1] = next;
}

pub fn terrain_sculpt(
    trigger: Trigger<TerrainSculpt>,
    mesh_query: Query<(Entity, &Mesh3d, &Transform), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut height_map: ResMut<HeightMap>,
    mut commands: Commands,

) {
    let Ok((e, mesh_handle, t)) = mesh_query.get(trigger.entity()) else {
        return;
    };

    let mesh = meshes.get_mut(mesh_handle).unwrap();
    let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let ev = trigger.event();
    let up = ev.up;
    let v = ev.idx;
    // Get sheet position from world position
    let p1 = ev.p1 - t.translation + Vec3::new(SHEET_WIDTH * 0.5, 0.0, SHEET_LENGTH * 0.5);
    let p2 = ev.p2 - t.translation + Vec3::new(SHEET_WIDTH * 0.5, 0.0, SHEET_LENGTH * 0.5);

    let Some((c1x, c1y)) = height_map.get_cell_from_pos(p1.x, p1.z) else { return; };
    let Some((c2x, c2y)) = height_map.get_cell_from_pos(p2.x, p2.z) else { return; };


    let h = STONE_RADIUS * 0.1 * if up { 1.0 } else { -1.0 };

    add_height(c1x, c1y, h, &mut *height_map, &mut *vert_pos);
    let ns = get_neighbours(c1x, c1y);
    for (x, y) in ns {
        add_height(x, y, h * 0.8, &mut *height_map, &mut *vert_pos);
        let ns = get_neighbours(x, y);
        for (x, y) in ns {
            add_height(x, y, h * 0.3, &mut *height_map, &mut *vert_pos);
        }
    }
    if c1x != c2x || c1y != c2y {
        add_height(c2x, c2y, h, &mut *height_map, &mut *vert_pos);
        let ns = get_neighbours(c2x, c2y);
        for (x, y) in ns {
            add_height(x, y, h * 0.8, &mut *height_map, &mut *vert_pos);
            let ns = get_neighbours(x, y);
            for (x, y) in ns {
                add_height(x, y, h * 0.3, &mut *height_map, &mut *vert_pos);
            }

        }
    }

    mesh.compute_normals();
    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}

fn get_neighbours(x: usize, z: usize) -> Vec<(usize, usize)> {
    let w = CELL_WIDTH;
    let l = CELL_LENGTH;
    let mut ns: Vec<(usize,usize)> = vec![];

    if x > 0 {
        ns.push((x - 1, z)); // left
    }
    if x < w {
        ns.push((x + 1, z)); // right
    }
    if z > 0 {
        ns.push((x, z - 1)); // back
    }
    if z < l {
        ns.push((x, z + 1))
    }
    return ns;
}

fn terraform(mesh: &mut Mesh, map: &mut HeightMap, ratio: f32) {
    let vert = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = vert else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };


    /*
    let mut rng = rand::thread_rng();
        for _ in 0..1000 {
        let x = rng.gen_range(0..CELL_WIDTH-1);
        let y = rng.gen_range(0..CELL_LENGTH-1);
        let h = rng.gen_range(1.0..STONE_RADIUS*0.8);
        set_height(x, y, h, map, vert_pos);

        let ns = get_neighbours(x, y);
        for (x, y) in ns {
            set_height(x, y, h /2.0, map, vert_pos);
        }
    }*/

    let perlin = PerlinNoise::new();

    let mut max = -9999.0;
    let mut min = 9999.0;
    let terrain_height = MAX_TERRAIN_HEIGHT * ratio;
    for y in 0..map.cell_h {
        for x in 0..map.cell_w {
            let mut h = perlin.get3d([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
            h = h.max(0.5) - 0.5;
            set_height(x, y, h as f32 * terrain_height, map, vert_pos);

            if h < min { min = h; };
            if h > max { max = h; };
        }
    }
    dbg!(min, max);

    let cols: Vec<[f32; 4]> = vert_pos
        .iter()
        .map(|[_, h, _]| {
            let h = *h;// / terrain_height;
            if h > 7.0 {
                Color::WHITE.to_linear().to_f32_array()
            } else if h > 1.0{
                Color::srgb(0.4, 0.4, 0.1)
                    .to_linear()
                    .to_f32_array()
            } else {
                Color::linear_rgb(0.36,0.7, 0.219)
                //Color::srgb(0.1, 0.5, 0.0)
                    .to_linear()
                    .to_f32_array()
            }
        })
        .collect();

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        cols,
    );

    mesh.compute_normals();

}

fn build_plane(mb: PlaneMeshBuilder) -> Mesh {
    let size = mb.plane.half_size * 2.0;
    let z_vertex_count = CELL_LENGTH as u32;
    let x_vertex_count = CELL_WIDTH as u32;
    let num_vertices = (z_vertex_count * x_vertex_count) as usize;
    let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

    let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

    let rotation = Quat::from_rotation_arc(Vec3::Y, *mb.plane.normal);

    for z in 0..z_vertex_count {
        for x in 0..x_vertex_count {
            let tx = x as f32 / (x_vertex_count - 1) as f32;
            let tz = z as f32 / (z_vertex_count - 1) as f32;
            let pos = rotation * Vec3::new((-0.5 + tx) * size.x, 0.0, (-0.5 + tz) * size.y);
            positions.push(pos);
            normals.push(mb.plane.normal.to_array());
            uvs.push([tx, tz]);
        }
    }

    for z in 0..z_vertex_count - 1 {
        for x in 0..x_vertex_count - 1 {
            let quad = z * x_vertex_count + x;
            indices.push(quad + x_vertex_count + 1);
            indices.push(quad + 1);
            indices.push(quad + x_vertex_count);
            indices.push(quad);
            indices.push(quad + x_vertex_count);
            indices.push(quad + 1);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
