use bevy::prelude::*;

use crate::timey::Timey;
use crate::game::{despawn_screen, GameState};

#[derive(Component)]
struct OnSplashScreen;

#[derive(Component)]
pub struct SplashTimer;

pub fn splash_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Splash), splash_setup)
        .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
        .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
}

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

    commands.spawn((
        Timey::new(15.0),
        SplashTimer,
        OnSplashScreen
    ));

}


pub fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut timers: Query<&mut Timey, With<SplashTimer>>,
) {
    for mut timer in timers.iter_mut() {
        if timer.tick(time.delta()) {
            game_state.set(GameState::InGame);
        }
        if timer.elapsed().as_secs() > 1 && buttons.just_pressed(MouseButton::Left) {
            game_state.set(GameState::InGame);
        }
    }
}
