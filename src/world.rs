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
    for y in 18..20 {
        for x in 0..50 {
            spawn_floor(&mut commands, &textures, IVec2::new(x, y));
        }
    }

    for x in 10..40 {
        spawn_tile(
            &mut commands,
            IVec2::new(x, 20),
            textures.bar.clone(),
            Passable::Blocking,
        );
    }

    for y in 21..22 {
        for x in 0..50 {
            spawn_floor(&mut commands, &textures, IVec2::new(x, y));
        }
    }

    for x in 10..40 {
        spawn_tile(
            &mut commands,
            IVec2::new(x, 17),
            textures.barback.clone(),
            Passable::Blocking,
        );
    }
}

fn spawn_floor(commands: &mut Commands, textures: &Res<TextureAssets>, tile_location: IVec2) {
    spawn_tile(
        commands,
        tile_location,
        textures.floor1.clone(),
        Passable::Passable,
    );
}

// Camera defaults to center of screen being 0.0/0.0
// So tile 0,0 (top left) will be at -25*16/20*16
// tile 50,40 will be at 25*16/-20*16
// tile at 25,20 will be at 0/0
fn spawn_tile(
    commands: &mut Commands,
    tile_location: IVec2,
    texture: Handle<Image>,
    passable: Passable,
) {
    let translation = Vec3::new(
        (tile_location.x - 25) as f32 * 16.,
        (20 - tile_location.y) as f32 * 16.,
        tile_location.y as f32,
    );
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
// (20 - y) * 16 = tile_y (320 -> -320)
// (20 - y) = tile_y / 16
// 20 - (tile_y / 16) = y

impl AsTile for Vec3 {
    fn as_tile(&self) -> Vec3 {
        Vec3::new(self.x / 16., self.y / 16., 20. - (self.y / 16.)).floor()
    }
}
