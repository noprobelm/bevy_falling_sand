use bevy::{platform::collections::HashSet, prelude::*};
use bevy_falling_sand::prelude::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Sides {
    Top,
    Right,
    Bottom,
    Left,
}

impl Sides {
    pub fn all() -> Vec<Sides> {
        vec![Sides::Top, Sides::Right, Sides::Bottom, Sides::Left]
    }
}

pub struct SetupBoundary {
    irect: IRect,
    particle_type: ParticleType,
    sides: HashSet<Sides>,
    thickness: u32,
}

impl SetupBoundary {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32, particle_type: ParticleType) -> Self {
        Self {
            irect: IRect::new(x0, y0, x1, y1),
            particle_type,
            sides: HashSet::from_iter(Sides::all()),
            thickness: 1,
        }
    }

    pub fn from_irect(irect: IRect, particle_type: ParticleType) -> Self {
        Self {
            irect,
            particle_type,
            sides: HashSet::from_iter(Sides::all()),
            thickness: 1,
        }
    }

    pub fn from_corners(p0: IVec2, p1: IVec2, particle_type: ParticleType) -> Self {
        Self {
            irect: IRect::from_corners(p0, p1),
            particle_type,
            sides: HashSet::from_iter(Sides::all()),
            thickness: 1,
        }
    }

    pub fn without_sides(self, sides: Vec<Sides>) -> SetupBoundary {
        let sides: HashSet<Sides> = HashSet::from_iter(sides);
        SetupBoundary {
            sides: self.sides.difference(&sides).cloned().collect(),
            ..self
        }
    }

    pub fn with_thickness(self, thickness: u32) -> SetupBoundary {
        SetupBoundary { thickness, ..self }
    }
}

impl Command for SetupBoundary {
    fn apply(self, world: &mut World) {
        if let Some(map) = world.get_resource::<ParticleTypeMap>() {
            if map.contains(&self.particle_type.name) {
                let thickness = self.thickness as i32;

                // Spawn horizontal sides (top and bottom)
                for x in self.irect.min.x..=self.irect.max.x {
                    if self.sides.contains(&Sides::Bottom) {
                        for layer in 0..thickness {
                            world.spawn((
                                Particle::from_string(self.particle_type.name.to_string()),
                                Transform::from_xyz(
                                    x as f32,
                                    (self.irect.min.y - layer) as f32,
                                    0.0,
                                ),
                            ));
                        }
                    }
                    if self.sides.contains(&Sides::Top) {
                        for layer in 0..thickness {
                            world.spawn((
                                Particle::from_string(self.particle_type.name.to_string()),
                                Transform::from_xyz(
                                    x as f32,
                                    (self.irect.max.y + layer) as f32,
                                    0.0,
                                ),
                            ));
                        }
                    }
                }

                // Spawn vertical sides (left and right)
                for y in self.irect.min.y..=self.irect.max.y {
                    if self.sides.contains(&Sides::Left) {
                        for layer in 0..thickness {
                            world.spawn((
                                Particle::from_string(self.particle_type.name.to_string()),
                                Transform::from_xyz(
                                    (self.irect.min.x - layer) as f32,
                                    y as f32,
                                    0.0,
                                ),
                            ));
                        }
                    }
                    if self.sides.contains(&Sides::Right) {
                        for layer in 0..thickness {
                            world.spawn((
                                Particle::from_string(self.particle_type.name.to_string()),
                                Transform::from_xyz(
                                    (self.irect.max.x + layer) as f32,
                                    y as f32,
                                    0.0,
                                ),
                            ));
                        }
                    }
                }
            } else {
                warn!(
                    "No paricle type {} found in ParticleTypeMap; will not spawn boundary",
                    self.particle_type.name
                );
            }
        }
    }
}
