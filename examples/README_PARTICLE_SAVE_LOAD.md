# Particle Save/Load Example

This example demonstrates the complete workflow for saving and loading particle definitions using the `bfs_assets` system. It shows how to:

1. **Load particle definitions from RON files** using Bevy's asset system
2. **Dynamically cycle through loaded particle types** during runtime
3. **Save particle definitions back to RON format** (printed to console)
4. **Spawn particles continuously** from loaded definitions

## How to Run

```bash
cargo run --example particle_save_load
```

## Controls

- **TAB**: Cycle through available particle types (loaded from asset)
- **F1**: Toggle continuous particle spawning on/off
- **F2**: Toggle debug particle chunk map visualization
- **F3**: Save current particle definitions to RON format (printed to console)
- **WASD**: Pan the camera around the simulation
- **Mouse Wheel**: Zoom in/out
- **R**: Reset simulation (clear all dynamic particles)

## Features Demonstrated

### 1. Asset Loading
- Loads `assets/particles/demo_particles.ron` on startup
- Automatically spawns particle type entities when asset loads
- Updates UI to show available particle types

### 2. Dynamic Particle Type Selection
- Lists all particle types loaded from the asset
- Allows cycling through types with TAB key
- Shows current selection in UI with count (e.g., "Water (1/6)")

### 3. Continuous Particle Spawning
- Spawns particles of the selected type continuously when enabled
- Creates a circular spray pattern above the simulation area
- Respects particle type availability (only spawns if loaded)

### 4. Particle Definition Saving
- Demonstrates how to serialize particle data back to RON
- Saves a subset of interesting particles (Water, Sand, Oil, FIRE, Wall)
- Prints the RON output to console (in real apps, would save to file)

### 5. Visual Simulation
- Shows different particle behaviors from loaded definitions:
  - **Water**: Liquid behavior with transparency
  - **Sand**: Movable solid particles
  - **Oil**: Flammable liquid that can burn and spread fire
  - **FIRE**: Gas particles that spread and burn other materials
  - **Smoke**: Light gas particles produced by reactions
  - **Wall**: Static boundary particles

## Asset File Structure

The example loads from `demo_particles.ron`, which contains:

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
    // ... more particles
}
```

## Code Structure

### Key Components

1. **Asset Loading Setup**
   ```rust
   let particles_handle: Handle<ParticleDefinitionsAsset> = 
       asset_server.load("particles/demo_particles.ron");
   commands.spawn(ParticleDefinitionsHandle::new(particles_handle));
   ```

2. **Asset Loading Check**
   ```rust
   fn check_asset_loading(
       handles: Query<&ParticleDefinitionsHandle>,
       assets: Res<Assets<ParticleDefinitionsAsset>>,
       // ... updates UI and particle type list when loaded
   )
   ```

3. **Dynamic Particle Spawning**
   ```rust
   fn stream_particles(
       mut commands: Commands, 
       current_particle_type: Res<CurrentParticleType>,
       particle_type_map: Res<ParticleTypeMap>,
   ) {
       // Spawns particles of the currently selected type
   }
   ```

4. **Particle Definition Saving**
   ```rust
   fn save_particles(
       handles: Query<&ParticleDefinitionsHandle>,
       assets: Res<Assets<ParticleDefinitionsAsset>>,
   ) {
       // Serializes particle data back to RON format
   }
   ```

### Resource Management

- **`CurrentParticleType`**: Tracks which particle type is selected for spawning
- **`SpawnParticles`**: Controls whether continuous spawning is active
- **`BoundaryReady`**: Ensures boundary walls are set up before spawning

## Real-World Usage

This example demonstrates patterns useful for:

- **Level Editors**: Loading/saving particle configurations
- **Mod Support**: Allowing users to define custom particles
- **Game Configuration**: Runtime particle behavior customization
- **Content Pipelines**: Converting between internal and external formats

## Hot Reloading

Thanks to Bevy's asset system, you can modify `demo_particles.ron` while the example is running to see changes applied automatically (in debug builds).

## Extending the Example

To add more functionality:

1. **File I/O**: Replace console printing with actual file saving
2. **UI Improvements**: Add particle type thumbnails or property displays
3. **Interactive Editing**: Allow runtime modification of particle properties
4. **Multiple Assets**: Load particle definitions from multiple files
5. **Particle Templates**: Create base templates for common particle types

## Related Files

- `assets/particles/demo_particles.ron` - Example particle definitions
- `assets/particles/modern_particles.ron` - Full particle set from conversion
- `examples/README_ASSETS.md` - Detailed documentation of the asset system