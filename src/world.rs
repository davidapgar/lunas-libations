use crate::loading::TextureAssets;
use crate::player::Item;
use crate::tilemap::TileMap;
use crate::GameState;
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 32.;
pub const SCALE: Vec3 = Vec3::new(TILE_SIZE / 16., TILE_SIZE / 16., 1.0);
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
// Actual 25x20'ish, as have scaled all assets to 2x for better visibilty

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_world_tiles.in_schedule(OnEnter(GameState::Playing)))
            .add_system(update_tile_positions.in_set(OnUpdate(GameState::Playing)));
    }
}

fn update_tile_positions(
    tile_map_query: Query<&TileMap>,
    mut transform_query: Query<&mut Transform, Without<TileMap>>,
) {
    let tile_map = tile_map_query.single();

    tile_map.transform_tiles(&mut transform_query);
}

fn spawn_world_tiles(mut commands: Commands, textures: Res<TextureAssets>) {
    // Spawn the entity early, so we can add children.
    let tile_map_id = commands
        .spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(
                -1. * SCREEN_SIZE.x / 2.,
                -1. * SCREEN_SIZE.y / 2.,
                1.0,
            ))
            .with_scale(Vec3::new(2., 2., 1.)),
            ..default()
        })
        .id();

    let mut tile_map = TileMap::new(WORLD_SIZE, IVec2::new(16, 16), IVec2::new(2, 2));

    for x in 4..20 {
        let position = IVec2::new(x, 4);
        let id = spawn_tile(
            &mut commands,
            position,
            textures.barback.clone(),
            Passable::Blocking,
        );
        if x == 12 {
            let spawner = Item::Spawner.spawn(Vec3::new(0., 16., 0.5), &mut commands, &textures);
            commands.entity(id).add_child(spawner);
        }
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }

    for y in 5..10 {
        for x in 2..24 {
            let position = IVec2::new(x, y);
            let id = spawn_floor(&mut commands, &textures, position);
            if y == 6 && x == 12 {
                let orange = Item::Orange.spawn(Vec3::new(0., 0., 0.5), &mut commands, &textures);
                commands.entity(id).add_child(orange);
            } else if y == 8 && x == 28 {
                let banana = Item::Banana.spawn(Vec3::new(0., 0., 0.5), &mut commands, &textures);
                commands.entity(id).add_child(banana);
            }
            tile_map.insert(tile_map_id, id, position, &mut commands);
        }
    }

    for x in 4..20 {
        let position = IVec2::new(x, 10);
        let id = spawn_tile(
            &mut commands,
            position,
            textures.bar.clone(),
            Passable::Blocking,
        );
        if x == 6 {
            let mixer = Item::Mixer.spawn(Vec3::new(0., 16., 0.5), &mut commands, &textures);
            commands.entity(id).add_child(mixer);
        }
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }

    for x in 2..4 {
        let position = IVec2::new(x, 10);
        tile_map.insert(
            tile_map_id,
            spawn_floor(&mut commands, &textures, position),
            position,
            &mut commands,
        );
    }
    for x in 20..24 {
        let position = IVec2::new(x, 10);
        tile_map.insert(
            tile_map_id,
            spawn_floor(&mut commands, &textures, position),
            position,
            &mut commands,
        );
    }

    for y in 11..18 {
        for x in 2..24 {
            let position = IVec2::new(x, y);
            tile_map.insert(
                tile_map_id,
                spawn_floor(&mut commands, &textures, position),
                position,
                &mut commands,
            );
        }
    }

    // Add the tilemap component to the tilemap entity
    commands.entity(tile_map_id).insert(tile_map);
}

fn spawn_floor(
    commands: &mut Commands,
    textures: &Res<TextureAssets>,
    tile_location: IVec2,
) -> Entity {
    spawn_tile(
        commands,
        tile_location,
        textures.floor1.clone(),
        Passable::Passable,
    )
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
) -> Entity {
    let translation = tile_location.as_tile().to_camera_space();
    commands
        .spawn((
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
        ))
        .id()
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
