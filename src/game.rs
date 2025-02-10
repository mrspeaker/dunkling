use avian3d::prelude::*;
use bevy::{
    prelude::*,
    color::palettes::css::*,
    scene::SceneInstanceReady,
};

use std::f32::consts::*;

use crate::constants::{
    SHEET_LENGTH,
    SHEET_PRE_AREA,
    STONE_RADIUS,
    STONE_DAMPENING,
    STONE_MAX_VEL,
    STONE_X,
    STONE_Y,
    STONE_Z,
};
use crate::camera::CameraPlugin;
use crate::player::{PlayerPlugin, HurlStone};
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

#[derive(Component)]
pub struct TextDistance;
#[derive(Component)]
pub struct TextPower;

#[derive(Component)]
pub struct BigThor;


#[derive(Resource, Deref, DerefMut)]
struct GraphHandle(AnimationGraph);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MeshPickingPlugin,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()));
        app.add_plugins(splash_plugin);
        app.add_plugins(CameraPlugin);
        app.add_plugins(PlayerPlugin);
        app.add_plugins(SheetPlugin);
        app.add_plugins(TownsfolkPlugin);
        app.init_state::<GameState>()
            .add_sub_state::<GamePhase>();

        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_systems(Update, countdown.run_if(in_state(GamePhase::Aiming)));
        app.add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);

        app.add_systems(OnEnter(GamePhase::Sculpting), fire_stone);
        app.add_systems(
            Update,
            (
                track_stone.run_if(in_state(GamePhase::Sculpting)),
                text_distance,
                text_power
            ));

        app.add_observer(on_hurl_stone);
        app.add_observer(start_anims_on_load);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/stone076.jpg");
    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.8,
        ..default()
    });

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
        LinearVelocity(Vec3::new(0.0, 0.0, 160.0)),//160.0)),
        AngularVelocity(Vec3::new( 10.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(material_handle),//materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(STONE_X, STONE_Y, STONE_Z),
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
    let aspect = 2.0;//0.25;
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
        color: Color::linear_rgb(1.0,1.0, 1.0),
        brightness: 200.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::CLEAR_SUNRISE,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 1.3),
            ..default()
        },
        OnGameScreen
    ));

    commands.insert_resource(
        SplashTimer(Timer::from_seconds(15.0, TimerMode::Once))
    );

    commands.spawn((
        TextFont {
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ))
        .with_child( Text::new("Distance:"))
        .with_child((
            Text::new(""),
            TextDistance
        ));

    commands.spawn((
        TextFont {
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(5.0),
            ..default()
        },
    ))
        .with_child( Text::new("Power:"))
        .with_child((
            Text::new(""),
            TextPower
        ));

    const BIG_THOR_PATH: &str = "models/mano.glb";
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(BIG_THOR_PATH)),
    ]);
    dbg!(graph.clone(), node_indices);
    graphs.add(graph);


    commands
        .spawn((
            Name::new("BigThor"),
            BigThor,
            OnGameScreen,
            SceneRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset(BIG_THOR_PATH))),
            Transform::from_xyz(0.0, 92.0, -SHEET_LENGTH + SHEET_PRE_AREA)
                .with_scale(Vec3::splat(25.0))
        ));


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
    mut state: ResMut<NextState<GameState>>,
    mut stone_stats: Local<StoneStats>,
){
    let Ok(vel) = stone.get_single() else { return; };
    //dbg!(vel.length());

    if vel.length() < 2.0 {
        state.set(GameState::Splash);
    }

}

fn on_hurl_stone(
    trigger: Trigger<HurlStone>,
    mut phase: ResMut<NextState<GamePhase>>,
    mut stone: Query<&mut LinearVelocity, With<Stone>>
) {
    let Ok(mut vel) = stone.get_single_mut() else { return; };
    vel.z = trigger.event().power * 100.0;
    vel.y = -100.0;
    info!("power: {}", vel.z);
    phase.set(GamePhase::Sculpting);
}


fn text_distance(
    mut txt: Query<&mut Text, With<TextDistance>>,
    stone: Query<&Transform, With<Stone>>
) {
    let Ok(t) = stone.get_single() else { return; };

    for mut span in txt.iter_mut() {
        let vtxt = t.translation.z;//vel.length();
        span.0 = format!("{vtxt:.2}");
    }
}
fn text_power(
    mut txt: Query<&mut Text, With<TextPower>>,
    stone: Query<&LinearVelocity, With<Stone>>
) {
    let Ok(vel) = stone.get_single() else { return; };

    for mut span in txt.iter_mut() {
        let vtxt = vel.length();
        span.0 = format!("{vtxt:.2}");
    }
}

fn start_anims_on_load(
    trigger: Trigger<SceneInstanceReady>,
    thor: Query<Entity, With<BigThor>>,
    mut players: Query<&mut AnimationPlayer> //, With<BigThor>>
) {
    let e = trigger.entity();
    let Ok(thor) = thor.get_single() else { return; };
    if e != thor { return; };

    info!("got bigthor");

    for mut player in players.iter_mut() {
        info!("attempt to pla");
        player.play(1.into()).repeat();
    }
}
