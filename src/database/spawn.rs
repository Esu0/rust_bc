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
pub fn spawn_unit(commands: &mut Commands, selector: UnitSelector) {
    let id = commands.spawn(DummyUnit { selector }).id();
    commands.entity(id).insert(TempId {id});
}

fn replace_dummy(mut commands: Commands, query: Query<(&DummyUnit, &TempId)>) {
    for (dummy_unit, id) in &query {
        
        // spawning character
        
        let id = id.id;
        commands.entity(id).despawn();
    }
}