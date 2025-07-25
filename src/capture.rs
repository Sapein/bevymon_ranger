mod math;
mod ui;

use crate::camera::GizmoCamera;
use crate::capture::math::{intersects, length};
use crate::capture::ui::CaptureUiPlugin;
use crate::creature::{CaptureProgress, CaptureRequirements};
use crate::{Despawn, DespawnWith};
use avian2d::position::Rotation;
use avian2d::prelude::{Collider, Collisions};
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;

pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CaptureLineConnected>()
            .add_event::<CaptureLineCollision>()
            .add_event::<CapturePointLifted>()
            .add_event::<CapturePointPressed>()
            .add_event::<TakeDamage>()
            .add_event::<CaptureFailed>()
            .add_event::<CaptureSuccess>()
            .register_type::<CaptureLine>()
            .register_type::<Health>()
            .register_type::<Assets>()
            .init_resource::<Assets>()
            .add_plugins(CaptureUiPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, adjust_linewidth)
            .add_systems(Update, take_damage)
            .add_systems(
                Update,
                player_start_capture.run_if(
                    input_just_pressed(MouseButton::Left)
                        .or(input_just_pressed(MouseButton::Right)),
                ),
            )
            .add_systems(
                Update,
                player_stop_capture.run_if(
                    input_just_released(MouseButton::Left)
                        .or(input_just_released(MouseButton::Right)),
                ),
            )
            .add_systems(
                Update,
                (
                    add_points_to_capture_line,
                    trail_line,
                    follow_styler,
                    detect_capture_collision,
                    detect_complete.run_if(not(on_event::<CaptureLineCollision>)),
                    increase_capture_progress.run_if(on_event::<CaptureLineConnected>),
                    connect_points,
                )
                    .chain()
                    .run_if(any_with_component::<CaptureLine>),
            )
            .add_systems(
                Last,
                truncate_capture_line_to_intersection.run_if(on_event::<CaptureLineConnected>),
            )
            .add_systems(
                Update,
                emit_capture_events.run_if(on_event::<CapturePointLifted>),
            )
            .add_systems(
                Update,
                (destroy_line, reset_capture_progress)
                    .after(emit_capture_events)
                    .chain()
                    .run_if(
                        on_event::<CapturePointLifted>
                            .or(on_event::<CursorLeft>)
                            .or(on_event::<CaptureLineCollision>),
                    ),
            );
    }
}

fn detect_capture_collision(
    mut commands: Commands,
    capture_line: Single<Entity, With<CaptureLine>>,
    capture_start: Single<Entity, With<CaptureLineStart>>,
    collisions: Collisions,
    mut collision_event: EventWriter<CaptureLineCollision>,
    mut damage: EventWriter<TakeDamage>,
    damagable: Query<&Damage>,
) {
    let capture_line = capture_line.into_inner();
    let capture_start = capture_start.into_inner();
    for collision in collisions.collisions_with(capture_line) {
        let actual_collider = if collision.collider1 == capture_line {
            collision.collider2
        } else {
            collision.collider1
        };

        if actual_collider == capture_start {
            continue;
        }

        if let Ok(d) = damagable.get(actual_collider) {
            damage.write(TakeDamage(d.0));
            commands.trigger(TakeDamage(d.0));
        }

        collision_event.write(CaptureLineCollision);
        commands.trigger(CaptureLineCollision);
    }
}

fn setup(asset_server: Res<AssetServer>, mut assets: ResMut<Assets>, mut commands: Commands) {
    assets.styler = asset_server.load("Capture-Styler.png");
    assets.styler_start = asset_server.load("captureline-start2.png");
    commands.spawn(Health(4));
}

/// Represents when the user deliberately stops a capture
///
/// (IE: Lifting their finger off of the screen, releasing the mouse, etc.)
#[derive(Event, Debug)]
pub struct CapturePointLifted;

/// Represents when the user 'starts' a capture line.
#[derive(Event, Debug)]
pub struct CapturePointPressed;

/// Represents when a capture failed
///
/// Whether it was 'ending' the capture too early, regardless of reason and includes the Entity
/// that was attempted to be captured.
#[derive(Event, Debug)]
pub struct CaptureFailed(pub Entity);

/// Represents when a Capture has succeeded.
#[derive(Event, Debug)]
pub struct CaptureSuccess {
    /// The entity that was captured.
    pub captured: Entity,

