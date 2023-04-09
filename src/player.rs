use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::npc::{Stats, NPC};
use crate::tilemap::TileMap;
use crate::world::{Passable, Tile, SCALE};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct UserControllable;

#[derive(Component)]
pub struct Player {
    pub movement: Option<Vec2>,
    pub holding: Option<Entity>,
    pub requesting: Option<Entity>,
    pub heading: PlayerHeading,
    pub pickup_action: bool,
    pub interact_action: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            movement: None,
            holding: None,
            requesting: None,
            heading: PlayerHeading::Down,
            pickup_action: false,
            interact_action: false,
        }
    }
}

impl Player {
    fn hold_item(&mut self, player_entity: Entity, item_entity: Entity, commands: &mut Commands) {
        commands.entity(player_entity).add_child(item_entity);
        self.holding = Some(item_entity);
    }

    pub fn request(
        &mut self,
        item: Item,
        player_entity: Entity,
        commands: &mut Commands,
        textures: &Res<TextureAssets>,
    ) {
        let item_entity = item.spawn(Vec3::new(-8., 32., 0.), commands, textures);
        commands.entity(player_entity).add_child(item_entity);
        self.requesting = Some(item_entity);
    }

    pub fn stop_requesting(&mut self, player_entity: Entity, commands: &mut Commands) {
        let Some(item_entity) = std::mem::replace(&mut self.requesting, None) else {
            return;
        };

        commands.entity(item_entity).remove_parent().despawn();
    }
}

pub enum PlayerHeading {
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
            PlayerHeading::Down => IVec2::new(0, -1),
            PlayerHeading::Up => IVec2::new(0, 1),
            PlayerHeading::Left => IVec2::new(-1, 0),
            PlayerHeading::Right => IVec2::new(1, 0),
        }
    }
}

#[derive(Component)]
pub enum Interactable {
    Spawner(Item),
    Mixer(Mixer),
    Container(Container),
    Trash,
}

impl Interactable {
    fn texture(&self, texture_assets: &Res<TextureAssets>) -> Handle<Image> {
        match self {
            Interactable::Spawner(_) => texture_assets.bowl_filled.clone(),
            Interactable::Mixer(_) => texture_assets.mixer.clone(),
            Interactable::Container(_) => texture_assets.bowl_empty.clone(),
            Interactable::Trash => texture_assets.trash.clone(),
        }
    }

    pub fn spawn(
        self,
        translation: Vec3,
        commands: &mut Commands,
        textures: &Res<TextureAssets>,
    ) -> Entity {
        commands
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
            .id()
    }

    pub fn pickup(
        &mut self,
        _entity: Entity,
        commands: &mut Commands,
        textures: &Res<TextureAssets>,
    ) -> Option<Entity> {
        match self {
            Interactable::Spawner(item) => Some(item.clone().spawn_internal(
                Vec3::splat(0.),
                Visibility::Hidden,
                commands,
                textures,
            )),
            Interactable::Mixer(mixer) => {
                // Pickup and spawn entity from mixer if available.
                if let Some(item) = mixer.pickup() {
                    Some(item.spawn_internal(
                        Vec3::splat(0.),
                        Visibility::Hidden,
                        commands,
                        textures,
                    ))
                } else {
                    None
                }
            }
            Interactable::Container(container) => {
                if let None = container.holding {
                    return None;
                };

                let item_entity = std::mem::replace(&mut container.holding, None).unwrap();

                commands.entity(item_entity).remove_parent();

                Some(item_entity)
            }
            Interactable::Trash => None,
        }
    }

    pub fn consume(
        &mut self,
        entity: Entity,
        item: Item,
        item_entity: Entity,
        commands: &mut Commands,
    ) -> bool {
        match self {
            Interactable::Spawner(_) => false,
            Interactable::Mixer(mixer) => {
                mixer.add(item);
                commands.entity(item_entity).remove_parent();
                commands.entity(item_entity).despawn();
                true
            }
            Interactable::Container(container) => {
                let None = container.holding else {
                    return false;
                };
                container.holding = Some(item_entity);
                commands.entity(item_entity).remove_parent();
                commands.entity(entity).add_child(item_entity);

                true
            }
            Interactable::Trash => {
                commands.entity(item_entity).remove_parent();
                commands.entity(item_entity).despawn();
                true
            }
        }
    }

