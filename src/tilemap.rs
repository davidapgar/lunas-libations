use bevy::prelude::*;

#[derive(Component)]
pub struct TileMap {
    /// Size in tiles
    pub size: IVec2,
    /// Size of tiles in pixels
    pub tile_size: IVec2,
    pub scale: IVec2,
    tiles: Vec<Option<Entity>>,
}

impl TileMap {
    pub fn new(size: IVec2, tile_size: IVec2, scale: IVec2) -> Self {
        let tiles = vec![None; (size.x * size.y) as usize];
        TileMap {
            size,
            tile_size,
            scale,
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

    pub fn transform_tiles(&self, query: &mut Query<&mut Transform, Without<TileMap>>) {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let tile_index = (x + (y * self.size.x)) as usize;
                if let Some(tile) = self.tiles[tile_index] {
                    if let Ok(mut tile_transform) = query.get_mut(tile) {
                        let position = Vec3::new(
                            (x * self.tile_size.x) as f32,
                            (y * self.tile_size.y) as f32,
                            self.tile_z(&IVec2::new(x, y)),
                        );
                        tile_transform.translation = position;
                    }
                }
            }
        }
    }

    /// Z-level of passed tile.
    pub fn tile_z(&self, tile: &IVec2) -> f32 {
        (self.size.y - tile.y) as f32
    }

    pub fn to_tile_space(&self, origin: Vec3, point: Vec3) -> Vec3 {
        let mut result = point - origin;
        result.z = point.z;
        result
    }

    // Convert tile space point to tile x,y
    pub fn to_tile(&self, point: Vec3) -> IVec2 {
        // tile space is 0,0 bottom left
        // tile index is 0,0 bottom left
        // index y = space 0, index 0 = space y
        let scaled = IVec2::new(point.x as i32 / self.scale.x, point.y as i32 / self.scale.y);
        IVec2::new(scaled.x / self.tile_size.x, scaled.y / self.tile_size.y)
    }

    pub fn camera_to_tile(&self, origin: Vec3, point: Vec3) -> IVec2 {
        self.to_tile(self.to_tile_space(origin, point))
    }

    pub fn tile_at(&self, point: IVec2) -> Option<Entity> {
        if point.x < 0 || point.x >= self.size.x {
            None
        } else if point.y < 0 || point.y >= self.size.y {
            None
        } else {
            self.tiles[self.tile_index(point)]
        }
    }

    fn tile_index(&self, point: IVec2) -> usize {
        (point.x + (point.y * self.size.x)) as usize
    }
}
