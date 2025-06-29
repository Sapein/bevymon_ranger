use avian2d::prelude::Collider;
use bevy::prelude::*;

pub struct CreaturePlugin;
impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .register_type::<CaptureProgress>()
            .register_type::<CaptureRequirements>()
            .add_systems(Startup, spawn_enemy);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Enemy;

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct CaptureRequirements(pub u32);

#[derive(Component, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct CaptureProgress(pub u32);

fn spawn_enemy(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_icon = asset_server.load("TempEnemy.png");
    commands.spawn((
        Name::from("Testmon"),
        CaptureProgress::default(),
        Enemy,
        CaptureRequirements(3),
        Collider::rectangle(32., 32.),
        Sprite::from_image(enemy_icon),
    ));
}
