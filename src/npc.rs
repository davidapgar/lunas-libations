use crate::loading::TextureAssets;
use crate::player::{Interactable, Item, Player, PlayerHeading};
use crate::tilemap::TileMap;
use crate::world::Tile;
use crate::GameState;
use bevy::prelude::*;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (update_npc_stats, npc_move, npc_ai.after(npc_move))
                .in_set(OnUpdate(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct NPC {
    /// Current internal stats driving the AI.
    stats: Stats,
    /// Goal to move to. If `None`, will stand still.
    move_to: Option<IVec2>,
    behavior: Behavior,
    timer: Timer,
}

#[derive(Default, Copy, Clone)]
pub struct Stats {
    /// How quenched or thirsty. Negative is thirsy
    pub quench: f32,
    /// Happy or sad. Positive is happy.
    pub mood: f32,
    /// How inebriated. Sober is 0 or below.
    pub drunk: f32,
}

enum Behavior {
    Idle,
    Request(Item),
    Grab,
    Drink,
    Chat,
    Fight,
    Dance,
    Cry,
    Puke,
}

impl Default for Behavior {
    fn default() -> Self {
        Behavior::Idle
    }
}

fn update_npc_stats(time: Res<Time>, mut npc_query: Query<&mut NPC>) {
    let delta = time.delta_seconds();
    for mut npc in &mut npc_query {
        // TODO: Tune these. They drop a percentage of the value towards zero, then also a
        // constant.
        npc.stats.quench = npc.stats.quench - (npc.stats.quench * delta * 0.20) - (delta * 20.);
        npc.stats.mood = npc.stats.mood - (npc.stats.mood * delta * 0.10) - (delta * 10.);
        npc.stats.drunk = npc.stats.quench - (npc.stats.quench * delta * 0.05) - (delta * 1.);
    }
}

fn npc_move(
    mut query: Query<(&mut NPC, &mut Player, &Transform)>,
    tile_map_query: Query<(&TileMap, &Transform)>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    for (mut npc, mut player, npc_transform) in &mut query {
        let Some(move_to) = npc.move_to else {
            player.movement = None;
            continue;
        };

        let npc_tile =
            tile_map.camera_to_tile(tile_map_transform.translation, npc_transform.translation);
        if npc_tile != move_to {
            let movement = (move_to - npc_tile).as_vec2().normalize_or_zero();
            player.movement = Some(movement);
        } else {
            npc.move_to = None;
            player.movement = None;
        }
    }
}

fn npc_ai(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut NPC, &mut Player, &Transform)>,
    tile_map_query: Query<(&TileMap, &Transform)>,
    interactable_query: Query<(Entity, &Interactable, &Parent)>,
    tile_query: Query<(&Tile, &Children)>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    for (entity, mut npc, mut player, npc_transform) in &mut query {
        let npc_tile =
            tile_map.camera_to_tile(tile_map_transform.translation, npc_transform.translation);

        npc.timer.tick(time.delta());
        if !npc.timer.finished() {
            continue;
        }

        match &npc.behavior {
            Behavior::Idle => {
                println!("Update to request");
                // find a container
                let mut dest: Option<IVec2> = None;
                for (_, interactable, parent) in &interactable_query {
                    if let Interactable::Container(_) = interactable {
                        if let Some(location) = tile_map.find_tile(parent.get()) {
                            dest = Some(location - IVec2::new(0, 1));
                            break;
                        }
                    }
                }
                npc.behavior = Behavior::Request(Item::Banana);
                npc.move_to = dest;
                player.request(Item::Banana, entity, &mut commands, &textures);
            }
            Behavior::Request(_item) => {
                if let None = npc.move_to {
                    println!("Update to grab");
                    player.heading = PlayerHeading::Up;
                    npc.behavior = Behavior::Grab;
                }
            }
            Behavior::Grab => {
                if let Some(_) = player.holding {
                    println!("Update to drink");
                    player.stop_requesting(entity, &mut commands);
                    npc.behavior = Behavior::Drink;
                    npc.move_to = Some(npc_tile + IVec2::new(0, -5));
                } else {
                    player.pickup_action = true;
                }
            }
            Behavior::Drink => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Drink");
                if let Some(holding) = std::mem::replace(&mut player.holding, None) {
                    commands.entity(holding).remove_parent().despawn();
                }
                npc.behavior = Behavior::Idle;
                npc.timer = Timer::from_seconds(0.5, TimerMode::Once);
            }
            _ => {}
        }
    }
}
