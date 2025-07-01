use crate::capture::{CaptureFailed, CaptureLineCollision, CaptureProgressChanged};
use crate::creature::{CaptureProgress, CaptureRequirements, Creature};
use crate::Despawn;
use bevy::color;
use bevy::prelude::*;

pub struct CaptureUiPlugin;
impl Plugin for CaptureUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_timer)
            .add_observer(capture_incomplete::<CaptureLineCollision>)
            .add_observer(capture_status_changed);
    }
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
struct TextDisappearTimer(Timer);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CaptureCountText;

fn tick_timer(
    timer: Query<(Entity, &mut TextDisappearTimer), Without<Despawn>>,
    delta_time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timer) in timer {
        timer.tick(delta_time.delta());
        if timer.finished() {
            commands.entity(entity).insert(Despawn);
        }
    }
}

fn capture_incomplete<T>(
    _: Trigger<T>,
    capture_ui: Query<Entity, With<CaptureCountText>>,
    mut commands: Commands,
) {
    for element in capture_ui.iter() {
        commands.entity(element).insert(Despawn);
    }
}

fn capture_status_changed(
    changed: Trigger<CaptureProgressChanged>,
    creatures: Query<
        (
            Entity,
            &CaptureProgress,
            &CaptureRequirements,
            Option<&Children>,
        ),
        With<Creature>,
    >,
    mut existing_ui: Query<
        (&mut Text2d, &mut TextColor, &mut TextDisappearTimer),
        With<CaptureCountText>,
    >,
    mut commands: Commands,
) {
    let (creature, progress, requirements, children) = match creatures.get(changed.0) {
        Ok(creature) => creature,
        Err(_) => return,
    };

    let remaining = requirements.0.saturating_sub(progress.0);

    if let Some(children) = children {
        let (mut text, mut color, mut timer) = existing_ui.get_mut(children[0]).unwrap();
        timer.reset();
        if remaining == 0 {
            *text = Text2d::new("OK");
            *color = TextColor(color::palettes::css::LIGHT_SEA_GREEN.into());
        } else {
            *text = Text2d::from(remaining.to_string());
        }
    } else if remaining == 0 {
        commands.entity(creature).with_child((
            TextDisappearTimer(Timer::from_seconds(10., TimerMode::Once)),
            Text2d::from("OK"),
            TextColor(color::palettes::css::LIGHT_SEA_GREEN.into()),
            CaptureCountText,
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    } else {
        commands.entity(creature).with_child((
            TextDisappearTimer(Timer::from_seconds(10., TimerMode::Once)),
            Text2d::from(remaining.to_string()),
            TextColor(Color::WHITE),
            CaptureCountText,
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    }
}
