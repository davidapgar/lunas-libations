use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::world::{AsTile, Passable, Tile};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(GameState::Playing)))
            .add_system(move_player.in_set(OnUpdate(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: textures.texture_logo.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.5)),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            ..Default::default()
        })
        .insert(Player);
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
        let new_tile = new_translation.as_tile();
        if new_tile == player_transform.translation.as_tile() {
            player_transform.translation += movement;
            continue;
        }

        for (transform, passable) in &world_query {
            let tile = transform.translation.as_tile();
            if tile == new_tile {
                if let Passable::Passable = passable.0 {
                    player_transform.translation += movement;
                    player_transform.translation.z = tile.z;
                }
                break;
            }
        }
    }
}
