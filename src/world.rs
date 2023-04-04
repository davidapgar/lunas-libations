use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 16.;
pub const SCREEN_SIZE: Vec2 = Vec2::new(800., 600.);
pub const WORLD_SIZE: IVec2 = IVec2::new(
    (SCREEN_SIZE.x / TILE_SIZE) as i32,
    (SCREEN_SIZE.y / TILE_SIZE) as i32,
);

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
    for y in 18..21 {
        for x in 0..50 {
            spawn_floor(&mut commands, &textures, IVec2::new(x, y));
        }
    }

    for x in 10..40 {
        spawn_tile(
            &mut commands,
            IVec2::new(x, 21),
            textures.bar.clone(),
            Passable::Blocking,
        );
    }

    for y in 22..24 {
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
    let translation = tile_location.as_tile().to_camera_space();
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

pub trait TileSpace {
    fn to_tile_space(&self) -> Vec3;
    fn to_camera_space(&self) -> Vec3;
}

impl TileSpace for Vec3 {
    fn to_tile_space(&self) -> Vec3 {
        Vec3::new(
            self.x + (SCREEN_SIZE.x / 2.0),
            (SCREEN_SIZE.y / 2.0) - self.y,
            self.z,
        )
    }

    fn to_camera_space(&self) -> Vec3 {
        Vec3::new(
            self.x - (SCREEN_SIZE.x / 2.0),
            (SCREEN_SIZE.y / 2.0) - self.y,
            self.z,
        )
    }
}

pub trait AsTile {
    fn as_tile(&self) -> Vec3;
}

pub trait ToTileIndex {
    /// Convert from camera space to tile index.
    fn to_tile_index(&self) -> IVec2;
}
// (20 - y) * 16 = tile_y (320 -> -320)
// (20 - y) = tile_y / 16
// 20 - (tile_y / 16) = y

impl ToTileIndex for Vec3 {
    // Convert a vec3 from camera space to tile space to tile index
    fn to_tile_index(&self) -> IVec2 {
        (self.to_tile_space() / TILE_SIZE).as_ivec3().truncate()
    }
}

impl AsTile for IVec2 {
    // Convert from tile index into coordinates in tile space
    fn as_tile(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * TILE_SIZE,
            self.y as f32 * TILE_SIZE,
            self.y as f32,
        )
    }
}
