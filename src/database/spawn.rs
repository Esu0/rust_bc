#![allow(dead_code)]

use bevy::prelude::*;

use super::animation::UnitSelector;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SpawnUnitSet {
    Prepare,
    Spawn,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct DummyUnit {
    selector: UnitSelector,
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct TempId {
    id: Entity,
}
pub fn spawn_unit(commands: &mut Commands, selector: UnitSelector, transform: Transform) {
    let id = commands.spawn((DummyUnit { selector }, transform)).id();
    commands.entity(id).insert(TempId {id});
}

fn replace_dummy(mut commands: Commands, query: Query<(&DummyUnit, &TempId, &Transform)>) {
    for (dummy_unit, id, transform) in &query {
        
        // spawning character
        
        
        let id = id.id;
        commands.entity(id).despawn();
    }
}