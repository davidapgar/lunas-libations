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
    spawn_floor(&mut commands, &textures, Vec3::new(0., 48., 1.0));

    for x in (-16)..(16) {
        let x_coord = x as f32 * 16.0;
        spawn_tile(
            &mut commands,
            Vec3::new(x_coord, 16., 2.0),
            textures.bar.clone(),
            Passable::Blocking,
        );
    }

    for x in (-16)..(16) {
        let x_coord = x as f32 * 16.0;
        spawn_tile(
            &mut commands,
            Vec3::new(x_coord, 48.0, 2.0),
            textures.barback.clone(),
            Passable::Blocking,
        );
    }
}

fn spawn_floor(commands: &mut Commands, textures: &Res<TextureAssets>, translation: Vec3) {
    spawn_tile(
        commands,
        translation,
        textures.floor1.clone(),
        Passable::Passable,
    );
}

fn spawn_tile(
    commands: &mut Commands,
    translation: Vec3,
    texture: Handle<Image>,
    passable: Passable,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(translation),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            ..default()
        },
        Tile(passable),
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
