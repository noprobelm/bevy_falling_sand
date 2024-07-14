`bevy_falling_sand` is a generic plugin for adding falling sand physics to your Bevy project.

## Bevy versions

| `bevy_falling_sand`   | `bevy`    |
|-----------------------|-----------|
| 0.2.x                 | 0.14.x    |
| 0.1.x                 | 0.13.x    |

## Example
If you clone this repository, there is an example available that provides a full GUI interface for a "sandbox" like 
experience. Though this project is surprisingly optimized, I recommend running the example with the `--release` flag to
maximize performance:
```rust
cargo run --example sandbox --release
```

## How to use

Spawning a particle is easy, just insert a `ParticleType` component variant to an entity with a `Transform`
component and it will be added to the simulation:
```rust
commands.spawn((
    ParticleType::Water,
    SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
    ));
```

## Current limitations
### ChunkMap size
For optimization reasons, the underlying mapping mechanism for particles utilizes a sequence of chunks, each of which
will induce a "hibernating" state on itself and the particles it contains if no movement is detected in a given frame.

For this reason, the total map size is limited (the default is 1024x1024 spanning from transform `(-512, 512)` 
through `(512,  -512)`). Any particle processed outside of this region will cause a panic. This will be resolved
in a future release, which will modify the ChunkMap to "follow" and entity with an arbitrary specified component
(for example, a main camera), loading and unloading chunks as it moves.

### Single-threaded simulation
I've found the simulation runs surprisingly well despite the lack of multi-threading capabiltiy (excluding what `bevy` 
inherently provides with its scheduling mechanisms). 

An optional multi-threaded simulation is planned for a future release, but if you want to tweak with CPU thread 
allocation in the meantime to experiment with performance, you might try tweaking the default task pool thread
assignment policy that `bevy` provides. I've found differing results in performance based on CPU 
manufacturer/architecture (sorry, no benchmarks available yet)

```rust
use bevy::{
    core::TaskPoolThreadAssignmentPolicy, prelude::*, tasks::available_parallelism,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
        .set(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions {
                compute: TaskPoolThreadAssignmentPolicy {
                    min_threads: available_parallelism(),
                    max_threads: std::usize::MAX,
                    percent: 1.0,
                },
                ..default()
            },
        }));
}
```

## Visualizing chunk behavior

If you want to visualize how chunks behave, insert the `DebugParticles` resource:
```rust
app.init_resource::<DebugParticles>()
```

![Debug Mode Demo](https://github.com/noprobelm/bevy_falling_sand/blob/media/debug.gif)

## Demos
### Steam
![Steam Demo](https://github.com/noprobelm/bevy_falling_sand/blob/media/steam.gif)
### Water
![Water Demo](https://github.com/noprobelm/bevy_falling_sand/blob/media/water.gif)
