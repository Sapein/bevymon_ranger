use crate::creature::{Attack, CaptureProgress, CaptureRequirements, Creature};
use avian2d::position::Rotation;
use avian2d::prelude::{Collider, Collisions};
use bevy::input::common_conditions::{input_just_released, input_pressed};
use bevy::math::VectorSpace;
use bevy::prelude::*;

pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CaptureCircleComplete>()
            .add_event::<CaptureLineCollision>()
            .add_event::<CaptureLineLifted>()
            .add_event::<TakeDamage>()
            .add_event::<CaptureFailed>()
            .add_event::<CaptureSuccess>()
            .register_type::<CaptureLine>()
            .register_type::<Health>()
            .add_systems(Startup, setup)
            .add_systems(Update, adjust_linewidth)
            .add_systems(Update, (take_damage, is_dead).chain())
            .add_systems(
                Update,
                (
                    add_points,
                    detect_capture_collision,
                    detect_complete.run_if(not(on_event::<CaptureLineCollision>)),
                    increase_capture_progress.run_if(on_event::<CaptureCircleComplete>),
                    connect_points,
                )
                    .chain()
                    .run_if(input_pressed(MouseButton::Left).or(input_pressed(MouseButton::Right))),
            )
            .add_systems(Last, shorten_line.run_if(on_event::<CaptureCircleComplete>))
            .add_systems(
                Update,
                emit_capture_events.run_if(
                    input_just_released(MouseButton::Left)
                        .or(input_just_released(MouseButton::Right)),
                ),
            )
            .add_systems(
                Update,
                (destroy_line, reset_capture_progress)
                    .after(emit_capture_events)
                    .chain()
                    .run_if(
                        input_just_released(MouseButton::Left).or(input_just_released(
                            MouseButton::Right,
                        )
                        .or(on_event::<CursorLeft>)
                        .or(on_event::<CaptureLineCollision>)),
                    ),
            );
    }
}

fn detect_capture_collision(
    capture_line: Single<Entity, With<CaptureLine>>,
    collisions: Collisions,
    mut collision: EventWriter<CaptureLineCollision>,
    mut damage: EventWriter<TakeDamage>,
    damagable: Query<&Damage>,
) {
    let capture_line = capture_line.into_inner();
    if let Some(collide) = collisions
        .collisions_with(capture_line)
        .next()
    {
        let actual_collider = if collide.collider1 == capture_line {
            collide.collider2
        } else {
            collide.collider1
        };
        
        if let Ok(d) = damagable.get(actual_collider) {
            damage.write(TakeDamage(d.0));
        }
        
        collision.write(CaptureLineCollision);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((CaptureLine::default(),Health(4)));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Health(u32);

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
    width: f32,
}
impl Default for CaptureLine {
    fn default() -> Self {
        Self {
            line: vec![],
            width: 4.0,
            start_color: Some(Color::linear_rgb(0.168_627_46, 0.211_764_71, 0.529_411_8)),
            end_color: Some(Color::linear_rgb(0.411_764_7, 0.478_431_37, 0.980_392_16)),
        }
    }
}

fn take_damage(capture_line: Single<&mut Health>, mut damage_event: EventReader<TakeDamage>) {
    let mut capture_line = capture_line.into_inner();
    for damage in damage_event.read() {
        capture_line.0 -= damage.0;
    }
}

fn is_dead(capture_line: Single<&mut Health>) {
    let capture_line = capture_line.into_inner();
    if capture_line.0 <= 0 {
        println!("Game Over");
    }
}

#[derive(Event, Debug)]
struct CaptureCircleComplete {
    intersecting_point_a: (usize, (Vec2, Vec2)),
}

#[derive(Event, Debug)]
struct CaptureLineCollision;

#[derive(Event, Debug)]
struct TakeDamage(u32);

#[derive(Event, Debug)]
struct CaptureLineLifted;

#[derive(Event, Debug)]
pub struct CaptureFailed(pub Entity);

#[derive(Event, Debug)]
pub struct CaptureSuccess {
    pub captured: Entity,
    pub overshot_by: usize,
}

fn detect_complete(line: Single<&CaptureLine>, mut complete: EventWriter<CaptureCircleComplete>) {
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
                complete.write(CaptureCircleComplete {
                    intersecting_point_a: (i, (*first.0, *first.1)),
                });
            }
        }
    }
}

