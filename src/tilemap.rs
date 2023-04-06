use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct TileMap {
    /// Size in tiles
    pub size: IVec2,
    /// Size of tiles in pixels
    pub tile_size: IVec2,
    tiles: Vec<Option<Entity>>,
}

impl TileMap {
    pub fn new(size: IVec2, tile_size: IVec2) -> Self {
        let tiles = vec![None; (size.x * size.y) as usize];
        TileMap {
            size,
            tile_size,
            tiles,
        }
    }

    pub fn insert(
        &mut self,
        tilemap_entity: Entity,
        entity: Entity,
        location: IVec2,
        commands: &mut Commands,
    ) {
        if location.x < 0 || location.x >= self.size.x {
            return;
        }
        if location.y < 0 || location.y >= self.size.y {
            return;
        }

        let tile_index = (location.x + (location.y * self.size.x)) as usize;
        if let Some(existing_tile) = self.tiles[tile_index] {
            // Remove this tile and despawn.
            commands.entity(existing_tile).remove_parent().despawn();
        }
        // Add this child
        commands.entity(tilemap_entity).add_child(entity);
        self.tiles[tile_index] = Some(entity);
    }

    fn transform_tiles(&self, commands: &Commands, query: &mut Query<&mut Transform>) {
        let height = self.size.y * self.tile_size.y;

        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let tile_index = (x + (y * self.size.x)) as usize;
                if let Some(tile) = self.tiles[tile_index] {
                    if let Ok(mut tile_transform) = query.get_mut(tile) {
                        let position = Vec3::new(
                            (x * self.tile_size.x) as f32,
                            (height - y * self.tile_size.y) as f32,
                            y as f32,
                        );
                        tile_transform.translation = position;
                    }
                }
            }
        }
    }
}
