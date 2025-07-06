use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};

pub struct OverworldPlugin;
impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
       app.add_plugins(LdtkPlugin)
           .insert_resource(LevelSelection::index(0))
           .add_systems(Startup, ldtk_setup);
        
    }
}

fn ldtk_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("overworld.ldtk").into(),
        transform: Transform::from_xyz(-128., -128., 0.0),
        ..default()
    });
}