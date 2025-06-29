use crate::physics::{Speed, Velocity};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Velocity, Speed)]
pub struct Player;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_icon = asset_server.load("TempSprite.png");
    commands.spawn((
        Player,
        Sprite::from_image(player_icon),
        Speed(200.0),
        children![Sprite::from_image(hitbox_icon)],
    ));
}

fn movement(mut player: Single<&mut Velocity, With<Player>>, mut keys: EventReader<KeyboardInput>) {
    for key in keys.read() {
        if key.key_code == KeyCode::ArrowUp {
            if key.state == ButtonState::Released {
                player.directional.y = 0.;
            } else {
                player.directional.y = 1.;
            }
        } else if key.key_code == KeyCode::ArrowDown {
            if key.state == ButtonState::Released {
                player.directional.y = 0.;
            } else {
                player.directional.y = -1.;
            }
        } else if key.key_code == KeyCode::ArrowLeft {
            if key.state == ButtonState::Released {
                player.directional.x = 0.;
            } else {
                player.directional.x = -1.;
            }
        } else if key.key_code == KeyCode::ArrowRight {
            if key.state == ButtonState::Released {
                player.directional.x = 0.;
            } else {
                player.directional.x = 1.;
            }
        }
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, movement);
    }
}
