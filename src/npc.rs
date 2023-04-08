use crate::loading::TextureAssets;
use crate::player::{Item, Player};
use crate::tilemap::TileMap;
use crate::world::{AsTile, Passable, Tile, TileSpace, SCALE};
use crate::GameState;
use bevy::prelude::*;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((update_npc_stats, npc_move, npc_ai).in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component, Default)]
pub struct NPC {
    /// Current internal stats driving the AI.
    stats: Stats,
    /// Goal to move to. If `None`, will stand still.
    move_to: Option<IVec2>,
    behavior: Behavior,
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
    mut query: Query<(&NPC, &mut Player, &Transform)>,
    tile_map_query: Query<(&TileMap, &Transform)>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    for (npc, mut player, npc_transform) in &mut query {
        let Some(move_to) = npc.move_to else {
            player.movement = None;
            continue;
        };

        let npc_tile =
            tile_map.camera_to_tile(tile_map_transform.translation, npc_transform.translation);
        if npc_tile != move_to {
            let movement = (npc_tile - move_to).as_vec2().normalize_or_zero();
            player.movement = Some(movement);
        }
    }
}

fn npc_ai(mut query: Query<(&mut NPC, &Transform)>, tile_map_query: Query<(&TileMap, &Transform)>) {
    let (tile_map, tile_map_transform) = tile_map_query.single();

    for (mut npc, npc_transform) in &mut query {
        let npc_tile =
            tile_map.camera_to_tile(tile_map_transform.translation, npc_transform.translation);

        if let Behavior::Idle = npc.behavior {
            npc.behavior = Behavior::Request(Item::Banana);
            npc.move_to = Some(npc_tile + IVec2::new(0, -4));
        }
    }
}