fn increase_capture_progress(
    capture_line: Single<(&CaptureLine, &Collider)>,
    creatures: Query<(&mut CaptureProgress, &Transform), Without<Captured>>,
) {
    let (line, our_collider) = capture_line.into_inner();
    for (mut progress, creature_location) in creatures {
        let our_collider = our_collider.shape().as_polyline().unwrap();
        let polygon =
            Collider::convex_decomposition(line.line.clone(), our_collider.indices().to_vec());
        if polygon.contains_point(
            Vec2::ZERO,
            Rotation::IDENTITY,
            creature_location.translation.xy(),
        ) {
            progress.0 += 1;
        }
    }
}

fn intersects(segment_a: (&Vec2, &Vec2), segment_b: (&Vec2, &Vec2)) -> Option<Vec2> {
    let (x1, y1) = (segment_a.0.x, segment_a.0.y);
    let (x2, y2) = (segment_a.1.x, segment_a.1.y);
    let (x3, y3) = (segment_b.0.x, segment_b.0.y);
    let (x4, y4) = (segment_b.1.x, segment_b.1.y);

    if (x1 == x2) && (y1 == y2) || (x3 == x4) && (y3 == y4) {
        return None;
    }
    let denominator = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    let numerator_a = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
    let numerator_b = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);

    if denominator == 0. {
        return None;
    }

    let ua = numerator_a / denominator;
    let ub = numerator_b / denominator;

    if !(0. ..=1.).contains(&ua) || !(0. ..=1.).contains(&ub) {
        return None;
    }

    Some(Vec2::new(x1 + ua * (x2 - x1), y1 + ua * (y2 - y1)))
}

fn adjust_linewidth(mut config_store: ResMut<GizmoConfigStore>, lines: Single<&CaptureLine>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.joints = GizmoLineJoint::Round(1);
    config.line.width = lines.width;
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

fn add_points(
    mut ev_mouse: EventReader<CursorMoved>,
    camera: Single<(&Camera, &GlobalTransform)>,
    lines: Single<(Entity, &mut CaptureLine)>,
    mut commands: Commands,
) {
    let (camera, transform) = camera.into_inner();

    let (e, mut line) = lines.into_inner();
    for mouse in ev_mouse.read() {
        let line_pos = camera
            .viewport_to_world(transform, mouse.position)
            .map(|r| r.origin.truncate())
            .unwrap();

        line.line.push(line_pos);
        if line.line.len() >= 2 {
            commands
                .entity(e)
                .insert((Collider::polyline(line.line.clone(), None),));
        }
    }
}

fn shorten_line(
    lines: Single<(Entity, &mut CaptureLine)>,
    mut commands: Commands,
    mut event_reader: EventReader<CaptureCircleComplete>,
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

        let (point_a1, point_a2) = points[complete.intersecting_point_a.0];
        if point_a1.1 == &complete.intersecting_point_a.1 .0
            && point_a2.1 == &complete.intersecting_point_a.1 .1
        {
            lines.line.truncate(point_a1.0);
            commands
                .entity(e)
                .insert(Collider::polyline(lines.line.clone(), None));
        }
    }
}

fn destroy_line(lines: Single<(Entity, &mut CaptureLine)>, mut commands: Commands) {
    let (e, mut lines) = lines.into_inner();
    lines.line.clear();
    commands.entity(e).remove_with_requires::<Collider>();
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
        } else {
            failed_capture_event.write(CaptureFailed(entity));
        }
    }
}
