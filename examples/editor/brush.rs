use crate::cursor::CursorPosition;
use bevy::prelude::*;

pub(crate) struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<BrushGizmos>()
            .init_resource::<MaxBrushSize>()
            .init_state::<BrushTypeState>()
            .init_state::<BrushSpawnState>()
            .add_systems(Startup, setup)
            .add_systems(Update, update_brush_gizmos);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
struct Brush;

#[derive(Component)]
pub struct BrushSize(pub usize);

#[derive(Component)]
struct BrushColor(pub Color);

#[derive(Reflect, Resource)]
pub struct MaxBrushSize(pub usize);

impl Default for MaxBrushSize {
    fn default() -> Self {
        MaxBrushSize(50)
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrushSpawnState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
pub enum BrushTypeState {
    Line,
    #[default]
    Circle,
    Cursor,
}

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
