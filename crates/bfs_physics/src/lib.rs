use avian2d::prelude::*;
use bevy::prelude::*;
use bfs_movement::Wall;

pub struct FallingSandPhysicsPlugin;

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, accelerate_bodies);
        app.add_systems(Update, print_wall_particles);
    }
}

#[derive(Component, Debug)]
struct MyTestRigidBody;

fn setup(mut commands: Commands) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Sprite {
            color: Color::srgba(1., 0., 0., 1.),
            ..default()
        },
        MyTestRigidBody,
    ));
    commands.spawn((RigidBody::Static, Collider::rectangle(5.0, 0.5)));
}

fn accelerate_bodies(
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<MyTestRigidBody>>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();
    for (mut linear_velocity, mut angular_velocity) in &mut query {
        linear_velocity.x += 2.0 * delta_secs;
        angular_velocity.0 += 0.5 * delta_secs;
    }
}

fn print_wall_particles(query: Query<&Wall>) {
    query.iter().for_each(|w| println!("{:?}", w));
}
