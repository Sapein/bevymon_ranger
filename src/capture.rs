use crate::Despawn;
use bevy::input::common_conditions::{input_just_released, input_pressed};
use bevy::prelude::*;

pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LinePoints>()
            .init_resource::<DrawnPoints>()
            .insert_resource(LineWidth(4.))
            .register_type::<LineWidth>()
            .add_event::<CaptureCircleComplete>()
            .add_systems(
                Update,
                (add_points, detect_complete, connect_points)
                    .chain()
                    .run_if(input_pressed(MouseButton::Left).or(input_pressed(MouseButton::Right))),
            )
            .add_systems(
                Update,
                (clear_points, despawn_capture_lines).chain().run_if(
                    input_just_released(MouseButton::Left).or(input_just_released(
                        MouseButton::Right,
                    )
                    .or(on_event::<CursorLeft>)
                    .or(on_event::<CaptureCircleComplete>)),
                ),
            );
    }
}

#[derive(Resource, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct LinePoints(Vec<Vec2>);

#[derive(Resource, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct DrawnPoints(Vec<Vec2>);

#[derive(Resource, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct LineWidth(f32);

fn despawn_capture_lines(mut commands: Commands, lines: Query<Entity, With<CaptureLine>>) {
    for e in lines {
        commands.entity(e).insert(Despawn);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CaptureLine;

#[derive(Event, Debug)]
struct CaptureCircleComplete;

fn detect_complete(
    width: Res<LineWidth>,
    mut drawn: ResMut<DrawnPoints>,
    mut points: ResMut<LinePoints>,
    mut complete: EventWriter<CaptureCircleComplete>,
) {
    if drawn.is_empty() {
        return;
    }
    let drawn_points = drawn.0.iter().zip(drawn[1..].iter());
    let mut has_intersection = false;
    for (i, first) in drawn_points.clone().enumerate() {
        for second in drawn_points.clone().skip(i) {
            if second.0 == first.1 {
                continue;
            }

            let second_w1 = (&(second.0 + (width.0 / 2.)), &(second.1 + (width.0 / 2.)));
            let second_w2 = (&(second.0 - (width.0 / 2.)), &(second.1 - (width.0 / 2.)));

            let first_w1 = (&(first.0 + (width.0 / 2.)), &(first.1 + (width.0 / 2.)));
            let first_w2 = (&(first.0 - (width.0 / 2.)), &(first.1 - (width.0 / 2.)));

            if intersects(second, first)
                || intersects(second_w1, first_w1)
                || intersects(second_w2, first_w2)
            {
                has_intersection = true;
                complete.write(CaptureCircleComplete);
            }
        }
    }
    if has_intersection {
        drawn.clear();
        points.clear();
    }
}

fn intersects(segment_a: (&Vec2, &Vec2), segment_b: (&Vec2, &Vec2)) -> bool {
    let (x1, y1) = (segment_a.0.x, segment_a.0.y);
    let (x2, y2) = (segment_a.1.x, segment_a.1.y);
    let (x3, y3) = (segment_b.0.x, segment_b.0.y);
    let (x4, y4) = (segment_b.1.x, segment_b.1.y);

    if (x1 == x2) && (y1 == y2) || (x3 == x4) && (y3 == y4) {
        return false;
    }
    let denominator = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    let numerator_a = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
    let numerator_b = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);

    if denominator == 0. {
        return false;
    }

    let ua = numerator_a / denominator;
    let ub = numerator_b / denominator;

    if !(0. ..=1.).contains(&ua) || !(0. ..=1.).contains(&ub) {
        return false;
    }

    true
}

fn connect_points(
    mut commands: Commands,
    lines: Res<LinePoints>,
    width: Res<LineWidth>,
    mut drawn: ResMut<DrawnPoints>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    if lines.is_changed() && !lines.is_empty() {
        let zipped = lines.iter().zip(lines[1..].iter());
        for (start, end) in zipped.clone() {
            if !drawn.contains(start) || !drawn.contains(end) {
                let mut gizmo = GizmoAsset::default();
                gizmo.line_2d(*start, *end, Color::WHITE);

                commands.spawn((
                    Gizmo {
                        handle: gizmo_assets.add(gizmo),
                        line_config: GizmoLineConfig {
                            width: width.0,
                            ..default()
                        },
                        ..default()
                    },
                    CaptureLine,
                ));

                if !drawn.contains(start) {
                    drawn.push(*start);
                }
                if !drawn.contains(end) {
                    drawn.push(*end);
                }
            }
        }
    }
}

fn add_points(
    mut lines: ResMut<LinePoints>,
    mut ev_mouse: EventReader<CursorMoved>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, transform) = camera.into_inner();

    for mouse in ev_mouse.read() {
        let line_pos = camera
            .viewport_to_world(transform, mouse.position)
            .map(|r| r.origin.truncate())
            .unwrap();
        lines.push(line_pos);
    }
}

fn clear_points(mut lines: ResMut<LinePoints>, mut drawn: ResMut<DrawnPoints>) {
    lines.clear();
    drawn.clear();
}
