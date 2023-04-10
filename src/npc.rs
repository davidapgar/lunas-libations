use crate::animate::{Animation, AnimationComponent};
use crate::loading::TextureAssets;
use crate::player::{Beverage, Interactable, Item, Player, PlayerHeading};
use crate::tilemap::TileMap;
use crate::world::Tile;
use crate::GameState;
use bevy::prelude::*;
use rand::prelude::*;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_npc_animations.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (update_npc_stats, npc_move, npc_ai.after(npc_move))
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct NPCAnimations {
    dance: Animation,
    talk_right: Animation,
    talk_left: Animation,
    puke: Animation,
    cry: Animation,
    punch_left: Animation,
    punch_right: Animation,
    drink: Animation,
}

impl NPCAnimations {
    fn new() -> Self {
        NPCAnimations {
            dance: Animation::new(&[4, 5, 6, 7], 0.3, true),
            talk_right: Animation::new(&[3, 9], 0.4, true),
            talk_left: Animation::new(&[2, 8], 0.4, true),
            puke: Animation::new(&[2, 10, 11, 12], 0.3, true),
            cry: Animation::new(&[0, 13, 14, 15, 16, 17], 0.3, true),
            punch_left: Animation::new(&[2, 18, 19, 20], 0.3, true),
            punch_right: Animation::new(&[3, 21, 22, 23], 0.3, true),
            drink: Animation::new(&[24, 25], 0.3, true),
        }
    }
}

fn setup_npc_animations(mut commands: Commands) {
    commands.spawn(NPCAnimations::new());
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

#[derive(Default, Copy, Clone, Debug)]
pub struct Stats {
    /// How quenched or thirsty. Negative is thirsy
    pub quench: f32,
    /// Happy or sad. Positive is happy.
    pub mood: f32,
    /// How inebriated. Sober is 0 or below.
    pub drunk: f32,
}

impl std::ops::Add for Stats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            quench: self.quench + other.quench,
            mood: self.mood + other.mood,
            drunk: self.drunk + other.drunk,
        }
    }
}

impl std::ops::AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
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
        /*
        npc.stats.quench = npc.stats.quench - (npc.stats.quench * delta * 0.20) - (delta * 2.0);
        npc.stats.mood = npc.stats.mood - (npc.stats.mood * delta * 0.10) - (delta * 1.0);
        npc.stats.drunk = npc.stats.drunk - (npc.stats.drunk * delta * 0.05) - (delta * 0.1);
        */
        npc.stats.quench = npc.stats.quench - (delta * 0.5);
        npc.stats.mood = npc.stats.mood - (delta * 0.3);
        npc.stats.drunk = npc.stats.drunk - (delta * 0.1);
        if npc.stats.quench < 0. {
            npc.stats.quench = 0.;
        }
    }
}

fn npc_consume_drink(npc: &mut NPC, item: &Item) {
    if let Item::Beverage(beverage) = item {
        npc.stats += beverage.stats;
    } else {
        npc.stats.quench -= 10.;
        npc.stats.mood -= 5.;
        npc.stats.drunk -= 3.;
    }
}

fn npc_decide_next_action(npc: &NPC) -> Behavior {
    println!("Decide action, current NPC stats: {:?}", npc.stats);
    if npc.stats.quench <= 10. {
        return Behavior::Request(Item::Beverage(Beverage::default()));
    }

    if npc.stats.drunk < 30. {
        if npc.stats.mood < 0. {
            Behavior::Cry
        } else {
            Behavior::Chat
        }
    } else if npc.stats.drunk < 60. {
        if npc.stats.mood < 0. {
            Behavior::Fight
        } else {
            Behavior::Dance
        }
    } else {
        Behavior::Puke
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
    npc_animations_query: Query<&NPCAnimations>,
    mut query: Query<(
        Entity,
        &mut NPC,
        &mut Player,
        &Transform,
        &mut AnimationComponent,
    )>,
    tile_map_query: Query<(&TileMap, &Transform)>,
    interactable_query: Query<(Entity, &Interactable, &Parent)>,
    tile_query: Query<(&Tile, Option<&Children>)>,
    item_query: Query<&Item>,
) {
    let (tile_map, tile_map_transform) = tile_map_query.single();
    let npc_animations = npc_animations_query.single();

    for (entity, mut npc, mut player, npc_transform, mut animation) in &mut query {
        let npc_tile =
            tile_map.camera_to_tile(tile_map_transform.translation, npc_transform.translation);

        npc.timer.tick(time.delta());
        if !npc.timer.finished() {
            continue;
        }

        match &npc.behavior {
            Behavior::Idle => {
                println!("Update to request");
                match npc_decide_next_action(&npc) {
                    Behavior::Request(_) => {
                        npc_to_request(
                            &mut commands,
                            &textures,
                            entity,
                            &mut npc,
                            &mut player,
                            &mut animation,
                            tile_map,
                            &interactable_query,
                            &tile_query,
                        );
                    }
                    Behavior::Chat => {
                        npc_to_chat(&mut npc);
                    }
                    Behavior::Fight => {
                        npc_to_fight(&mut npc);
                    }
                    Behavior::Dance => {
                        npc_to_dance(&mut npc);
                    }
                    Behavior::Cry => {
                        npc_to_cry(&mut npc);
                    }
                    Behavior::Puke => npc_to_puke(&mut npc),
                    _ => {}
                }
            }
            Behavior::Request(_item) => {
                if let None = npc.move_to {
                    println!("Update to grab");
                    npc_to_grab(&mut npc, &mut player);
                }
            }
            Behavior::Grab => {
                if let Some(_) = player.holding {
                    println!("Update to drink");
                    npc_to_drink(
                        &mut commands,
                        entity,
                        &mut npc,
                        &mut player,
                        tile_map,
                        &tile_query,
                    );
                } else {
                    player.pickup_action = true;
                }
            }
            Behavior::Drink => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Drink");
                npc_start_drinking(
                    &mut commands,
                    &mut npc,
                    &mut player,
                    &mut animation,
                    npc_animations,
                    &item_query,
                );
            }
            Behavior::Chat => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Chat");
                npc_start_chat(&mut npc, &mut animation, npc_animations);
            }
            Behavior::Fight => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Fight");
                npc_start_fight(&mut npc, &mut animation, npc_animations);
            }
            Behavior::Dance => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Dance");
                npc_start_dance(&mut npc, &mut animation, npc_animations);
            }
            Behavior::Cry => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Cry");
                npc_start_cry(&mut npc, &mut animation, npc_animations);
            }
            Behavior::Puke => {
                let None = npc.move_to else {
                    continue;
                };
                println!("Puke");
                npc_start_puke(&mut npc, &mut animation, npc_animations);
            }
        }
    }
}

