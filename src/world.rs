use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;

#[derive(Component, Default)]
pub struct Tile(pub Passable);

pub enum Passable {
    Passable,
    Blocking,
}

impl Default for Passable {
    fn default() -> Self {
        Passable::Passable
    }
}

// World is 50x40 tiles (800x600 configured window size).

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_world_tiles.in_schedule(OnEnter(GameState::Playing)));
    }
}

fn spawn_world_tiles(mut commands: Commands, textures: Res<TextureAssets>) {
    for y in [0.0, 32.0] {
        for x in (-17)..(17) {
            let x_coord = x as f32 * 16.0;
            spawn_floor(&mut commands, &textures, Vec3::new(x_coord, y, 1.0));
        }
    }

    for x in [-17., 16.] {
        spawn_floor(&mut commands, &textures, Vec3::new(x * 16., 16., 1.0));
    }

    for x in (-16)..(16) {
        let x_coord = x as f32 * 16.0;
        commands.spawn((
            SpriteBundle {
                texture: textures.bar.clone(),
                transform: Transform::from_translation(Vec3::new(x_coord, 16., 2.0)),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomLeft,
                    ..default()
                },
                ..default()
            },
            Tile(Passable::Blocking),
        ));
    }
}

fn spawn_floor(commands: &mut Commands, textures: &Res<TextureAssets>, translation: Vec3) {
    commands.spawn((
        SpriteBundle {
            texture: textures.floor1.clone(),
            transform: Transform::from_translation(translation),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(16.0)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            ..default()
        },
        Tile::default(),
    ));
}

pub trait AsTile {
    fn as_tile(&self) -> Vec3;
}

impl AsTile for Vec3 {
    fn as_tile(&self) -> Vec3 {
        Vec3::new(self.x / 16., self.y / 16., self.z).floor()
    }
}
