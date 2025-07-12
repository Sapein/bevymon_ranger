use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Deref, DerefMut, Default)]
#[reflect(Component)]
#[require(MovementVector)]
pub struct Speed(pub f32);

#[derive(Component, Reflect, Debug, Deref, DerefMut, Default)]
#[reflect(Component)]
pub struct MovementVector(pub Vec2);

fn apply_movement_vector(mover: Query<(&mut Transform, &Speed, &MovementVector)>, time: Res<Time>) {
    for (mut transform, speed, movement_vector) in mover {
        let displacement = movement_vector.0.normalize_or_zero() * speed.0 * time.delta_secs();
        transform.translation += displacement.extend(0.);
    }
}

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_movement_vector)
            .register_type::<MovementVector>()
            .register_type::<Speed>();
    }
}
