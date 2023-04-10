use crate::actions::{set_movement_actions, Actions};
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .init_resource::<SoundStates>()
            .add_system(start_audio.in_schedule(OnEnter(GameState::Playing)))
            .add_system(
                control_flying_sound
                    .after(set_movement_actions)
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct SoundStates {
    pub drinking: bool,
    pub chatting: bool,
}

impl Default for SoundStates {
    fn default() -> Self {
        SoundStates {
            drinking: false,
            chatting: false,
        }
    }
}

#[derive(Resource)]
struct BackgroundAudio(Handle<AudioInstance>);
#[derive(Resource)]
struct DrinkingAudio(Handle<AudioInstance>);
#[derive(Resource)]
struct ChattingAudio(Handle<AudioInstance>);

fn start_audio(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.pause();
    let handle = audio
        .play(audio_assets.background.clone())
        .looped()
        .with_volume(0.2)
        .handle();
    commands.insert_resource(BackgroundAudio(handle));

    let handle = audio
        .play(audio_assets.drinking.clone())
        .looped()
        .with_volume(0.3)
        .handle();
    commands.insert_resource(DrinkingAudio(handle));

    let handle = audio
        .play(audio_assets.chatter.clone())
        .looped()
        .with_volume(0.3)
        .handle();
    commands.insert_resource(ChattingAudio(handle));
}

fn control_flying_sound(
    sound_states: Res<SoundStates>,
    audio: Res<DrinkingAudio>,
    background: Res<BackgroundAudio>,
    chatting: Res<ChattingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(instance) = audio_instances.get_mut(&background.0) {
        match instance.state() {
            PlaybackState::Paused { .. } => {
                instance.resume(AudioTween::default());
            }
            _ => {}
        }
    }

    if let Some(instance) = audio_instances.get_mut(&audio.0) {
        match instance.state() {
            PlaybackState::Paused { .. } => {
                if sound_states.drinking {
                    instance.resume(AudioTween::default());
                }
            }
            PlaybackState::Playing { .. } => {
                if !sound_states.drinking {
                    instance.pause(AudioTween::default());
                }
            }
            _ => {}
        }
    }

    if let Some(instance) = audio_instances.get_mut(&chatting.0) {
        match instance.state() {
            PlaybackState::Paused { .. } => {
                if sound_states.chatting {
                    instance.resume(AudioTween::default());
                }
            }
            PlaybackState::Playing { .. } => {
                if !sound_states.chatting {
                    instance.pause(AudioTween::default());
                }
            }
            _ => {}
        }
    }
}
