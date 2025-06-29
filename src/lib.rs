extern crate core;

mod camera;
mod capture;
mod creature;
use crate::camera::BevymonCameraPlugin;
use crate::capture::CapturePlugin;
use crate::creature::CreaturePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Despawn;

pub struct BevymonRangerPlugin;
impl Plugin for BevymonRangerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(avian2d::PhysicsPlugins::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BevymonCameraPlugin)
        .add_plugins(CapturePlugin)
        .add_plugins(CreaturePlugin)
        .add_systems(Last, despawn_entities);
    }
}

fn despawn_entities(mut commands: Commands, entities: Query<Entity, With<Despawn>>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}
