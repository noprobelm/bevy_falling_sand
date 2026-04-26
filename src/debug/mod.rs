//! Provides debug and statistics constructs
pub mod schedule;

use bevy::prelude::*;

pub use crate::debug::resources::*;
pub use crate::debug::schedule::*;
use crate::debug::systems::SystemsPlugin;

/// Adds the constructs and systems necessary for debugging the Falling Sand simulation.
pub struct FallingSandDebugPlugin;

impl Plugin for FallingSandDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((schedule::SchedulePlugin, ResourcesPlugin, SystemsPlugin));
    }
}

pub(super) mod resources {
    use bevy::prelude::*;

    pub(super) struct ResourcesPlugin;

    impl Plugin for ResourcesPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<DynamicParticleCount>()
                .init_resource::<StaticParticleCount>()
                .init_resource::<TotalParticleCount>()
                .init_resource::<ActiveParticleCount>()
                .init_resource::<RigidBodyCount>()
                .init_resource::<ChunkColor>()
                .init_resource::<ActiveChunkColor>()
                .init_resource::<DirtyRectColor>()
                .init_resource::<DebugParticleCount>()
                .init_resource::<DebugParticleMap>()
                .init_resource::<DebugDirtyRects>()
                .init_gizmo_group::<DebugGizmos>();
        }
    }

    /// Resource tracking the number of dynamic (moveable) particles in the simulation.
    ///
    /// Updated each frame when [`DebugParticleCount`] is present.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::DynamicParticleCount;
    ///
    /// fn show_count(count: Res<DynamicParticleCount>) {
    ///     println!("Dynamic particles: {}", count.0);
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct DynamicParticleCount(pub u64);

    /// Resource tracking the number of static (wall) particles in the simulation.
    ///
    /// Updated each frame when [`DebugParticleCount`] is present.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::StaticParticleCount;
    ///
    /// fn show_count(count: Res<StaticParticleCount>) {
    ///     println!("Static particles: {}", count.0);
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct StaticParticleCount(pub u64);

    /// Resource tracking the total number of particles in the simulation.
    ///
    /// Updated each frame when [`DebugParticleCount`] is present.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::TotalParticleCount;
    ///
    /// fn show_count(count: Res<TotalParticleCount>) {
    ///     println!("Total particles: {}", count.0);
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct TotalParticleCount(pub u64);

    /// Resource tracking the number of active (recently moved) particles in the simulation.
    ///
    /// Updated each frame when [`DebugParticleCount`] is present.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::ActiveParticleCount;
    ///
    /// fn show_count(count: Res<ActiveParticleCount>) {
    ///     println!("Active particles: {}", count.0);
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct ActiveParticleCount(pub u64);

    /// Resource tracking the number of rigid bodies in the simulation.
    ///
    /// Updated each frame when [`DebugParticleCount`] is present.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::RigidBodyCount;
    ///
    /// fn show_count(count: Res<RigidBodyCount>) {
    ///     println!("Rigid bodies: {}", count.0);
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct RigidBodyCount(pub u64);

    /// Marker resource that enables particle counting systems.
    ///
    /// Insert this resource to activate the systems that update
    /// [`DynamicParticleCount`], [`StaticParticleCount`], [`TotalParticleCount`],
    /// [`ActiveParticleCount`], and [`RigidBodyCount`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::DebugParticleCount;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.init_resource::<DebugParticleCount>();
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct DebugParticleCount;

    /// Marker resource that enables chunk boundary visualization via gizmos.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::DebugParticleMap;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.init_resource::<DebugParticleMap>();
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct DebugParticleMap;

    /// Marker resource that enables dirty rectangle visualization via gizmos.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::DebugDirtyRects;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.init_resource::<DebugDirtyRects>();
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, Default, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct DebugDirtyRects;

    /// Resource controlling the color used to render chunk boundaries.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::ChunkColor;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.insert_resource(ChunkColor(Color::srgba(1.0, 0.0, 0.0, 1.0)));
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, PartialEq, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct ChunkColor(pub Color);

    impl Default for ChunkColor {
        fn default() -> Self {
            Self(Color::srgba(0.67, 0.21, 0.24, 1.0))
        }
    }

    /// Resource controlling the color used to render active chunk boundaries.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::ActiveChunkColor;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.insert_resource(ActiveChunkColor(Color::srgba(0.0, 1.0, 0.0, 1.0)));
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, PartialEq, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct ActiveChunkColor(pub Color);

    impl Default for ActiveChunkColor {
        fn default() -> Self {
            Self(Color::srgba(0.52, 0.80, 0.51, 1.0))
        }
    }

    /// Resource controlling the color used to render dirty rectangles.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::debug::DirtyRectColor;
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.insert_resource(DirtyRectColor(Color::WHITE));
    /// }
    /// ```
    #[derive(Resource, Copy, Clone, PartialEq, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct DirtyRectColor(pub Color);

    impl Default for DirtyRectColor {
        fn default() -> Self {
            Self(Color::srgba(1., 1., 1., 1.))
        }
    }

    #[derive(GizmoConfigGroup, Copy, Clone, Default, Debug, Reflect)]
    pub(super) struct DebugGizmos;
}

