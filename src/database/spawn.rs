#![allow(dead_code)]

use bevy::prelude::*;

use super::animation::{UnitImages, UnitSelector, UnitSpriteIds, Unit, UnitSpritePartParent, UnitSpriteId, UnitImage};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SpawnUnitSet {
    Prepare,
    Spawn,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct DummyUnit {
    id: LocalUnitId,
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct TempId {
    id: Entity,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalUnitId {
    id: usize,
}

pub fn spawn_unit(commands: &mut Commands, id: LocalUnitId, transform: Transform) {
    let id = commands.spawn((DummyUnit { id }, transform)).id();
    commands.entity(id).insert(TempId { id });
}

fn replace_dummy(
    mut commands: Commands,
    query: Query<(&DummyUnit, &TempId, &Transform)>,
    images: Res<UnitImages>,
    mut ids: ResMut<UnitSpriteIds>,
) {
    for (dummy_unit, id, transform) in &query {
        // spawning character
        let UnitImage {
            materials: material_handles,
            meshes: mesh_handles,
            size: sizes,
            mamodels,
        } = images.images[dummy_unit.id.id];
        commands.spawn((Unit, SpatialBundle {
            transform: transform.clone(),
            ..default()
        })).with_children(|parent| {
            let ids = Vec::new();
            
            parent.spawn((UnitSpritePartParent, SpatialBundle::default()))
        })
        
        let id = id.id;
        commands.entity(id).despawn();
    }
}
