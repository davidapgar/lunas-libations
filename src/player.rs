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
    fn hold_item(
        &mut self,
        entity: Entity,
        item: Item,
        commands: &mut Commands,
        textures: Res<TextureAssets>,
    ) {
        let item_id = commands
            .spawn((
                SpriteBundle {
                    texture: item.texture(textures),
                    transform: Transform::from_translation(Vec3::new(8.0, 24., 1.)),
                    sprite: Sprite {
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                    ..default()
                },
                item,
            ))
            .id();
        commands.entity(entity).push_children(&[item_id]);
        self.holding = Some(item_id);
    }
}

#[derive(Component)]
pub enum Item {
    Orange,
    Banana,
}

impl Item {
    fn texture(&self, texture_assets: Res<TextureAssets>) -> Handle<Image> {
        match self {
            Item::Orange => texture_assets.orange.clone(),
            Item::Banana => texture_assets.banana.clone(),
        }
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
        .spawn(SpriteBundle {
            texture: textures.texture_logo.clone(),
            transform: Transform::from_translation(IVec2::new(12, 9).as_tile().to_camera_space())
                .with_scale(SCALE),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
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
    mut world_query: Query<(Entity, &mut Transform), (With<Tile>, Without<Player>)>,
) {
    if actions.pick_up.0 == true && actions.pick_up.1 == false {
        let (entity, transform, mut player) = player_query.single_mut();

        if let Some(holding) = player.holding {
            commands.entity(entity).remove_children(&[holding]);
            commands.entity(holding).despawn();
            player.holding = None;
        } else {
            player.hold_item(entity, Item::Banana, &mut commands, textures);
        }
    }
}
