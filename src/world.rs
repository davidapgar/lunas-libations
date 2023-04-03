use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;

#[derive(Component)]
pub struct Tile;

// World is 50x40 tiles (800x600 configured window size).

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_world_tiles.in_schedule(OnEnter(GameState::Playing)));
    }
}

fn spawn_world_tiles(mut commands: Commands, textures: Res<TextureAssets>) {
    for y in [-16.0, 16.0] {
        for x in (-16)..(16) {
            let x_coord = x as f32 * 16.0;
            commands.spawn((
                SpriteBundle {
                    texture: textures.floor1.clone(),
                    transform: Transform::from_translation(Vec3::new(x_coord, y, 1.0)),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(16.0)),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                    ..default()
                },
                Tile,
            ));
        }
    }

    for x in (-16)..(16) {
        let x_coord = x as f32 * 16.0;
        commands.spawn((
            SpriteBundle {
                texture: textures.bar.clone(),
                transform: Transform::from_translation(Vec3::new(x_coord, 0., 2.0)),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomLeft,
                    ..default()
                },
                ..default()
            },
            Tile,
        ));
    }
}
