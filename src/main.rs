use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevymon_ranger::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevymon_ranger".into(),
                        name: Some("bevymon_ranger".into()),

                        // This allows i3wm to force it into floating.
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    filter: "info,wgpu_core=off,wgpu_hal=off,mygame=debug".into(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                }),
        )
        .add_plugins(BevymonRangerPlugin)
        .run();
}
