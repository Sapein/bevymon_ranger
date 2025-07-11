extern crate core;

mod camera;
mod capture;
mod creature;
mod devtools;
mod ui;
mod overworld;
mod movement;

use crate::camera::BevymonCameraPlugin;
use bevy::prelude::*;
use crate::movement::MovementPlugin;
use crate::overworld::OverworldPlugin;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Despawn;

pub struct BevymonRangerPlugin;
impl Plugin for BevymonRangerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian2d::PhysicsPlugins::default())
            .add_plugins(BevymonCameraPlugin)
            .add_plugins(OverworldPlugin)
            .add_plugins(MovementPlugin)
            .add_systems(Last, despawn_entities);

        #[cfg(feature = "devtools")]
        app.add_plugins(devtools::Devtools);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[relationship(relationship_target = DespawnChildren)]
pub(crate) struct DespawnWith(Entity);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[relationship_target(relationship = DespawnWith, linked_spawn)]
pub(crate) struct DespawnChildren(Vec<Entity>);
fn despawn_entities(mut commands: Commands, entities: Query<Entity, With<Despawn>>) {
    for entity in entities {
        commands.entity(entity).despawn_related::<DespawnChildren>();
        commands.entity(entity).despawn()
    }
}