    pub fn interact(&mut self) -> bool {
        match self {
            Interactable::Spawner(_) => false,
            Interactable::Mixer(mixer) => mixer.mix(),
            Interactable::Container(_) => false,
            Interactable::Trash => false,
        }
    }
}

pub struct Mixer {
    contains: Vec<Item>,
    result: Option<Item>,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            contains: Vec::new(),
            result: None,
        }
    }

    pub fn add(&mut self, item: Item) {
        self.contains.push(item);
    }

    pub fn mix(&mut self) -> bool {
        if self.contains.len() > 0 {
            self.contains.clear();
            self.result = Some(Item::Beverage(Beverage {
                stats: Stats::default(),
            }));
            true
        } else {
            false
        }
    }

    pub fn pickup(&mut self) -> Option<Item> {
        std::mem::replace(&mut self.result, None)
    }
}

pub struct Container {
    holding: Option<Entity>,
}

impl Container {
    pub fn new() -> Self {
        Container { holding: None }
    }
}

#[derive(Component, Clone)]
pub enum Item {
    Orange,
    Banana,
    Beverage(Beverage),
}

#[derive(Clone)]
pub struct Beverage {
    stats: Stats,
}

impl Item {
    fn texture(&self, texture_assets: &Res<TextureAssets>) -> Handle<Image> {
        match self {
            Item::Orange => texture_assets.orange.clone(),
            Item::Banana => texture_assets.banana.clone(),
            Item::Beverage(_) => texture_assets.beverage.clone(),
        }
    }

    pub fn spawn(
        self,
        translation: Vec3,
        commands: &mut Commands,
        textures: &Res<TextureAssets>,
    ) -> Entity {
        self.spawn_internal(translation, Visibility::Visible, commands, textures)
    }

    pub fn spawn_internal(
        self,
        translation: Vec3,
        visibility: Visibility,
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
                    visibility,
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
            .add_systems(
                (
                    handle_actions,
                    player_pickup,
                    player_interact,
                    position_held.after(player_pickup),
                )
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    // Swag a position for the player and NPC, based on knowing the tile map origin of -400,-300
    // Player at 12, 14
    let position = Vec3::new(-400. + (12. * 32.), -300. + (14. * 32.), 18. - 14. + 0.5);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: textures.luna.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                anchor: bevy::sprite::Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(position).with_scale(SCALE),
            ..Default::default()
        },
        Player::default(),
        UserControllable,
    ));

    // NPC at 12, 6
    let position = Vec3::new(-400. + (12. * 32.), -300. + (6. * 32.), 18. - 6. + 0.5);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: textures.npc1.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                anchor: bevy::sprite::Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(position).with_scale(SCALE),
            ..Default::default()
        },
        Player::default(),
        NPC::default(),
    ));
}

fn move_player(
    time: Res<Time>,
    mut player_query: Query<
        (&mut Transform, &mut TextureAtlasSprite, &mut Player),
        (Without<Tile>, Without<Item>, Without<TileMap>),
    >,
    tile_map_query: Query<(&TileMap, &Transform), (With<TileMap>, Without<Player>)>,
    tile_query: Query<(&Tile, &Transform)>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    for (mut player_transform, mut sprite, mut player) in &mut player_query {
        sprite.index = player.heading.sprite_index();

        let Some(player_movement) = player.movement else {
            continue;
        };
        let speed = 150.;
        let movement = Vec3::new(
            player_movement.x * speed * time.delta_seconds(),
            player_movement.y * speed * time.delta_seconds(),
            0.,
        );

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
                    player_transform.translation.z = tile_map.tile_z(&new_tile) + 0.5;
                }
            }
        }
    }
}

fn handle_actions(
    actions: Res<Actions>,
    mut player_query: Query<&mut Player, With<UserControllable>>,
) {
    let mut player = player_query.single_mut();
    player.pickup_action = actions.pick_up.0 == true && actions.pick_up.1 == false;
    player.interact_action = actions.interact.0 == true && actions.interact.1 == false;

    player.movement = actions.player_movement;
}

