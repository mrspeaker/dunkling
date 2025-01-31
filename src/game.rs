use avian3d::prelude::*;

use bevy::{
    prelude::*,
    color::palettes::css::*,
};

use std::f32::consts::*;

use crate::constants::{
    STONE_RADIUS,
    STONE_DAMPENING,
    STONE_MAX_VEL
};
use crate::camera::CameraPlugin;
use crate::player::PlayerPlugin;
use crate::sheet::SheetPlugin;
use crate::splash::{splash_plugin, SplashTimer};
use crate::townsfolk::TownsfolkPlugin;

pub struct GamePlugin;

#[derive(Component)]
pub struct Stone;

#[derive(Component)]
pub struct Spotty;

#[derive(Component)]
pub struct OnGameScreen;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Splash,
    InGame,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::InGame)]
pub enum GamePhase {
    #[default]
    Aiming,
    Sculpting,
    EndGame
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MeshPickingPlugin,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()));
        app.add_plugins(CameraPlugin);
        app.add_plugins(PlayerPlugin);
        app.add_plugins(SheetPlugin);
        app.add_plugins(splash_plugin);
        app.add_plugins(TownsfolkPlugin);

        app.init_state::<GameState>()
            .add_sub_state::<GamePhase>();

        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_systems(Update, countdown.run_if(in_state(GamePhase::Aiming)));
        app.add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);

        app.add_systems(OnEnter(GamePhase::Sculpting), fire_stone);
        app.add_systems(Update, track_stone.run_if(in_state(GamePhase::Sculpting)));
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
        OnGameScreen,
        //RigidBody::Dynamic,
        Collider::sphere(STONE_RADIUS),
        LinearDamping(STONE_DAMPENING),
        MaxLinearSpeed(STONE_MAX_VEL),
        //Friction::new(10.0),
        //CollisionMargin(0.1),
        //Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 80.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, STONE_RADIUS * 4.0, -800.0),
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
        Spotty,
        OnGameScreen
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
            )),
        OnGameScreen
    ));

    // Lights
    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 0.8),
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
            rotation: Quat::from_rotation_y(-PI / 2.0),
            ..default()
        },
        OnGameScreen
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(100.0, 100.0, 50.0),
            rotation: Quat::from_rotation_x(-PI * 1.1),
            ..default()
        },
        OnGameScreen
    ));

    commands.insert_resource(
        SplashTimer(Timer::from_seconds(3.5, TimerMode::Once))
    );

}


fn fire_stone(
    stone: Query<Entity, With<Stone>>,
    mut commands: Commands
) {
    let Ok(e) = stone.get_single() else { return; };
    commands.entity(e).insert(RigidBody::Dynamic);
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn countdown(
    mut game_state: ResMut<NextState<GamePhase>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GamePhase::Sculpting);
    }
}

#[derive(Default, Debug)]
struct StoneStats {
    stopped_dt: f32,
}

fn track_stone(
    stone: Query<&LinearVelocity, With<Stone>>,
    mut phase: ResMut<NextState<GameState>>,
    mut stone_stats: Local<StoneStats>,
){
    let Ok(vel) = stone.get_single() else { return; };
    //dbg!(vel.length());

    if vel.length() < 2.0 {
        phase.set(GameState::Splash);
    }

}
