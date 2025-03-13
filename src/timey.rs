use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct Timey {
    pub timer: Timer,
}
impl Timey {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once)
        }
    }

    pub fn tick(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta).just_finished()
    }

    pub fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }
}
