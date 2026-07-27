`bevy_falling_sand` is a falling-sand simulation engine for Bevy apps.

It provides a particle grid, chunked simulation scheduling, rendering, movement
rules, reactions, persistence, scene loading, and avian2d physics
integration.

Use the default plugin for the full engine, or disable default features and add
only the systems you need.

- [Bevy Versions](#bevy-versions)
- [Getting Started](#getting-started)
- [Particle Types](#particle-types)
- [Feature Flags](#feature-flags)
- [Common Pitfalls](#common-pitfalls)
  - [Frame pacing](#frame-pacing)
  - [Slow simulation speeds](#slow-simulation-speeds)
    - [Profile optimizations](#profile-optimizations)
    - [Complex particle types](#complex-particle-types)
    - [Undefined particle movement behavior in parallel systems](#undefined-particle-movement-behavior-in-parallel-systems)

# Bevy Versions

| `bevy_falling_sand` | `bevy` |
| ------------------- | ------ |
| 0.8.x               | 0.19.x |
| 0.7.x               | 0.18.x |

# Getting Started

```rust
use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default()
                // 64x64 particles per chunk
                .with_chunk_size(64)
                // 64x64 active chunks in the map
                .with_map_size(64),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, sand_emitter)
        .run();
}

#[derive(Resource)]
struct SandParticle(ParticleTypeId);

fn setup(mut commands: Commands) {
    let sand = ParticleType::new();
    commands.insert_resource(SandParticle(sand.id()));

    commands.spawn((
        sand,
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
            Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
        ]),
        Movement::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ]),
        Density::new(1250),
        Speed::new(5, 10),
    ));
}

fn sand_emitter(mut writer: MessageWriter<SpawnParticleSignal>, sand: Res<SandParticle>) {
    for x in 0..10 {
        for y in 0..10 {
            writer.write(SpawnParticleSignal::new(sand.0, IVec2::new(x, y)));
        }
    }
}
```

See [docs.rs/bevy_falling_sand](https://docs.rs/bevy_falling_sand) for the full
module guide and API documentation. The repository also includes runnable
examples for movement, reactions, mutation, noise, rigid bodies, and basic setup.

# Particle Types

Entities with `ParticleType` are templates for spawned `Particle` entities.
Each `ParticleType` owns a stable `ParticleTypeId`; keep that ID in your own
resources or components when gameplay code needs to spawn, mutate, despawn, or
otherwise refer to that type.

Spawn particles by sending `SpawnParticleSignal` with Bevy's `MessageWriter`.
Despawn them with `DespawnParticleSignal`. Directly spawning `Particle` entities
is not supported; the plugin needs to keep the particle map, type registry, and
child entities synchronized.

Common components for `ParticleType` entities:

| Component                 | Description                                                | Feature     |
| ------------------------- | ---------------------------------------------------------- | ----------- |
| `ColorProfile`            | Color profile from a palette, gradient, or texture         | `render`    |
| `ForceColor`              | Overrides assigned particle color                          | `render`    |
| `Movement`                | Ordered movement candidate groups                          | `movement`  |
| `Density`                 | Displacement comparison value                              | `movement`  |
| `Speed`                   | Maximum movement attempts per frame                        | `movement`  |
| `AirResistance`           | Per-tier chance to skip movement into empty space          | `movement`  |
| `ParticleResistor`        | Chance to resist displacement by another particle          | `movement`  |
| `Momentum`                | Bias toward the last successful movement direction         | `movement`  |
| `ContactReaction`         | Contact rules that consume and produce particle type IDs   | `reactions` |
| `Fire`, `Flammable`       | Fire spread and burn behavior                              | `reactions` |
| `Corrosive`, `Corrodible` | Corrosion behavior                                         | `reactions` |
| `StaticRigidBodyParticle` | Include particles in generated static collision meshes     | `physics`   |
| `TimedLifetime`           | Despawn after a duration                                   | core        |
| `ChanceLifetime`          | Chance to despawn per tick                                 | core        |
| `TimedMutation`           | Mutate into another particle type after a duration         | core        |
| `ChanceMutation`          | Chance to mutate into another particle type per tick       | core        |

# Feature Flags

All features are enabled by default.

| Feature       | Description                                      | Implies              |
| ------------- | ------------------------------------------------ | -------------------- |
| `render`      | Particle colors, chunk textures, effect layers   | —                    |
| `movement`    | Particle movement systems                        | —                    |
| `reactions`   | Contact, fire, and corrosion reactions           | `render`, `movement` |
| `physics`     | avian2d rigid body integration                   | —                    |
| `debug`       | Debug counters and gizmo overlays                | —                    |
| `persistence` | Chunk save/load and particle type serialization  | `bfs`, `bfc`         |
| `scenes`      | Layered scene assets from RON and images         | —                    |
| `bfs`         | Compact particle format without color            | —                    |
| `bfc`         | Particle format with per-particle color          | `render`             |

# Common Pitfalls

## Frame pacing

Consider adding frame pacing to your app with something like
[bevy_framepace](https://github.com/aevyrie/bevy_framepace). Particles are evaluated
_per frame_, so a simulation at 60 Hz will look very different than one at 144 Hz.

60 fps is a reasonable starting point for your simulation.

## Slow simulation speeds

`bfs` is well optimized, but some setups can still run slowly.

### Profile optimizations

Optimized debug and release profiles make a noticeable difference.
Building your project with [bevy_cli](https://github.com/TheBevyFlock/bevy_cli) is recommended,
as it handles most of these cases for you.

To squeeze out every last bit of performance at the expense of long compile times, set `lto = "true"`
in your release profile (which `bevy_cli` does not do as of
[v0.6.0](https://github.com/TheBevyFlock/bevy_cli/releases/tag/lint-v0.6.0)).

### Complex particle types

`ParticleType` creation is intentionally flexible, which also makes it easy to build particle
behaviors that take a long time to process in simulation hot paths.

The `Movement` component is a common offender of this. Depending on your hardware, it is
usually a good idea to keep movement candidate positions for a particle below ~12 total
positions.

`Speed` is another component that should be carefully configured. A particle with a max speed
of 10 may try to move 10 times in a single frame. A particle with 3 movement candidates and a
speed of 10 can be evaluated as many as 30 times per frame.

These examples typically run fine even with many moving particles, but they are near the upper
limit for modern hardware. Finding a balance for these components is key in creating a cool-looking
and fast-performing simulation.

### Undefined particle movement behavior in parallel systems

To make movement parallel, particles are subdivided into chunks and iterated in a checkerboard
pattern. This is only well-defined when a particle's `Movement` behavior doesn't let it move
farther than `chunk_length / 2` in a single frame.

For example, a world with a chunk size of 64 must not have particles with `Movement` and
`Speed` components that would move them more than 32 positions in a single frame via the movement
systems.

If this happens, the offending particle may attempt to mutably access positions in the particle map
that other threads are accessing at the same time, leading to undefined behavior.

This safety consideration is exclusive to movement systems. Manually moving a `Particle`
entity's `GridPosition` is safe, as long as the user keeps the `GridPosition` in sync with
the `ParticleMap` resource.
