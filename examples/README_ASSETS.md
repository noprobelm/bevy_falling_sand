# BFS Assets - Particle Definition Loading

The `bfs_assets` crate provides functionality to load particle definitions from RON (Rusty Object Notation) files. This allows you to define all your particle types in external files and load them at runtime.

## Features

- **Serializable Particle Data**: All particle components can be defined in RON files
- **Bevy Custom Asset**: Integrates with Bevy's asset system for hot-reloading support
- **Complete Component Support**: Supports all BFS particle components:
  - Core: `ParticleTypeId`
  - Movement: `Density`, `Velocity`, `Momentum`, material types (`Liquid`, `Gas`, `MovableSolid`, `Solid`, `Wall`)
  - Color: `ColorProfile`, `ChangesColor`
  - Reactions: `Fire`, `Burns`, `Burning`

## Usage

### 1. Add the Plugin

```rust
use bfs_assets::FallingSandAssetsPlugin;

App::new()
    .add_plugins((
        DefaultPlugins,
        FallingSandCorePlugin,
        FallingSandColorPlugin,
        FallingSandMovementPlugin,
        FallingSandReactionsPlugin,
        FallingSandAssetsPlugin, // Add this
    ))
    .run();
```

### 2. Load Particle Definitions

```rust
use bfs_assets::{ParticleDefinitionsAsset, ParticleDefinitionsHandle};

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the particle definitions asset
    let particles_handle: Handle<ParticleDefinitionsAsset> = 
        asset_server.load("particles/my_particles.ron");
    
    // Spawn an entity to track the asset loading
    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));
}
```

### 3. Create Particle Definition Files

Create RON files in your `assets/particles/` directory. See `modern_particles.ron` for a complete example.

## RON File Format

The RON file should contain a map of particle names to `ParticleData` structures:

```ron
{
    "Water": ParticleData(
        name: "Water",
        density: Some(750),
        max_velocity: Some(3),
        momentum: Some(true),
        liquid: Some(5),
        colors: Some(["#0B80AB80"]),
    ),
    "Fire": ParticleData(
        name: "Fire",
        density: Some(450),
        max_velocity: Some(3),
        gas: Some(1),
        colors: Some(["#FF5900FF", "#FF9100FF", "#FFCF00FF"]),
        fire: Some(FireData(
            burn_radius: 1.5,
            chance_to_spread: 0.01,
        )),
    ),
    // ... more particles
}
```

## ParticleData Fields

### Movement Properties
- `density: Option<u32>` - Particle density for movement priority
- `max_velocity: Option<u8>` - Maximum movement speed
- `momentum: Option<bool>` - Whether particle maintains momentum

### Material Types (mutually exclusive)
- `liquid: Option<u8>` - Liquid viscosity (higher = more viscous)
- `gas: Option<u8>` - Gas buoyancy
- `movable_solid: Option<bool>` - Falls like sand
- `solid: Option<bool>` - Static but can be moved by other particles
- `wall: Option<bool>` - Completely immovable

### Visual Properties
- `colors: Option<Vec<String>>` - Hex color strings (e.g., "#FF0000", "#FF0000FF")
- `changes_colors: Option<f64>` - Chance to randomly change color per frame

### Reaction Properties
- `fire: Option<FireData>` - Fire emission properties
- `burning: Option<BurningData>` - Active burning state
- `burns: Option<BurnsData>` - Burn susceptibility and behavior

## Sub-structures

### FireData
```ron
FireData(
    burn_radius: 1.5,        // How far fire can spread
    chance_to_spread: 0.01,  // Probability per tick
)
```

### BurningData
```ron
BurningData(
    duration: 1000,    // Burn duration in milliseconds
    tick_rate: 100,    // Tick interval in milliseconds
)
```

### BurnsData
```ron
BurnsData(
    duration: 5000,
    tick_rate: 100,
    chance_destroy_per_tick: Some(0.1),
    reaction: Some(ReactionData(
        produces: "Smoke",
        chance_to_produce: 0.035,
    )),
    colors: Some(["#FF0000", "#FF5900"]), // Colors while burning
    spreads: Some(FireData(               // Fire spread while burning
        burn_radius: 2.0,
        chance_to_spread: 0.2,
    )),
)
```

### ReactionData
```ron
ReactionData(
    produces: "Smoke",           // Name of particle to produce
    chance_to_produce: 0.035,    // Probability per tick
)
```

## Color Format

Colors are specified as hex strings:
- `"#RGB"` - Short format (3 digits)
- `"#RRGGBB"` - Standard format (6 digits)
- `"#RRGGBBAA"` - With alpha channel (8 digits)

Examples:
- `"#FF0000"` - Pure red
- `"#0B80AB80"` - Blue-green with 50% transparency
- `"#FFFFFF"` - White

## Automatic Loading

The `FallingSandAssetsPlugin` automatically:
1. Registers the `ParticleDefinitionsAsset` type
2. Registers the asset loader for `.ron` files
3. Runs a system that spawns particle types when assets are loaded
4. Handles all component setup based on the RON data

## Hot Reloading

Thanks to Bevy's asset system, particle definition files support hot reloading in development builds. Modify your RON files and the changes will be automatically applied.

## Migration from Old Format

The old `particles.ron` format used a simpler structure. The new format is more explicit and supports all current BFS features. Use the provided converter or manually update your files to the new `ParticleData` structure.

## Example

See `examples/particle_assets.rs` for a complete working example of how to set up and use the asset loading system.