mod debug;

use crate::devtools::debug::DebugTools;
use bevy::prelude::*;

pub struct Devtools;

impl Plugin for Devtools {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "egui_inspector")]
        app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

        #[cfg(debug_assertions)]
        app.add_plugins(DebugTools);
    }
}
