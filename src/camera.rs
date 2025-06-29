use bevy::prelude::Projection::Orthographic;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_simple_screen_boxing::CameraBox;

pub struct BevymonCameraPlugin;
impl Plugin for BevymonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::Fixed {
        width: 640.,
        height: 360.,
    };
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Orthographic(projection),
        CameraBox::ResolutionIntegerScale {
            resolution: (640., 360.).into(),
            allow_imperfect_aspect_ratios: false,
        },
    ));
}
