use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;

#[derive(Component)]
pub struct Tile;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_world_tiles.in_schedule(OnEnter(GameState::Playing)));
    }
}

fn spawn_world_tiles(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: textures.floor1.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.0)),
            ..default()
        },
        Tile,
    ));
}
