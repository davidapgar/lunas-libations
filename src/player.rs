use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::tilemap::TileMap;
use crate::world::{AsTile, Passable, Tile, TileSpace, ToTileIndex, SCALE};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player {
    holding: Option<Entity>,
    heading: PlayerHeading,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            holding: None,
            heading: PlayerHeading::Down,
        }
    }
}

impl Player {
    fn hold_item(&mut self, player_entity: Entity, item_entity: Entity, commands: &mut Commands) {
        commands.entity(player_entity).add_child(item_entity);
        self.holding = Some(item_entity);
    }
}

enum PlayerHeading {
    Down,
    Up,
    Left,
    Right,
}

impl PlayerHeading {
    fn from_vec(movement: Vec2) -> Self {
        if movement.y < 0. {
            PlayerHeading::Down
        } else if movement.y > 0. {
            PlayerHeading::Up
        } else if movement.x < 0. {
            PlayerHeading::Left
        } else if movement.x > 0. {
            PlayerHeading::Right
        } else {
            PlayerHeading::Down
        }
    }

    fn sprite_index(&self) -> usize {
        match self {
            PlayerHeading::Down => 0,
            PlayerHeading::Up => 1,
            PlayerHeading::Left => 2,
            PlayerHeading::Right => 3,
        }
    }

    fn as_offset(&self) -> IVec2 {
        match self {
            PlayerHeading::Down => IVec2::new(0, 1),
            PlayerHeading::Up => IVec2::new(0, -1),
            PlayerHeading::Left => IVec2::new(-1, 0),
            PlayerHeading::Right => IVec2::new(1, 0),
        }
    }
}

#[derive(Component)]
pub enum Item {
    Orange,
    Banana,
}

impl Item {
    fn texture(&self, texture_assets: &Res<TextureAssets>) -> Handle<Image> {
        match self {
            Item::Orange => texture_assets.orange.clone(),
            Item::Banana => texture_assets.banana.clone(),
        }
    }

    pub fn spawn(
        self,
        translation: Vec3,
        commands: &mut Commands,
        textures: &Res<TextureAssets>,
    ) -> Entity {
        let item_id = commands
            .spawn((
                SpriteBundle {
                    texture: self.texture(textures),
                    transform: Transform::from_translation(translation),
                    sprite: Sprite {
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                    ..default()
                },
                self,
            ))
            .id();
        item_id
    }
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(GameState::Playing)))
            .add_system(move_player.in_set(OnUpdate(GameState::Playing)))
            .add_system(player_interact.in_set(OnUpdate(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: textures.luna.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(IVec2::new(12, 9).as_tile().to_camera_space())
                .with_scale(SCALE),
            ..Default::default()
        })
        .insert(Player::default());
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<
        (&mut Transform, &mut TextureAtlasSprite, &mut Player),
        (Without<Tile>, Without<Item>, Without<TileMap>),
    >,
    tile_map_query: Query<(&TileMap, &Transform), (With<TileMap>, Without<Player>)>,
    tile_query: Query<(&Tile, &Transform)>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    if actions.player_movement.is_none() {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    for (mut player_transform, mut sprite, mut player) in &mut player_query {
        player.heading = PlayerHeading::from_vec(movement.truncate());
        sprite.index = player.heading.sprite_index();

        let new_translation = player_transform.translation + movement;
        let new_tile = tile_map
            .to_tile(tile_map.to_tile_space(tile_map_transform.translation, new_translation));
        if new_tile
            == tile_map.to_tile(
                tile_map
                    .to_tile_space(tile_map_transform.translation, player_transform.translation),
            )
        {
            player_transform.translation += movement;
            continue;
        }

        if let Some(tile_entity) = tile_map.tile_at(new_tile) {
            if let Ok((tile, _transform)) = tile_query.get(tile_entity) {
                if let Passable::Passable = tile.0 {
                    player_transform.translation += movement;
                    player_transform.translation.z = new_tile.y as f32 + 1.5;
                }
            }
        }
    }
}

fn player_interact(
    mut commands: Commands,
    actions: Res<Actions>,
    mut player_query: Query<
        (Entity, &Transform, &mut Player),
        (Without<TileMap>, Without<Item>, Without<Tile>),
    >,
    tile_map_query: Query<(&TileMap, &Transform), (Without<Player>, Without<Tile>, Without<Item>)>,
    mut item_query: Query<
        (Entity, &mut Transform, &Item),
        (Without<Player>, Without<Tile>, Without<TileMap>),
    >,
    tile_query: Query<&Children, With<Tile>>,
) {
    // Only interact if the button was just released.
    if !(actions.pick_up.0 == true && actions.pick_up.1 == false) {
        return;
    }

    let (player_entity, player_transform, mut player) = player_query.single_mut();
    let (tile_map, tile_map_transform) = tile_map_query.single();

    let tile_index =
        tile_map.camera_to_tile(tile_map_transform.translation, player_transform.translation);
    if let Some(holding) = player.holding {
        // Drop item.
        commands.entity(holding).remove_parent();
        player.holding = None;

        if let Some(tile_entity) = tile_map.tile_at(tile_index) {
            println!("Drop on tile");
            let (_, mut item_transform, _) = item_query.get_mut(holding).unwrap();
            item_transform.translation = Vec3::new(0., 0., 0.5);
            commands.entity(tile_entity).add_child(holding);
        }
    } else {
        // Try to pick up.
        for idx in [tile_index, tile_index + player.heading.as_offset()] {
            if let Some(tile_entity) = tile_map.tile_at(idx) {
                if pickup(
                    player_entity,
                    &mut player,
                    tile_entity,
                    &mut commands,
                    &mut item_query,
                    &tile_query,
                ) {
                    break;
                }
            }
        }
    }
}

fn pickup(
    player_entity: Entity,
    player: &mut Player,
    tile_entity: Entity,
    commands: &mut Commands,
    item_query: &mut Query<
        (Entity, &mut Transform, &Item),
        (Without<Player>, Without<Tile>, Without<TileMap>),
    >,
    tile_query: &Query<&Children, With<Tile>>,
) -> bool {
    if let Ok(children) = tile_query.get(tile_entity) {
        for child in children.iter() {
            if let Ok((item_entity, mut item_transform, _item)) = item_query.get_mut(*child) {
                println!("Pickup");
                item_transform.translation = Vec3::new(12., 16., 0.5);
                commands.entity(item_entity).remove_parent();
                commands.entity(player_entity).add_child(item_entity);
                player.holding = Some(item_entity);
                return true;
            }
        }
    }

    false
}
