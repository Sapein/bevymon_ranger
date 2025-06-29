use bevy::prelude::*;

pub struct CreaturePlugin;
impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .add_systems(Startup, spawn_enemy);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Enemy;

fn spawn_enemy(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_icon = asset_server.load("TempEnemy.png");
    commands.spawn((
        Name::from("Test Enemy"),
        Enemy,
        Sprite::from_image(enemy_icon),
    ));
}
