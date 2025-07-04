use crate::capture::Health;
use bevy::prelude::*;

pub struct Capture;
impl Plugin for Capture {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_health)
            .add_systems(Startup, setup);
    }
}

const BACKGROUND_COLOR: Color = Color::linear_rgba(0.066, 0.060, 0.060, 0.624);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Node, BackgroundColor(BACKGROUND_COLOR), Text("Seeing this is an error".into()), Name = Name::from("Health UI"))]
struct HealthUi;

fn setup(mut commands: Commands) {
    commands.spawn(HealthUi);
}

fn update_health(health: Single<&Health, Changed<Health>>, ui: Single<&mut Text, With<HealthUi>>) {
    let mut ui = ui.into_inner();
    let health = health.into_inner().0.to_string();
    let mut ui_text = String::from("Health: ");
    ui_text.push_str(&health);

    **ui = ui_text;
}
