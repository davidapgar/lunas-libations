use crate::GameState;
use bevy::prelude::*;

pub struct AnimatePlugin;

impl Plugin for AnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprites.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Clone)]
pub struct Animation {
    frames: Vec<usize>,
    timing: f32,
    looping: bool,
}

impl Animation {
    pub fn new(frames: &[usize], timing: f32, looping: bool) -> Self {
        Animation {
            frames: frames.to_vec(),
            timing,
            looping,
        }
    }
}

struct RunningAnimation {
    animation: Animation,
    frame_idx: usize,
    timer: Timer,
}

impl RunningAnimation {
    fn new(animation: &Animation) -> Self {
        let timer = Timer::from_seconds(animation.timing, TimerMode::Once);
        RunningAnimation {
            animation: animation.clone(),
            frame_idx: 0,
            timer,
        }
    }
}

#[derive(Component)]
pub struct AnimationComponent {
    running: Option<RunningAnimation>,
}

impl Default for AnimationComponent {
    fn default() -> Self {
        AnimationComponent { running: None }
    }
}

impl AnimationComponent {
    pub fn start_animation(&mut self, animation: &Animation) {
        self.running = Some(RunningAnimation::new(animation));
    }

    pub fn stop_animation(&mut self) {
        self.running = None;
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut AnimationComponent)>,
) {
    for (mut sprite, mut animation_component) in &mut query {
        let Some(running) = &mut animation_component.running else {
            continue;
        };

        sprite.index = running.animation.frames[running.frame_idx];
        running.timer.tick(time.delta());
        if running.timer.just_finished() {
            running.frame_idx += 1;
            if running.frame_idx >= running.animation.frames.len() {
                running.frame_idx = 0;
            }
            running.timer.reset();
        }
    }
}
