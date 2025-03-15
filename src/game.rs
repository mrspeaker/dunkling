use avian3d::prelude::*;
use bevy::{
    prelude::*,
    color::palettes::css::*,
    scene::SceneInstanceReady,
};
use bevy_hanabi::prelude::*;

use std::f32::consts::*;

use crate::constants::{
    SHEET_PRE_AREA,
    CHUNK_SIZE,
    STONE_RADIUS,
    STONE_STOP_VEL,
    TARGET_CENTRE,
    STONE_ANGULAR_DAMPENING_INC_START_AT,
    STONE_ANGULAR_DAMPENING_INC_AMOUNT,
    STONE_HURL_AIM_ANGLE_MULTIPLIER, STONE_MAX_VEL, SHOW_DBG
};

use crate::camera::camera_plugin;
use crate::player::{player_plugin, HurlStone};
use crate::sheet::{sheet_plugin, StoneInHole};
use crate::splash::splash_plugin;
use crate::stone::{Stone, stone_plugin};
use crate::timey::Timey;
use crate::townsfolk::townsfolk_plugin;

pub struct GamePlugin;

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
    EndGame,
    StoneStopped,
}

#[derive(Component)]
struct StoneStoppedTimer;

#[derive(Component)]
pub struct AimTooSlowTimer;

#[derive(Component)]
pub struct TextDistance;
#[derive(Component)]
pub struct TextPower;

#[derive(Component)]
pub struct BigThor;

#[derive(Resource)]
pub struct HiScore {
    pub score: f32,
    pub fault: bool
}

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Terrain,
    Stone,
    Townsfolk,
    Sensors
}

fn distance_to_target(pos: Vec3) -> f32 {
    pos.distance(TARGET_CENTRE)
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Library plugins
        app.add_plugins((
            MeshPickingPlugin,
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            HanabiPlugin));

        // Game plugins
        app.add_plugins((
            camera_plugin,
            player_plugin,
            sheet_plugin,
            splash_plugin,
            stone_plugin,
            townsfolk_plugin
        ));

        app.insert_resource(HiScore { score: 2000.0, fault: false });
        app.init_state::<GameState>()
            .add_sub_state::<GamePhase>();

        // Systems
        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_systems(OnEnter(GamePhase::Sculpting), fire_stone);
        app.add_systems(OnEnter(GamePhase::StoneStopped), on_stone_stopped_enter);
        app.add_systems(
            Update,
            (
                check_keys,
                aim_countdown.run_if(in_state(GamePhase::Aiming)),
                gameover_update.run_if(in_state(GamePhase::EndGame)),
                stone_stopped_update.run_if(in_state(GamePhase::StoneStopped)),
                track_and_dampen_stone.run_if(in_state(GamePhase::Sculpting)),
                text_distance,
                text_power,
            ));
        app.add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);

        // Triggers
        app.add_observer(on_hurl_stone);
        app.add_observer(start_anims_on_load);
        app.add_observer(on_stone_in_hole);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
    mut hi: ResMut<HiScore>
) {
    // Reset cheat check
    hi.fault = false;

    // Thor plane
    let texture_handle = asset_server.load("thor.png");
    let aspect = 2.0;//0.25;
    let quad_width = STONE_RADIUS * 10.0;
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        double_sided: true,
        cull_mode: None, //Some(Face::Back)
        alpha_mode: bevy::prelude::AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(quad_width, quad_width * aspect))),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(200.0, 1.0, 2000.0)
            .with_rotation(Quat::from_euler(
                // YXZ = "yaw"/"pitch"/"roll"
                EulerRot::YXZ,
                (180.0_f32).to_radians(),
                (0.0_f32).to_radians(),
                (0.0_f32).to_radians(),
            )),
        OnGameScreen
    ));


    // Spot light
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            range: 3000.0,
            radius: 3000.0,
            color: BLUE.into(),
            shadows_enabled: true,
            ..default()
        },
        Spotty,
        OnGameScreen
    ));


    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 1.0),
        brightness: 500.0,
    });

    // Sun light
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::CLEAR_SUNRISE,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.0 -0.4),
            ..default()
        },
        OnGameScreen
    ));

    // Auto fire when aiming (take too long)
    commands.spawn((
        Timey::new(25.0),
        AimTooSlowTimer,
        OnGameScreen
    ));

    if SHOW_DBG {
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
            OnGameScreen,
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
            OnGameScreen,
        ))
            .with_child( Text::new("Power:"))
            .with_child((
                Text::new(""),
                TextPower
            ));

    }

    const BIG_THOR_PATH: &str = "models/mano.glb";
    commands
        .spawn((
            Name::new("BigThor"),
            BigThor,
            OnGameScreen,
            SceneRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset(BIG_THOR_PATH))),
            Transform::from_xyz(0.0, 92.0, -CHUNK_SIZE + SHEET_PRE_AREA)
                .with_scale(Vec3::splat(25.0))
        ));

    // Particles...
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
    gradient.add_key(0.1, Vec4::new(0.5, 0.5, 0.0, 1.0));
    gradient.add_key(0.4, Vec4::new(0.5, 0.0, 0.0, 1.0));
    gradient.add_key(1.0, Vec4::splat(0.0));
    let mut module = Module::default();
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(10.),
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };
    let lifetime = module.lit(5.);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let accel = module.lit(Vec3::new(0., -20., 0.));
    let update_accel = AccelModifier::new(accel);
    let effect = EffectAsset::new(
        32768,
        Spawner::rate(50.0.into()),
        module
    )
        .with_name("MyEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        .render(ColorOverLifetimeModifier { gradient });

    let effect_handle = effects.add(effect);
    commands
        .spawn((
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_handle),
                transform: Transform::from_xyz(0.0, 50.0, 500.0),
                ..Default::default()
            },
            OnGameScreen));


}