    /// The amount the 'overshot' occurred by.
    pub overshot_by: usize,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CaptureLineStart;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CaptureStyler;

#[derive(Event, Debug)]
pub struct CaptureProgressChanged(pub Entity);

#[derive(Event, Debug)]
struct CaptureLineConnected {
    cull_to: (usize, (Vec2, Vec2)),
}

#[derive(Event, Debug)]
struct CaptureLineCollision;

#[derive(Event, Debug)]
struct TakeDamage(u32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Health(pub(crate) u32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Damage(pub u32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Captured;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CaptureLine {
    line: Vec<Vec2>,
    start_color: Option<Color>,
    end_color: Option<Color>,
    max_line_length: Option<usize>,
    width: f32,
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
struct Assets {
    styler: Handle<Image>,
    styler_start: Handle<Image>,
}

impl Default for CaptureLine {
    fn default() -> Self {
        Self {
            line: vec![],
            width: 10.0,
            start_color: Some(Color::linear_rgb(0.168_627_46, 0.211_764_71, 0.529_411_8)),
            end_color: Some(Color::linear_rgb(0.411_764_7, 0.478_431_37, 0.980_392_16)),
            max_line_length: Some(500),
        }
    }
}

fn take_damage(capture_line: Single<&mut Health>, mut damage_event: EventReader<TakeDamage>) {
    let mut capture_line = capture_line.into_inner();
    for damage in damage_event.read() {
        capture_line.0 -= damage.0;
    }
}

fn detect_complete(line: Single<&CaptureLine>, mut complete: EventWriter<CaptureLineConnected>) {
    if line.line.is_empty() {
        return;
    }

    let points = line.line.iter().zip(line.line[1..].iter());
    for (i, first) in points.clone().enumerate() {
        for second in points.clone().skip(i) {
            if second.0 == first.1 {
                continue;
            }

            let second_w1 = (
                &(second.0 + (line.width / 2.)),
                &(second.1 + (line.width / 2.)),
            );
            let second_w2 = (
                &(second.0 - (line.width / 2.)),
                &(second.1 - (line.width / 2.)),
            );

            let first_w1 = (
                &(first.0 + (line.width / 2.)),
                &(first.1 + (line.width / 2.)),
            );
            let first_w2 = (
                &(first.0 - (line.width / 2.)),
                &(first.1 - (line.width / 2.)),
            );

            if intersects(second, first).is_some()
                || intersects(second_w1, first_w1).is_some()
                || intersects(second_w2, first_w2).is_some()
            {
                complete.write(CaptureLineConnected {
                    cull_to: (i, (*first.0, *first.1)),
                });
            }
        }
    }
}

fn increase_capture_progress(
    mut commands: Commands,
    capture_line: Single<(&CaptureLine, &Collider)>,
    creatures: Query<(Entity, &mut CaptureProgress, &Transform), Without<Captured>>,
) {
    let (line, our_collider) = capture_line.into_inner();
    for (entity, mut progress, creature_location) in creatures {
        let our_collider = our_collider.shape().as_polyline().unwrap();
        let polygon =
            Collider::convex_decomposition(line.line.clone(), our_collider.indices().to_vec());

        if polygon.contains_point(
            Vec2::ZERO,
            Rotation::IDENTITY,
            creature_location.translation.xy(),
        ) {
            progress.0 += 1;
            commands.trigger(CaptureProgressChanged(entity));
        }
    }
}

fn adjust_linewidth(mut config_store: ResMut<GizmoConfigStore>, lines: Single<&CaptureLine>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.joints = GizmoLineJoint::Round(1);
    config.line.width = lines.width;
    config.render_layers = RenderLayers::layer(1);
}

fn connect_points(lines: Single<&CaptureLine>, mut gizmos: Gizmos) {
    if !lines.line.is_empty() {
        if lines.start_color.is_none() || lines.end_color.is_none() {
            let color = match (lines.start_color, lines.end_color) {
                (Some(c), None) => c,
                (None, Some(c)) => c,
                _ => Color::WHITE,
            };
            gizmos.linestrip_2d(lines.line.clone(), color);
        } else {
            let start_color = lines.start_color.unwrap().to_linear();
            let end_color = lines.end_color.unwrap().to_linear();
            let lines_count = lines.line.len() as f32;
            let lines = lines.line.clone().into_iter().enumerate().map(|(i, v)| {
                (
                    v,
                    Color::LinearRgba(start_color.lerp(end_color, i as f32 / lines_count)),
                )
            });
            gizmos.linestrip_gradient_2d(lines);
        }
    }
}

fn add_points_to_capture_line(
    mut ev_mouse: EventReader<CursorMoved>,
    camera: Single<(&Camera, &GlobalTransform), Without<GizmoCamera>>,
    lines: Single<(Entity, &mut CaptureLine)>,
    mut commands: Commands,
) {
    let (camera, transform) = camera.into_inner();

    let (e, mut line) = lines.into_inner();
    for mouse in ev_mouse.read() {
        let line_pos = camera
            .viewport_to_world_2d(transform, mouse.position)
            .expect("Unable to get world coordinates from viewport!");

        if let Some(line_max) = line.max_line_length {
            let line_max = line_max as f32;
            if line.line.len() >= 2 {
                let mut length: f32 = line
                    .line
                    .iter()
                    .zip(line.line[1..].iter())
                    .map(length)
                    .sum();
                while length > line_max {
                    line.line.remove(0);
                    length = line
                        .line
                        .iter()
                        .zip(line.line[1..].iter())
                        .map(math::length)
                        .sum();
                }
            }
        }

        line.line.push(line_pos);

        if line.line.len() >= 2 {
            commands
                .entity(e)
                .insert((Collider::polyline(line.line.clone(), None),));
        }
    }
}

fn truncate_capture_line_to_intersection(
    lines: Single<(Entity, &mut CaptureLine)>,
    mut commands: Commands,
    mut event_reader: EventReader<CaptureLineConnected>,
) {
    let (e, mut lines) = lines.into_inner();
    for complete in event_reader.read() {
        let points_2 = lines.line.iter().enumerate().collect::<Vec<_>>();
        let points = lines.line.clone();
        let points = points
            .iter()
            .enumerate()
            .zip(points_2[1..].iter())
            .collect::<Vec<_>>();

        if points.len() <= complete.cull_to.0 {
            continue;
        }
        let (point_a1, point_a2) = points[complete.cull_to.0];
        if point_a1.1 == &complete.cull_to.1 .0 && point_a2.1 == &complete.cull_to.1 .1 {
            lines.line.truncate(point_a1.0);
            if lines.line.len() >= 2 {
                commands
                    .entity(e)
                    .insert(Collider::polyline(lines.line.clone(), None));
            }
        }
    }
}

fn follow_styler(
    line: Single<&CaptureLine>,
    capture_styler: Single<&mut Transform, With<CaptureStyler>>,
) {
    let line = line.into_inner();
    let mut capture_styler = capture_styler.into_inner();

    if line.line.is_empty() {
        return;
    }

    if line.line.last().unwrap() != &capture_styler.translation.xy() {
        capture_styler.translation = line.line.last().unwrap().extend(0.);
    }
}

fn trail_line(
    line: Single<&CaptureLine>,
    capture_start: Single<&mut Transform, With<CaptureLineStart>>,
) {
    let line = line.into_inner();
    let mut capture_start = capture_start.into_inner();
    if line.line.is_empty() {
        return;
    }

    if line.line[0] != capture_start.translation.xy() {
        capture_start.translation = line.line[0].extend(capture_start.translation.z);
    }
}

fn destroy_line(line: Single<Entity, With<CaptureLineStart>>, mut commands: Commands) {
    let e = line.into_inner();
    commands.entity(e).insert(Despawn);
}

fn reset_capture_progress(creature_progress: Query<&mut CaptureProgress, Without<Captured>>) {
    for mut progress in creature_progress {
        progress.0 = 0;
    }
}

fn emit_capture_events(
    mut commands: Commands,
    creatures: Query<(Entity, &CaptureProgress, &CaptureRequirements), Without<Captured>>,
    mut capture_event: EventWriter<CaptureSuccess>,
    mut failed_capture_event: EventWriter<CaptureFailed>,
) {
    for (entity, progress, requirements) in creatures.iter() {
        if progress.0 >= requirements.0 {
            capture_event.write(CaptureSuccess {
                captured: entity,
                overshot_by: (progress.0 - requirements.0) as usize,
            });
            commands.entity(entity).insert(Captured);
            commands.trigger(CaptureSuccess {
                captured: entity,
                overshot_by: (progress.0 - requirements.0) as usize,
            });
        } else {
            failed_capture_event.write(CaptureFailed(entity));
            commands.trigger(CaptureFailed(entity));
        }
    }
}

fn player_start_capture(
    mut commands: Commands,
    mut event_writer: EventWriter<CapturePointPressed>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), Without<GizmoCamera>>,
    assets: Res<Assets>,
) {
    event_writer.write(CapturePointPressed);

    let window = window.into_inner();
    let (camera, camera_transform) = camera.into_inner();
    let current_point = camera
        .viewport_to_world_2d(camera_transform, window.cursor_position().unwrap())
        .expect("Unable to calculate cursor position!");

    let mut sprite = Sprite::from_image(assets.styler_start.clone_weak());
    sprite.custom_size = Some(Vec2::new(9., 9.));
    let parent = commands
        .spawn((
            CaptureLineStart,
            sprite,
            Transform::from_translation(current_point.extend(0.)),
        ))
        .id();

    commands.spawn((
        CaptureLine {
            line: vec![current_point],
            width: 12.,
            ..default()
        },
        DespawnWith(parent),
    ));

    commands.spawn((
        CaptureStyler,
        Sprite::from_image(assets.styler.clone_weak()),
        Transform::from_translation(current_point.extend(-1.)),
        DespawnWith(parent),
    ));
}

fn player_stop_capture(mut event_writer: EventWriter<CapturePointLifted>) {
    event_writer.write(CapturePointLifted);
}