fn player_pickup(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut player_query: Query<
        (Entity, &Transform, &mut Player),
        (Without<TileMap>, Without<Interactable>, Without<Tile>),
    >,
    tile_map_query: Query<
        (&TileMap, &Transform),
        (Without<Player>, Without<Tile>, Without<Interactable>),
    >,
    mut interactable_query: Query<
        (Entity, &Transform, &mut Interactable),
        (Without<Player>, Without<Tile>, Without<TileMap>),
    >,
    tile_query: Query<&Children, With<Tile>>,
    item_query: Query<&Item>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();
    for (player_entity, player_transform, mut player) in &mut player_query {
        if !player.pickup_action {
            continue;
        }
        player.pickup_action = false;

        let tile_index =
            tile_map.camera_to_tile(tile_map_transform.translation, player_transform.translation);

        if let Some(holding) = player.holding {
            // Get the held item
            let item = item_query.get(holding).unwrap();
            // Look for an interactable that can receive the item.
            for idx in [tile_index, tile_index + player.heading.as_offset()] {
                let Some(tile_entity) = tile_map.tile_at(idx) else {
                continue;
            };

                let Ok(children) = tile_query.get(tile_entity) else {
                continue;
            };

                for child in children.iter() {
                    let Ok((i_entity, _interactable_transform, mut interactable)) = interactable_query.get_mut(*child) else {
                        continue;
                    };

                    if interactable.consume(i_entity, item.clone(), holding, &mut commands) {
                        // Drop the entity, hold nothing.
                        println!("Drop in interactable");
                        player.holding = None;
                        break;
                    }
                }
            }
        } else {
            // Try to pick up.
            for idx in [tile_index, tile_index + player.heading.as_offset()] {
                let Some(tile_entity) = tile_map.tile_at(idx) else {
                continue;
            };

                let Ok(children) = tile_query.get(tile_entity) else {
                continue;
            };

                for child in children.iter() {
                    let Ok((i_entity, _interactable_transform, mut interactable)) = interactable_query.get_mut(*child) else {
                    continue;
                };

                    if let Some(item_entity) =
                        interactable.pickup(i_entity, &mut commands, &textures)
                    {
                        println!("Pickup");
                        player.hold_item(player_entity, item_entity, &mut commands);
                        break;
                    }
                }
            }
        }
    }
}

fn position_held(
    player_query: Query<&Player, Without<Item>>,
    mut item_query: Query<(&mut Transform, &mut Visibility), (With<Item>, Without<Player>)>,
) {
    for player in &player_query {
        let Some(item_entity) = player.holding else {
        continue;
    };
        let Ok((mut transform, mut visibility)) = item_query.get_mut(item_entity) else {
        continue;
    };

        *visibility = Visibility::Visible;

        let x = {
            let offset = player.heading.as_offset();
            if offset.x < 0 {
                8. * offset.x as f32
            } else if offset.x > 0 {
                0.
            } else {
                4. * offset.y as f32
            }
        };
        transform.translation = Vec3::new(x, 16.0, 0.5);
    }
}

fn player_interact(
    player_query: Query<
        (&Transform, &Player),
        (Without<TileMap>, Without<Interactable>, Without<Tile>),
    >,
    tile_map_query: Query<
        (&TileMap, &Transform),
        (Without<Player>, Without<Tile>, Without<Interactable>),
    >,
    mut interactable_query: Query<
        (Entity, &Transform, &mut Interactable),
        (Without<Player>, Without<Tile>, Without<TileMap>),
    >,
    tile_query: Query<&Children, With<Tile>>,
) {
    for (player_transform, player) in &player_query {
        if !player.interact_action {
            continue;
        }

        let (tile_map, tile_map_transform) = tile_map_query.single();

        let tile_index =
            tile_map.camera_to_tile(tile_map_transform.translation, player_transform.translation);

        for idx in [tile_index, tile_index + player.heading.as_offset()] {
            let Some(tile_entity) = tile_map.tile_at(idx) else {
            continue;
        };

            let Ok(children) = tile_query.get(tile_entity) else {
            continue;
        };

            for child in children.iter() {
                let Ok((_i_entity, _interactable_transform, mut interactable)) = interactable_query.get_mut(*child) else {
                continue;
            };

                if interactable.interact() {
                    println!("Interact successful");
                    break;
                }
            }
        }
    }
}
