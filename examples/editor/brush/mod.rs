mod helpers;

use helpers::*;

use bevy::{input::common_conditions::input_pressed, prelude::*};

use crate::{cursor::CursorPosition, particles::SelectedParticle};

pub(crate) struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<BrushGizmos>()
            .init_resource::<MaxBrushSize>()
            .init_state::<BrushTypeState>()
            .init_state::<BrushSpawnState>()
            .add_event::<BrushResizeEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_brush_gizmos,
                    spawn_particles.run_if(input_pressed(MouseButton::Left)),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
#[allow(dead_code)]
struct Brush;

#[derive(Component)]
struct BrushSize(usize);

#[derive(Component)]
struct BrushColor(Color);

#[derive(Reflect, Resource)]
struct MaxBrushSize(pub usize);

impl Default for MaxBrushSize {
    fn default() -> Self {
        MaxBrushSize(50)
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum BrushSpawnState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
enum BrushTypeState {
    Line,
    #[default]
    Circle,
    Cursor,
}

#[derive(Event)]
struct BrushResizeEvent(pub usize);

fn setup(mut commands: Commands) {
    commands.spawn((
        Brush,
        BrushSize(2),
        BrushColor(Color::Srgba(Srgba::new(1., 1., 1., 0.3))),
    ));
}

fn update_brush_gizmos(
    cursor_position: Res<CursorPosition>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushTypeState>>,
    brush_query: Query<(&BrushSize, &BrushColor), With<Brush>>,
) -> Result {
    let (size, color) = brush_query.single()?;

    match brush_type.get() {
        BrushTypeState::Line => brush_gizmos.line_2d(
            Vec2::new(
                cursor_position.current.x - size.0 as f32 * 3. / 2.,
                cursor_position.current.y,
            ),
            Vec2::new(
                cursor_position.current.x + size.0 as f32 * 3. / 2.,
                cursor_position.current.y,
            ),
            color.0,
        ),
        BrushTypeState::Circle => {
            brush_gizmos.circle_2d(cursor_position.current, size.0 as f32, color.0);
        }
        _ => brush_gizmos.cross_2d(cursor_position.current, 6., color.0),
    }
    Ok(())
}

fn spawn_particles(
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    selected: Res<SelectedParticle>,
    brush_type_state: Res<State<BrushTypeState>>,
    brush_query: Query<&BrushSize>,
) -> Result {
    let brush_size = brush_query.single()?;
    let half_length = (cursor_position.current - cursor_position.previous).length() / 2.0;

    match brush_type_state.get() {
        BrushTypeState::Line => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                helpers::spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                );
            }
        }
        BrushTypeState::Circle => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                    half_length,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                    half_length,
                );
            }
        }
        BrushTypeState::Cursor => {
            let particle = selected.clone();

            // Interpolate between previous positions if movement is significant
            if (cursor_position.previous - cursor_position.previous_previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous_previous,
                    cursor_position.previous,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                );
            }
        }
    }

    Ok(())
}