pub(super) mod systems {
    #[cfg(feature = "physics")]
    use avian2d::prelude::RigidBody;
    use bevy::prelude::*;

    #[cfg(feature = "movement")]
    use crate::movement::Movement;
    use crate::{
        core::{ChunkDirtyState, ChunkRegion, GridPosition, Particle},
        debug::{
            ActiveParticleCount, ChunkColor, DebugDirtyRects, DebugGizmos, DebugParticleCount,
            DebugParticleMap, DirtyRectColor, DynamicParticleCount, RigidBodyCount,
            StaticParticleCount, TotalParticleCount, schedule::ParticleDebugSet,
        },
    };

    /// Adds the constructs and systems necessary for debugging the Falling Sand simulation.
    pub(super) struct SystemsPlugin;

    impl Plugin for SystemsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                (
                    color_active_chunks
                        .in_set(ParticleDebugSet)
                        .run_if(resource_exists::<DebugParticleMap>),
                    color_dirty_rects
                        .in_set(ParticleDebugSet)
                        .after(color_active_chunks)
                        .run_if(resource_exists::<DebugDirtyRects>),
                ),
            );

            // Particle counting systems — some require optional features
            {
                let counting_condition = resource_exists::<DebugParticleCount>;

                app.add_systems(
                    Update,
                    (count_total_particles, count_active_particles)
                        .in_set(ParticleDebugSet)
                        .run_if(counting_condition),
                );

                #[cfg(feature = "movement")]
                app.add_systems(
                    Update,
                    (count_dynamic_particles, count_wall_particles)
                        .in_set(ParticleDebugSet)
                        .run_if(counting_condition),
                );

                #[cfg(feature = "physics")]
                app.add_systems(
                    Update,
                    count_rigid_bodies
                        .in_set(ParticleDebugSet)
                        .run_if(counting_condition),
                );
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn color_dirty_rects(
        chunk_query: Query<&ChunkDirtyState, With<ChunkRegion>>,
        dirty_rect_color: Res<DirtyRectColor>,
        mut chunk_gizmos: Gizmos<DebugGizmos>,
    ) {
        for dirty_state in &chunk_query {
            if let Some(dirty_rect) = dirty_state.current {
                // Dirty rect uses inclusive max, so add 1 to max for correct rectangle
                let min = dirty_rect.min.as_vec2();
                let max = (dirty_rect.max + IVec2::ONE).as_vec2();
                let rect = Rect::from_corners(min, max);
                chunk_gizmos.rect_2d(rect.center(), rect.size(), dirty_rect_color.0);
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn color_active_chunks(
        chunk_query: Query<&ChunkRegion>,
        inactive_chunk_color: Res<ChunkColor>,
        mut chunk_gizmos: Gizmos<DebugGizmos>,
    ) {
        for region in &chunk_query {
            // Region uses inclusive max, so add 1 to max for correct rectangle
            let rect_bounds = region.region();
            let min = rect_bounds.min.as_vec2();
            let max = (rect_bounds.max + IVec2::ONE).as_vec2();
            let rect = Rect::from_corners(min, max);
            chunk_gizmos.rect_2d(rect.center(), rect.size(), inactive_chunk_color.0);
        }
    }

    #[cfg(feature = "movement")]
    #[allow(clippy::needless_pass_by_value)]
    fn count_dynamic_particles(
        mut dynamic_particle_count: ResMut<DynamicParticleCount>,
        query: Query<(), (With<Particle>, With<Movement>)>,
    ) {
        dynamic_particle_count.0 = query.iter().len() as u64;
    }

    #[cfg(feature = "movement")]
    #[allow(clippy::needless_pass_by_value)]
    fn count_wall_particles(
        mut wall_particle_count: ResMut<StaticParticleCount>,
        query: Query<(), (With<Particle>, Without<Movement>)>,
    ) {
        wall_particle_count.0 = query.iter().len() as u64;
    }

    fn count_total_particles(
        mut total_particle_count: ResMut<TotalParticleCount>,
        particle_query: Query<&Particle>,
    ) {
        total_particle_count.0 = particle_query.iter().len() as u64;
    }

    fn count_active_particles(
        mut active_particle_count: ResMut<ActiveParticleCount>,
        particle_query: Query<&Particle, Changed<GridPosition>>,
    ) {
        active_particle_count.0 = particle_query.iter().count() as u64;
    }

    #[cfg(feature = "physics")]
    fn count_rigid_bodies(
        mut rigid_body_count: ResMut<RigidBodyCount>,
        rigid_body_query: Query<&RigidBody>,
    ) {
        rigid_body_count.0 = rigid_body_query.iter().len() as u64;
    }
}
