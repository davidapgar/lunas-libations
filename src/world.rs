use crate::loading::TextureAssets;
use crate::player::{Container, Interactable, Item, Mixer};
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
                0.,
            ))
            .with_scale(Vec3::new(2., 2., 1.)),
            ..default()
        })
        .id();

    let mut tile_map = TileMap::new(WORLD_SIZE, IVec2::new(16, 16), IVec2::new(2, 2));
    // tiles are 0,0 bottom left to ~(25,18) top right
    //
    // Layout (B = bar, C = counter, f = floor) (extended out)
    // P player N npc
    // 16 fBBBBf
    // 15 ffffff
    // 14 ffPfff
    // 13 ffffff
    // 12 fCCCCf
    // 11 ffffff
    // 10 ffffff
    // 09 ffffff
    // 08 fTfTff (tables)
    // 07 ffffff
    // 06 ffNfff
    // 05 ffffff
    // 04 ffffff

    // Bar back, flooring
    let y = 16;
    for x in 4..20 {
        let position = IVec2::new(x, y);
        let id = spawn_tile(&mut commands, textures.barback.clone(), Passable::Blocking);
        if x == 12 {
            let spawner = Interactable::Spawner(Item::Banana).spawn(
                Vec3::new(0., 16., 0.5),
                &mut commands,
                &textures,
            );
            commands.entity(id).add_child(spawner);
        }
        if x == 6 {
            let trash =
                Interactable::Trash.spawn(Vec3::new(0., 16., 0.5), &mut commands, &textures);
            commands.entity(id).add_child(trash);
        }
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }
    for x in 2..4 {
        let position = IVec2::new(x, y);
        let id = spawn_floor(&mut commands, &textures);
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }
    for x in 20..22 {
        let position = IVec2::new(x, y);
        let id = spawn_floor(&mut commands, &textures);
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }

    // Flooring
    for y in 13..16 {
        for x in 2..22 {
            let position = IVec2::new(x, y);
            let id = spawn_floor(&mut commands, &textures);
            if y == 16 && x == 12 {
                let orange = Item::Orange.spawn(Vec3::new(0., 0., 0.5), &mut commands, &textures);
                commands.entity(id).add_child(orange);
            } else if y == 17 && x == 22 {
                let banana = Item::Banana.spawn(Vec3::new(0., 0., 0.5), &mut commands, &textures);
                commands.entity(id).add_child(banana);
            }
            tile_map.insert(tile_map_id, id, position, &mut commands);
        }
    }

    // Bar, flooring
    let y = 12;
    for x in 4..20 {
        let position = IVec2::new(x, y);
        let id = spawn_tile(&mut commands, textures.bar.clone(), Passable::Blocking);
        if x == 6 {
            let mixer = Interactable::Mixer(Mixer::new()).spawn(
                Vec3::new(0., 16., 0.5),
                &mut commands,
                &textures,
            );
            commands.entity(id).add_child(mixer);
        }
        if x == 12 {
            let container = Interactable::Container(Container::new()).spawn(
                Vec3::new(0., 16., 0.5),
                &mut commands,
                &textures,
            );
            commands.entity(id).add_child(container);
        }
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }

    for x in 2..4 {
        let position = IVec2::new(x, y);
        let id = spawn_floor(&mut commands, &textures);
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }
    for x in 20..22 {
        let position = IVec2::new(x, y);
        let id = spawn_floor(&mut commands, &textures);
        tile_map.insert(tile_map_id, id, position, &mut commands);
    }

    // Floor, tables
    for y in 04..12 {
        for x in 2..22 {
            let position = IVec2::new(x, y);
            if y == 10 && (x == 4 || x == 8 || x == 16 || x == 20) {
                tile_map.insert(
                    tile_map_id,
                    spawn_tile(&mut commands, textures.table.clone(), Passable::Blocking),
                    position,
                    &mut commands,
                );
            } else {
                tile_map.insert(
                    tile_map_id,
                    spawn_floor(&mut commands, &textures),
                    position,
                    &mut commands,
                );
            }
        }
    }

    // Add the tilemap component to the tilemap entity
    commands.entity(tile_map_id).insert(tile_map);
}

fn spawn_floor(commands: &mut Commands, textures: &Res<TextureAssets>) -> Entity {
    spawn_tile(commands, textures.floor1.clone(), Passable::Passable)
}

// Camera defaults to center of screen being 0.0/0.0
// So tile 0,0 (top left) will be at -25*16/20*16
// tile 50,40 will be at 25*16/-20*16
// tile at 25,20 will be at 0/0
fn spawn_tile(commands: &mut Commands, texture: Handle<Image>, passable: Passable) -> Entity {
    let translation = Vec3::splat(0.);
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
