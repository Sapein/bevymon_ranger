use bevy::input::common_conditions::{input_just_released, input_pressed};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::prelude::*;
use crate::camera::GizmoCamera;
use crate::movement::{MovementVector, Speed};

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Player;

#[derive(Bundle, Default, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    
    #[with(player_speed)]
    speed: Speed,

    #[sprite_sheet]
    sprite_sheet: Sprite,
    
    #[worldly]
    worldly: Worldly,
}

fn player_speed(instance: &EntityInstance) -> Speed {
    match instance.field_instances.iter().find(|f| f.identifier.eq_ignore_ascii_case("speed")) {
        None => Speed(10.),
        Some(t) => {
            Speed(match t.value {
                FieldValue::Float(Some(f)) => f,
                _ => 10.
            })
        }
    }
}

fn to_cursor_pos(
    camera: Single<(&Camera, &GlobalTransform), Without<GizmoCamera>>,
    player: Single<(&mut MovementVector, &GlobalTransform), With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let (camera, gt) = camera.into_inner();
    let window = window.into_inner();
    let (mut player, transform) = player.into_inner();
    let cursor_pos = match window.cursor_position() {
        None => return,
        Some(pos) => pos,
    };
    
    let cursor_pos = camera
        .viewport_to_world_2d(gt, cursor_pos)
        .expect("Unable to get world coordinates from viewport!");
    
    player.0 = cursor_pos - transform.translation().xy();
    
    if player.0.x.abs() < 1. && player.0.y.abs() < 1. {
        player.0 = Vec2::ZERO;
    }
}

fn clear_movement(
    player: Single<(&mut MovementVector), With<Player>>
) {
    player.into_inner().0 = Vec2::ZERO;
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(First, clear_movement.run_if(not(input_pressed(MouseButton::Left))))
            .add_systems(Last, to_cursor_pos.run_if(input_pressed(MouseButton::Left)));
    }
}
