`bevy_falling_sand` provides a [Falling Sand](https://en.wikipedia.org/wiki/Falling-sand_game) engine for Bevy.

- [Bevy versions](#bevy-versions)
- [Getting Started](#getting-started)
  - [Particle behavior components](#particle-behavior-components)
- [Common pitfalls](#common-pitfalls)
  - [Slow simulation speeds](#slow-simulation-speeds)
    - [Profile optimizations](#profile-optimizations)
    - [Complex particle types](#complex-particle-types)
    - [Undefined particle movement behavior in parallel systems](#undefined-particle-movement-behavior-in-parallel-systems)
    - [Integrated GPU](#integrated-gpu)
    - [Frame pacing](#frame-pacing)

# Bevy versions

| `bevy_falling_sand`   | `bevy`    |
|-----------------------|-----------|
| 0.8.x                 | 0.19.x    |
| 0.7.x                 | 0.18.x    |

# Getting Started

In addition to this boilerplate, several examples are available to help you get started.

```rust
use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default()
                // Create a map with 64x64 chunks, each of which can hold 64x64 particles
                .with_chunk_size(64)
                .with_map_size(64),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, sand_emitter)
        .run();
}

// Spawn a simple particle type with colors and movement behavior resembling sand.
fn setup(mut commands: Commands) {
    let sand = ParticleType::new();
    commands.insert_resource(SandParticle(sand.id()));

    commands.spawn((
        sand,
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
            Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
        ]),
        // First tier: look directly below. Second tier: look diagonally down.
        Movement::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ]),
        Density(1250),
        Speed::new(5, 10),
    ));
}

// Continuously emit sand between (0, 0) and (10, 10)
#[derive(Resource)]
struct SandParticle(ParticleTypeId);

impl SandParticle {
    fn id(&self) -> ParticleTypeId {
        self.0
    }
}

fn sand_emitter(mut writer: MessageWriter<SpawnParticleSignal>, sand: Res<SandParticle>) {
    for x in 0..10 {
        for y in 0..10 {
            writer.write(SpawnParticleSignal::new(
                sand.id(),
                IVec2::new(x, y),
            ));
        }
    }
}
```

Entities with the `ParticleType` component act as templates for spawned `Particle` entities.
Each `ParticleType` owns a `ParticleTypeId`; store that ID in your own resources or components
when you need to spawn, mutate, despawn, or otherwise refer to that particle type later.
Components added to a `ParticleType` (such as `ColorProfile`, `Movement`, `Density`, and `Speed`)
define the behavior of its child particles.

To spawn individual particles at runtime, send a `SpawnParticleSignal` via Bevy's
`MessageWriter`. Despawn them with `DespawnParticleSignal`.

## Particle type identities

`ParticleTypeId` is the stable handle you should pass around in gameplay code. It is `Copy`,
serializable, and independent of the Bevy `Entity` that currently owns the corresponding
`ParticleType` template. `ParticleType` is the ECS component that marks an entity as the template
for a type of particle.

Use `ParticleType::new()` for normal runtime allocation, then copy out its ID before moving the
component into the world:

```rust
let sand = ParticleType::new();
let sand_id = sand.id();

commands.spawn((sand, Movement::default()));
commands.insert_resource(SandParticle(sand_id));
```

Use `ParticleType::id()` when you already have a `ParticleType` component and need its handle.
Use `ParticleType::from_id(id)` when spawning or restoring the template entity for an ID you
already own, such as when loading persisted type definitions or building a stable catalog.
`ParticleTypeId::new()` is available when you need to allocate an ID before constructing the
template component. `ParticleTypeId::from_raw(...)` is reserved for persisted data and externally stable
catalogs where the numeric value is part of a file format or asset contract; it also reserves that
value from future automatic allocation.

The core crate does not store names on `ParticleType`. Keep comprehensive particle identifiers in
your own components/resources and map those names to `ParticleTypeId` at the application boundary.

## Particle behavior components

Insert any of these components on a [`ParticleType`] entity and [`Particle`] entities attached to
that particle type will derive their behaviors.

| Component                 | Description                                                       | Feature     |
| ------------------------- | ----------------------------------------------------------------- | ----------- |
| `ColorProfile`            | Color profile from a predefined palette or gradient               | `render`    |
| `ForceColor`              | Overrides `ColorProfile` with another color                       | `render`    |
| `Movement`                | Movement rulesets for a particle                                  | `movement`  |
| `Density`                 | Density, used for displacement comparisons                        | `movement`  |
| `Speed`                   | How many positions a particle can move per frame                  | `movement`  |
| `AirResistance`           | Chance to skip movement to a vacant location                      | `movement`  |
| `ParticleResistor`        | How much a particle resists being displaced                       | `movement`  |
| `Momentum`                | Biases movement toward the last direction                         | `movement`  |
| `ContactReaction`         | Reaction rulesets that target and produce `ParticleTypeId` values | `reactions` |
| `Fire`                    | Makes a particle spread fire                                      | `reactions` |
| `Flammable`               | Flammability properties; burn products use `ParticleTypeId`       | `reactions` |
| `Corrosive`               | Corrosive properties for particles                                | `reactions` |
| `Corrodible`              | Marks a particle as being subject to corrosion                    | `reactions` |
| `StaticRigidBodyParticle` | Marks particles for rigid body mesh generation                    | `physics`   |
| `TimedLifetime`           | Despawns a particle after a duration                              | —           |
| `ChanceLifetime`          | Chance to despawn on a per-tick basis                             | —           |
| `ChanceMutation`          | Chance to mutate a particle into another `ParticleTypeId`         | —           |
| `TimedMutation`           | Mutates a particle into another `ParticleTypeId` after a duration | —           |

For full documentation, see [docs.rs/bevy_falling_sand](https://docs.rs/bevy_falling_sand).

# Common pitfalls

## Frame pacing

It is recommended to add frame pacing to your app using something like
[bevy_framepace](https://github.com/aevyrie/bevy_framepace). Particles are evaluated on a
_per frame_ basis, so a simulation at 60 Hz will look very different than a simulation at 144 Hz.

60 fps is a reasonable starting point for your simulation.

## Slow simulation speeds

`bfs` is well optimized, but there are several situations that could cause a simulation to
run slowly.

### Profile optimizations

It is important to optimize your debug and release profiles to maximize performance.
Building your project with [bevy_cli](https://github.com/TheBevyFlock/bevy_cli) is recommended,
as it handles most of these cases for you.

To squeeze out every last bit of performance at the expense of long compile times, set `lto = "true"`
in your release profile (which `bevy_cli` does not do as of
[v0.6.0](https://github.com/TheBevyFlock/bevy_cli/releases/tag/lint-v0.6.0)).

### Complex particle types

This crate aims to provide maximum flexibility with `ParticleType` creation, but this means
the user is essentially unbounded in their options for defining particle behaviors. If one is
not careful, it's very easy to create particles that can take a long time to process in
simulation hot paths.

The `Movement` component is a common offender of this. Depending on your hardware, it is
usually a good idea to keep movement candidate positions for a particle below ~12 total
positions.

`Speed` is another component that should be carefully configured. A particle with a max speed
of 10 may try to move 10 times in a single frame. A particle with 3 movement candidates and a
speed of 10 has the potential to be evaluated as many as 30 times per frame.

Either example (12 possible movement positions, each evaluated potentially 10 times per frame)
typically runs fine even with many moving particles, but encroaches on the upper limit for
modern hardware. Finding a balance for these components is key in creating a cool-looking and
fast-performing simulation.

### Undefined particle movement behavior in parallel systems

In order to achieve movement parallelism, particles are subdivided into chunks and iterated
upon in a checkerboard pattern. This is guaranteed to work only if the user ensures a
particle's `Movement` behavior doesn't cause it to move greater than the size of the
`chunk_length / 2` in a single frame.

For example, a world with a chunk size of 64 must not have particles with `Movement` and
`Speed` components which would cause them to move more than 32 positions in a single frame
via the movement systems.

In the event this happens, the offending particle may attempt to mutably access positions in the
particle map that other threads are accessing at the same time, leading to undefined behavior.

This safety consideration is exclusive to movement systems. Manually moving a `Particle`
entity's `GridPosition` is safe, as long as the user keeps the `GridPosition` in sync with
the `ParticleMap` resource.