fn fire_stone(
    stone: Query<Entity, With<Stone>>,
    mut commands: Commands
) {
    let Ok(e) = stone.get_single() else { return; };
    commands.entity(e).insert(RigidBody::Dynamic);
}

pub fn track_and_dampen_stone(
    mut stone: Query<(&Transform, &LinearVelocity, &mut AngularDamping), With<Stone>>,
    mut phase: ResMut<NextState<GamePhase>>,
    time: Res<Time>
){
    let Ok((stone_pos, vel, mut damp)) = stone.get_single_mut() else { return; };

    // Slow down the stone faster when starting to go slow
    if vel.length() < STONE_ANGULAR_DAMPENING_INC_START_AT &&
        stone_pos.translation.y < STONE_ANGULAR_DAMPENING_INC_START_AT {
            damp.0 += STONE_ANGULAR_DAMPENING_INC_AMOUNT * time.delta_secs();
        }

    // Finish when too slow.
    if vel.length() < STONE_STOP_VEL {
        phase.set(GamePhase::StoneStopped);
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn aim_countdown(
    mut game_state: ResMut<NextState<GamePhase>>,
    time: Res<Time>,
    mut timers: Query<&mut Timey, With<AimTooSlowTimer>>,
) {
    for mut timer in timers.iter_mut() {
        if timer.tick(time.delta()) {
            game_state.set(GamePhase::Sculpting);
        }
    }
}

fn on_stone_in_hole (
    _trigger: Trigger<StoneInHole>,
    mut phase: ResMut<NextState<GamePhase>>,
) {
    phase.set(GamePhase::StoneStopped);
}

fn on_hurl_stone(
    trigger: Trigger<HurlStone>,
    mut phase: ResMut<NextState<GamePhase>>,
    mut stone: Query<&mut LinearVelocity, With<Stone>>
) {
    let Ok(mut vel) = stone.get_single_mut() else { return; };
    vel.x = trigger.event().angle * STONE_HURL_AIM_ANGLE_MULTIPLIER;
    vel.z = trigger.event().power * STONE_MAX_VEL;
    vel.y = -100.0;
    info!("power: {} angle: {}", vel.z, vel.x);
    phase.set(GamePhase::Sculpting);
}


fn text_distance(
    mut txt: Query<&mut Text, With<TextDistance>>,
    stone: Query<&Transform, With<Stone>>
) {
    let Ok(stone_pos) = stone.get_single() else { return; };

    for mut span in txt.iter_mut() {
        let vtxt = distance_to_target(stone_pos.translation);
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
    mut _players: Query<&mut AnimationPlayer> //, With<BigThor>>
) {
    let e = trigger.entity();
    let Ok(thor) = thor.get_single() else { return; };
    if e != thor { return; };

    info!("got bigthor gltf");
    /*    for mut player in players.iter_mut() {
        info!("attempt to pla");
        player.play(1.into()).repeat();
    }*/
}

fn on_stone_stopped_enter(
    mut cmds: Commands,
    stone: Query<(Entity, &Transform), With<Stone>>,
    mut hi: ResMut<HiScore>
) {
    let mut dist: f32 = 999.0;
    if let Ok((e, st)) = stone.get_single() {
        cmds.entity(e).remove::<RigidBody>();
        dist = distance_to_target(st.translation);
    }

    let hiscore = hi.score;
    let is_fault = hi.fault;
    let is_hi = !is_fault && dist < hiscore;
    if is_hi {
        hi.score = dist;
    }

    cmds.spawn((
        TextFont {
            font_size: 48.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
        OnGameScreen,
    ))
        .with_child(
            if is_fault {
                Text::new("CheaT:")
            } else {
                Text::new("OVeR:")
            })
        .with_child((
            Text::new(format!("{dist:.2}")),
        ));


    cmds.spawn((
        TextFont {
            font_size: 48.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(60.0),
            left: Val::Percent(50.0),
            ..default()
        },
        OnGameScreen,
    ))
        .with_child( Text::new("Best:"))
        .with_child(
            if is_hi {
                Text::new("NEW LO SCORE!")
            } else {
                Text::new(format!("{hiscore:.2}"))
            }
        );

    cmds.spawn((
        Timey::new(20.0),
        StoneStoppedTimer,
        OnGameScreen,
    ));

}

fn stone_stopped_update(
    mut state: ResMut<NextState<GamePhase>>,
    mut timers: Query<&mut Timey, With<StoneStoppedTimer>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut timer in timers.iter_mut() {
        // Back to splash if click after 1 second
        if timer.elapsed().as_secs() > 1 {
            let is_mouse = buttons.just_pressed(MouseButton::Left);
            let is_shift = keys.pressed(KeyCode::ShiftLeft);
            if is_mouse && !is_shift {
                state.set(GamePhase::EndGame);
            }
        }

        if timer.tick(time.delta()) {
            state.set(GamePhase::EndGame);
        }
    }
}

fn gameover_update(
    mut state: ResMut<NextState<GameState>>,
) {
    state.set(GameState::Splash);
}

fn check_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>
) {
    if keys.pressed(KeyCode::KeyR) {
        state.set(GameState::Splash);
    }
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
