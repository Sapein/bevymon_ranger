use bevy::prelude::Projection::Orthographic;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::RenderLayers;
use bevy_simple_screen_boxing::CameraBox;

pub struct BevymonCameraPlugin;
impl Plugin for BevymonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct GizmoCamera;

fn setup(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::Fixed {
        width: 640.,
        height: 360.,
    };
    commands.spawn((
        Name::from("Main Camera"),
        Camera2d,
        RenderLayers::layer(0),
        Camera {
            order: 2,
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(
                204. / 255.,
                170. / 255.,
                92. / 255.,
            )),
            ..default()
        },
        Orthographic(projection.clone()),
        CameraBox::ResolutionIntegerScale {
            resolution: (640., 360.).into(),
            allow_imperfect_aspect_ratios: false,
        },
    ));

    commands.spawn((
        Name::from("Gizmo Camera"),
        Camera2d,
        GizmoCamera,
        RenderLayers::layer(1),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(
                204. / 255.,
                170. / 255.,
                92. / 255.,
            )),
            ..default()
        },
        Orthographic(projection),
        CameraBox::ResolutionIntegerScale {
            resolution: (640., 360.).into(),
            allow_imperfect_aspect_ratios: false,
        },
    ));
}