fn npc_to_request(
    commands: &mut Commands,
    textures: &Res<TextureAssets>,
    entity: Entity,
    npc: &mut NPC,
    player: &mut Player,
    animation: &mut AnimationComponent,
    tile_map: &TileMap,
    interactable_query: &Query<(Entity, &Interactable, &Parent)>,
    tile_query: &Query<(&Tile, Option<&Children>)>,
) {
    animation.stop_animation();
    // find a container
    let containers = all_containers(&tile_map, &interactable_query, &tile_query);
    let dest = if containers.len() > 0 {
        let (point, _) = containers.choose(&mut rand::thread_rng()).unwrap();
        Some(*point + IVec2::new(0, -1))
    } else {
        None
    };
    npc.move_to = dest;
    player.request(
        Item::Beverage(Beverage::default()),
        entity,
        commands,
        textures,
    );
    npc.behavior = Behavior::Request(Item::Banana);
}

fn npc_to_grab(npc: &mut NPC, player: &mut Player) {
    player.heading = PlayerHeading::Up;
    npc.behavior = Behavior::Grab;
}

fn npc_to_drink(
    commands: &mut Commands,
    entity: Entity,
    npc: &mut NPC,
    player: &mut Player,
    tile_map: &TileMap,
    tile_query: &Query<(&Tile, Option<&Children>)>,
) {
    player.stop_requesting(entity, commands);
    let tables = all_tables(&tile_map, &tile_query);
    let (dest, _) = tables.choose(&mut rand::thread_rng()).unwrap();
    npc.move_to = Some(*dest + IVec2::new(0, 1));
    npc.behavior = Behavior::Drink;
}

fn npc_start_drinking(
    commands: &mut Commands,
    npc: &mut NPC,
    player: &mut Player,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
    item_query: &Query<&Item>,
) {
    if let Some(holding) = std::mem::replace(&mut player.holding, None) {
        if let Ok(item) = item_query.get(holding) {
            npc_consume_drink(npc, item);
        }
        commands.entity(holding).remove_parent().despawn();
    }
    animation.start_animation(&npc_animations.drink);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(3.5, TimerMode::Once);
}

fn npc_to_chat(npc: &mut NPC) {
    npc.behavior = Behavior::Chat;
}

fn npc_start_chat(
    npc: &mut NPC,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
) {
    animation.start_animation(&npc_animations.talk_right);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(6.5, TimerMode::Once);
}

fn npc_to_fight(npc: &mut NPC) {
    npc.behavior = Behavior::Fight;
}

fn npc_start_fight(
    npc: &mut NPC,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
) {
    animation.start_animation(&npc_animations.punch_right);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(2.5, TimerMode::Once);
}

fn npc_to_dance(npc: &mut NPC) {
    npc.behavior = Behavior::Dance;
}

fn npc_start_dance(
    npc: &mut NPC,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
) {
    animation.start_animation(&npc_animations.dance);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(10.5, TimerMode::Once);
}

fn npc_to_cry(npc: &mut NPC) {
    npc.behavior = Behavior::Cry;
}

fn npc_start_cry(
    npc: &mut NPC,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
) {
    animation.start_animation(&npc_animations.cry);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(9.5, TimerMode::Once);
}

fn npc_to_puke(npc: &mut NPC) {
    npc.behavior = Behavior::Puke;
}

fn npc_start_puke(
    npc: &mut NPC,
    animation: &mut AnimationComponent,
    npc_animations: &NPCAnimations,
) {
    animation.start_animation(&npc_animations.puke);
    npc.behavior = Behavior::Idle;
    npc.timer = Timer::from_seconds(6.0, TimerMode::Once);
}

fn all_containers(
    tile_map: &TileMap,
    interactable_query: &Query<(Entity, &Interactable, &Parent)>,
    tile_query: &Query<(&Tile, Option<&Children>)>,
) -> Vec<(IVec2, Entity)> {
    tile_map
        .iter()
        .filter(|(_, entity)| {
            if let Ok((_, tile_children_option)) = tile_query.get(*entity) {
                let Some(tile_children) = tile_children_option else {
                    return false;
                };

                for child in tile_children.iter() {
                    let Ok((_, interactable, _)) = interactable_query.get(*child) else {
                        continue;
                    };
                    if let Interactable::Container(_) = interactable {
                        return true;
                    }
                }
            }
            false
        })
        .collect()
}

fn all_tables(
    tile_map: &TileMap,
    tile_query: &Query<(&Tile, Option<&Children>)>,
) -> Vec<(IVec2, Entity)> {
    tile_map
        .iter()
        .filter(|(_, entity)| {
            if let Ok((tile, _)) = tile_query.get(*entity) {
                if let Tile::Table = tile {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect()
}
