`bevy_falling_sand` is a generic plugin for adding falling sand physics to your Bevy project.

## Bevy versions

| `bevy_falling_sand`   | `bevy`    |
|-----------------------|-----------|
| 0.2.x                 | 0.14.x    |
| 0.1.x                 | 0.13.x    |

## How to use

Spawning a particle is easy, just insert a [ParticleType] component variant to an entity with a [Transform]
component and it will be added to the simulation:
```rust
commands.spawn((
    ParticleType::Water,
    SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
    ));
```

## `ChunkMap`
For performance reasons, the underlying mapping mechanism for particles utilizes a sequence of _chunks_, each of which will
enter a "hibernating" state if there are no active particles within its region. As a consequence, the particle map
_is not_ unlimited in size. By default, a `ChunkMap` will track particles between a transform of `(-512, 512)` through
`(512, -512)`. Unless the bounds of the `ChunkMap` are changed, any particle processed outside of this region will
cause a panic.

In a future release, the `ChunkMap` will be capable of dynamically loading/unloading scenes according
to arbitrary any arbitrary entity's transform.

## Visualizing chunk behavior

If you want to visualize how chunks behave, insert the [DebugParticles] resource:
```rust
app.init_resource::<DebugParticles>()
```
