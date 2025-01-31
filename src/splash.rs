use bevy::prelude::*;

use super::game::{despawn_screen, GameState};

pub fn splash_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Splash), splash_setup)
        .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
        .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
}

#[derive(Component)]
struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
pub struct SplashTimer(pub Timer);

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let thor = asset_server.load("thor.png");
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(thor),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));
        });

    commands.insert_resource(SplashTimer(Timer::from_seconds(15.0, TimerMode::Once)));
}


pub fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::InGame);
    }
    if timer.elapsed_secs() > 0.5 && buttons.just_pressed(MouseButton::Left) {
        game_state.set(GameState::InGame);
    }
}
