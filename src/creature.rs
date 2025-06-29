use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use crate::capture::Damage;

pub struct CreaturePlugin;
impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Creature>()
            .register_type::<CaptureProgress>()
            .register_type::<CaptureRequirements>()
            .add_systems(Startup, spawn_enemy);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Creature;

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct CaptureRequirements(pub u32);

#[derive(Component, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct CaptureProgress(pub u32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct TestAttack(u32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Attack;

fn spawn_enemy(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_icon = asset_server.load("TempEnemy.png");
    commands.spawn((
        Name::from("Testmon"),
        CaptureProgress::default(),
        Creature,
        TestAttack(1),
        CaptureRequirements(3),
        Collider::rectangle(32., 32.),
        Sprite::from_image(enemy_icon),
    ));
}

fn attack(mut commands: Commands, asset_server: Res<AssetServer>, query: Single<&TestAttack>) {
    let attack = asset_server.load("round_bullet.png");
    let attack_hurt = query.into_inner();
    commands.spawn((
        Damage(attack_hurt.0),
        Attack,
        Collider::circle(32./2.),
        Sprite::from_image(attack),
        RigidBody::Dynamic,
    ));
}