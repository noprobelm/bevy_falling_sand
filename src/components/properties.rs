use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::utils::Duration;

use crate::*;

#[derive(Component)]
pub struct Fire;

#[derive(Component, Clone)]
pub struct Burns {
    pub duration: Duration,
    pub tick_duration: Duration,
    pub color_duration: Duration,
    pub colors: ParticleColors,
    pub color_chance: f64,
    pub produces: ParticleType,
    pub chance: f64,
}

impl Burns {
    pub fn new(
        duration: Duration,
        tick_duration: Duration,
        color_duration: Duration,
        colors: ParticleColors,
        color_chance: f64,
        produces: ParticleType,
        chance: f64,
    ) -> Burns {
        Burns {
            duration,
            tick_duration,
            color_duration,
            colors,
            color_chance,
            produces,
            chance,
        }
    }
}

impl Burns {
    pub fn into_burning(&self) -> Burning {
        Burning::new(
            self.duration,
            self.tick_duration,
            self.color_duration,
            self.colors.clone(),
            self.color_chance,
            self.produces,
            self.chance,
        )
    }
}

#[derive(Component)]
pub struct Burning {
    pub timer: Timer,
    pub tick_timer: Timer,
    pub color_timer: Timer,
    pub colors: ParticleColors,
    pub color_chance: f64,
    pub produces: ParticleType,
    pub chance: f64,
}

impl Burning {
    pub fn new(
        duration: Duration,
        tick_duration: Duration,
        color_duration: Duration,
        colors: ParticleColors,
        color_chance: f64,
        produces: ParticleType,
        chance: f64,
    ) -> Burning {
        Burning {
            timer: Timer::new(duration, TimerMode::Once),
            tick_timer: Timer::new(tick_duration, TimerMode::Repeating),
            color_timer: Timer::new(color_duration, TimerMode::Repeating),
            colors,
            color_chance,
            produces,
            chance,
        }
    }
}

impl Burning {
    pub fn tick(
        &mut self,
        commands: &mut Commands,
        coordinates: &Coordinates,
        delta: Duration,
        rng: &mut PhysicsRng,
        color: &mut ParticleColor,
    ) {
        self.timer.tick(delta);
        self.tick_timer.tick(delta);
        self.color_timer.tick(delta);

        if self.color_timer.finished() && rng.chance(self.color_chance) == true {
            color.0 = rng.sample(&self.colors);
        }
        if self.tick_timer.finished() && rng.chance(self.chance) == true {
            commands.spawn((
                self.produces,
                SpatialBundle::from_transform(Transform::from_xyz(
                    coordinates.0.x as f32,
                    coordinates.0.y as f32 + 1.,
                    0.,
                )),
            ));
        }
    }

    pub fn finished(&self) -> bool {
        self.timer.finished()
    }

    pub fn produces(&self) -> ParticleType {
        self.produces
    }

    pub fn burn(&self, rng: &mut PhysicsRng) {}
}
