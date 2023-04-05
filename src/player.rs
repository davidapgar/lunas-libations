use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::world::{AsTile, Passable, Tile, TileSpace, ToTileIndex, SCALE};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player {
    holding: Option<Entity>,
}

impl Default for Player {
    fn default() -> Self {
        Player { holding: None }
    }
}

impl Player {
    fn hold_item(&mut self, player_entity: Entity, item_entity: Entity, commands: &mut Commands) {
        commands.entity(player_entity).add_child(item_entity);
        self.holding = Some(item_entity);
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
            .add_system(player_pickup.in_set(OnUpdate(GameState::Playing)));
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
    mut player_query: Query<&mut Transform, (With<Player>, Without<Tile>)>,
    world_query: Query<(&Transform, &Tile), Without<Player>>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    for mut player_transform in &mut player_query {
        let new_translation = player_transform.translation + movement;
        let new_tile = new_translation.to_tile_index();
        if new_tile == player_transform.translation.to_tile_index() {
            player_transform.translation += movement;
            continue;
        }

        for (transform, passable) in &world_query {
            let tile = transform.translation.to_tile_index();
            if tile == new_tile {
                if let Passable::Passable = passable.0 {
                    player_transform.translation += movement;
                    player_transform.translation.z = new_tile.y as f32 + 0.5;
                }
                break;
            }
        }
    }
}

fn player_pickup(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    actions: Res<Actions>,
    mut player_query: Query<(Entity, &Transform, &mut Player), Without<Tile>>,
    tile_query: Query<
        (Entity, &Transform, Option<&Children>),
        (With<Tile>, Without<Player>, Without<Item>),
    >,
    mut item_query: Query<(Entity, &mut Transform, &Item), (Without<Player>, Without<Tile>)>,
) {
    if actions.pick_up.0 == true && actions.pick_up.1 == false {
        let (player_entity, transform, mut player) = player_query.single_mut();

        if let Some(holding) = player.holding {
            commands.entity(holding).remove_parent();
            player.holding = None;

            let player_tile = transform.translation.to_tile_index();
            if let Some(tile_entity) = tile_at(player_tile, &tile_query) {
                println!("Drop on tile");
                let (_, mut item_transform, _) = item_query.get_mut(holding).unwrap();
                item_transform.translation = Vec3::new(0., 0., 0.5);
                commands.entity(tile_entity).add_child(holding);
            } else {
                println!("despawn");
                commands.entity(holding).despawn();
            }
        } else {
            if let Some(tile_entity) = tile_at(transform.translation.to_tile_index(), &tile_query) {
                if let Ok((_, _, Some(children))) = tile_query.get(tile_entity) {
                    for child in children.iter() {
                        if let Ok((_, mut item_transform, _item)) = item_query.get_mut(*child) {
                            println!("pickup");
                            item_transform.translation = Vec3::new(12., 16., 0.5);
                            commands.entity(*child).remove_parent();
                            commands.entity(player_entity).add_child(*child);
                            player.holding = Some(*child);
                            return;
                        }
                    }
                }
            }

            println!("spawn");
            let item_entity =
                Item::Banana.spawn(Vec3::new(12., 16., 0.5), &mut commands, &textures);
            player.hold_item(player_entity, item_entity, &mut commands);
        }
    }
}

fn tile_at(
    tile_index: IVec2,
    tile_query: &Query<
        (Entity, &Transform, Option<&Children>),
        (With<Tile>, Without<Player>, Without<Item>),
    >,
) -> Option<Entity> {
    for (entity, transform, _) in tile_query {
        if transform.translation.to_tile_index() == tile_index {
            return Some(entity);
        }
    }
    None
}
