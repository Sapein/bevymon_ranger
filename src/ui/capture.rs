use bevy::prelude::*;
use crate::capture::Health;

pub struct Capture;
impl Plugin for Capture {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, update_health)
            .add_systems(Startup, setup);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct HealthUi;

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::from("HealthUI"),
        Node {
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::left(Val::Px(10.)).with_top(Val::Px(10.)),
            padding: UiRect::left(Val::Px(10.)).with_top(Val::Px(10.)),
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0.066, 0.060, 0.060, 0.624)),
        Text("Health: ".into()),
        HealthUi,
    ));
}

fn update_health(health: Single<&Health, Changed<Health>>, ui: Single<&mut Text, With<HealthUi>>) {
    let mut ui = ui.into_inner();
    let health = health.into_inner().0.to_string();
    let mut ui_text = String::from("Health: ");
    ui_text.push_str(&health);
    
    **ui = ui_text;
}