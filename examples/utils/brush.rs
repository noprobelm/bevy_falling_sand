use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_falling_sand::prelude::{
    DespawnParticleSignal, Particle, ParticleMap, ParticleSystems, SpawnParticleSignal,
};

use super::{
    cursor::{Cursor, update_cursor_position},
    states::AppState,
};

#[derive(Clone, Debug, Resource)]
pub struct ParticleSpawnList {
    particles: Vec<Particle>,
    index: usize,
}

impl ParticleSpawnList {
    pub fn new(particles: Vec<Particle>) -> Self {
        Self {
            particles,
            index: 0,
        }
    }

    pub fn current(&self) -> Option<&Particle> {
        self.particles.get(self.index)
    }

    pub fn cycle_next(&mut self) -> Option<&Particle> {
        if self.particles.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.particles.len();
        self.current()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BrushInput {
    Key(KeyCode),
    Mouse(MouseButton),
}

fn is_brush_input_pressed(
    input: BrushInput,
) -> impl Fn(Res<ButtonInput<KeyCode>>, Res<ButtonInput<MouseButton>>) -> bool + Clone {
    move |keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>| match input {
        BrushInput::Key(key) => keys.pressed(key),
        BrushInput::Mouse(button) => mouse.pressed(button),
    }
}

fn is_brush_input_just_pressed(
    input: BrushInput,
) -> impl Fn(Res<ButtonInput<KeyCode>>, Res<ButtonInput<MouseButton>>) -> bool + Clone {
    move |keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>| match input {
        BrushInput::Key(key) => keys.just_pressed(key),
        BrushInput::Mouse(button) => mouse.just_pressed(button),
    }
}

#[derive(Clone, Resource)]
pub struct BrushKeybindings {
    pub spawn_despawn_button: BrushInput,
    pub sample_button: BrushInput,
    pub toggle_brush_state_button: BrushInput,
    pub resize_modifier_key: KeyCode,
    pub cycle_particle_button: BrushInput,
    pub cycle_brush_type_button: BrushInput,
}

impl Default for BrushKeybindings {
    fn default() -> Self {
        Self {
            spawn_despawn_button: BrushInput::Mouse(MouseButton::Left),
            sample_button: BrushInput::Key(KeyCode::Space),
            toggle_brush_state_button: BrushInput::Key(KeyCode::Tab),
            resize_modifier_key: KeyCode::AltLeft,
            cycle_particle_button: BrushInput::Mouse(MouseButton::Right),
            cycle_brush_type_button: BrushInput::Mouse(MouseButton::Middle),
        }
    }
}

#[derive(Default)]
pub struct BrushPlugin {
    pub keybindings: BrushKeybindings,
}

impl BrushPlugin {
    pub fn with_keybindings(mut self, keybindings: BrushKeybindings) -> Self {
        self.keybindings = keybindings;
        self
    }
}

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        let keybindings = self.keybindings.clone();

        app.init_state::<BrushState>()
            .init_state::<BrushType>()
            .init_gizmo_group::<BrushGizmos>()
            .insert_resource(keybindings.clone())
            .add_systems(Startup, setup_brush)
            .add_systems(
                Update,
                (
                    update_brush_gizmos,
                    sample_hovered.run_if(is_brush_input_just_pressed(keybindings.sample_button)),
                    toggle_brush_state.run_if(is_brush_input_just_pressed(
                        keybindings.toggle_brush_state_button,
                    )),
                    cycle_selected_particle.run_if(is_brush_input_just_pressed(
                        keybindings.cycle_particle_button,
                    )),
                    cycle_brush_type.run_if(is_brush_input_just_pressed(
                        keybindings.cycle_brush_type_button,
                    )),
                    resize_brush,
                    handle_alt_app_state_transition,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_particles
                        .run_if(is_brush_input_pressed(keybindings.spawn_despawn_button))
                        .run_if(in_state(BrushState::Spawn))
                        .run_if(in_state(AppState::Canvas))
                        .before(ParticleSystems::Simulation)
                        .after(update_cursor_position),
                    despawn_particles
                        .run_if(is_brush_input_pressed(keybindings.spawn_despawn_button))
                        .run_if(in_state(BrushState::Despawn))
                        .run_if(in_state(AppState::Canvas))
                        .before(ParticleSystems::Simulation)
                        .after(update_cursor_position),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component, Copy, Clone, Default, Debug)]
pub struct Brush;

#[derive(Component, Copy, Clone, Default, Debug)]
pub struct BrushSize(pub usize);

#[derive(Component, Clone, Default, Debug)]
pub struct BrushColor(pub Color);

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrushState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(States, Default, Clone, Hash, Eq, PartialEq, Debug)]
pub enum BrushType {
    Line,
    #[default]
    Circle,
    Cursor,
}

impl BrushType {
    pub fn cycle_next(&self) -> Self {
        match self {
            BrushType::Line => BrushType::Circle,
            BrushType::Circle => BrushType::Cursor,
            BrushType::Cursor => BrushType::Line,
        }
    }
}

#[derive(Resource)]
pub struct SelectedBrushParticle(pub Particle);

fn setup_brush(mut commands: Commands) {
    commands.spawn((
        Brush,
        BrushSize(2),
        BrushColor(Color::Srgba(Srgba::new(1., 1., 1., 0.3))),
    ));
}

fn update_brush_gizmos(
    cursor: Res<Cursor>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<(&BrushSize, &BrushColor), With<Brush>>,
) -> Result {
    let (size, color) = brush_query.single()?;

    match brush_type.get() {
        BrushType::Line => brush_gizmos.line_2d(
            Vec2::new(cursor.current.x - size.0 as f32 * 3. / 2., cursor.current.y),
            Vec2::new(cursor.current.x + size.0 as f32 * 3. / 2., cursor.current.y),
            color.0,
        ),
        BrushType::Circle => {
            brush_gizmos.circle_2d(cursor.current, size.0 as f32, color.0);
        }
        BrushType::Cursor => brush_gizmos.cross_2d(cursor.current, 1., color.0),
    }
    Ok(())
}

fn spawn_particles(
    mut spawn_writer: MessageWriter<SpawnParticleSignal>,
    cursor: Res<Cursor>,
    selected: Res<SelectedBrushParticle>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&BrushSize, With<Brush>>,
) -> Result {
    let size = brush_query.single()?;
    for pos in alg::get_positions(
        cursor.current,
        cursor.previous,
        cursor.previous_previous,
        size.0 as f32,
        brush_type.get(),
    ) {
        spawn_writer.write(SpawnParticleSignal::new(selected.0.clone(), pos));
    }
    Ok(())
}

fn despawn_particles(
    mut despawn_writer: MessageWriter<DespawnParticleSignal>,
    cursor: Res<Cursor>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&BrushSize, With<Brush>>,
) -> Result {
    let size = brush_query.single()?;
    for pos in alg::get_positions(
        cursor.current,
        cursor.previous,
        cursor.previous_previous,
        size.0 as f32,
        brush_type.get(),
    ) {
        despawn_writer.write(DespawnParticleSignal::from_position(pos));
    }
    Ok(())
}

fn toggle_brush_state(
    mut brush_state: ResMut<NextState<BrushState>>,
    current_state: Res<State<BrushState>>,
) {
    match current_state.get() {
        BrushState::Spawn => brush_state.set(BrushState::Despawn),
        BrushState::Despawn => brush_state.set(BrushState::Spawn),
    }
}

fn resize_brush(
    mut scroll_events: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    keybindings: Res<BrushKeybindings>,
    mut brush_query: Query<&mut BrushSize, With<Brush>>,
) -> Result {
    if keys.pressed(keybindings.resize_modifier_key) {
        let mut size = brush_query.single_mut()?;
        for event in scroll_events.read() {
            if event.y > 0.0 {
                size.0 = size.0.saturating_add(1).min(50);
            } else if event.y < 0.0 {
                size.0 = size.0.saturating_sub(1).max(1);
            }
        }
    }
    Ok(())
}

pub fn handle_alt_app_state_transition(
    keys: Res<ButtonInput<KeyCode>>,
    keybindings: Res<BrushKeybindings>,
    mut app_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    if keys.pressed(keybindings.resize_modifier_key) && current_state.get() == &AppState::Canvas {
        app_state.set(AppState::Ui);
    }
}

pub fn handle_alt_release_without_egui(
    keys: Res<ButtonInput<KeyCode>>,
    keybindings: Res<BrushKeybindings>,
    mut app_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    if !keys.pressed(keybindings.resize_modifier_key) && current_state.get() == &AppState::Ui {
        app_state.set(AppState::Canvas);
    }
}

fn cycle_selected_particle(
    mut particle_spawn_list: ResMut<ParticleSpawnList>,
    mut selected_particle: ResMut<SelectedBrushParticle>,
) {
    if let Some(next_particle) = particle_spawn_list.cycle_next() {
        selected_particle.0 = next_particle.clone();
    }
}

fn cycle_brush_type(
    mut brush_type: ResMut<NextState<BrushType>>,
    current_type: Res<State<BrushType>>,
) {
    brush_type.set(current_type.get().cycle_next());
}

fn sample_hovered(
    cursor: Res<Cursor>,
    chunk_map: Res<ParticleMap>,
    particle_query: Query<&Particle>,
    mut selected_brush_particle: ResMut<SelectedBrushParticle>,
    mut brush_state: ResMut<NextState<BrushState>>,
) {
    if let Ok(Some(entity)) = chunk_map.get_copied(cursor.current.as_ivec2()) {
        let particle = particle_query.get(entity).unwrap();
        selected_brush_particle.0 = particle.clone();
        brush_state.set(BrushState::Spawn);
    }
}

pub mod alg {
    use bevy::prelude::*;

    use super::BrushType;

    pub fn get_positions(
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        brush_size: f32,
        brush_type: &BrushType,
    ) -> Vec<IVec2> {
        let cursor_pairs = [(p1, p2), (p2, p3)];

        cursor_pairs
            .iter()
            .flat_map(|(start, end)| match brush_type {
                BrushType::Circle => get_interpolated_circle_points(*start, *end, brush_size),
                BrushType::Line => get_interpolated_line_points(*start, *end, brush_size),
                BrushType::Cursor => get_interpolated_cursor_points(*start, *end),
            })
            .collect()
    }

    fn get_interpolated_line_points(start: Vec2, end: Vec2, line_length: f32) -> Vec<IVec2> {
        let mut positions = vec![];

        let min_x = -((line_length as i32) / 2) * 3;
        let max_x = (line_length as i32 / 2) * 3;

        let direction = (end - start).normalize();
        let length = (end - start).length();
        let num_samples = (length.ceil() as usize).max(1);

        for i in 0..=num_samples {
            let t = i as f32 / num_samples as f32;
            let sample_point = start + direction * length * t;

            for x_offset in min_x..=max_x {
                let position = IVec2::new(
                    (sample_point.x + x_offset as f32).floor() as i32,
                    sample_point.y.floor() as i32,
                );
                positions.push(position);
            }
        }

        positions
    }

    fn get_interpolated_cursor_points(start: Vec2, end: Vec2) -> Vec<IVec2> {
        if start == end {
            return vec![start.floor().as_ivec2()];
        }

        let mut positions = vec![];
        let direction = (end - start).normalize();
        let length = (end - start).length();
        let num_samples = (length.ceil() as usize).max(1);

        for i in 0..=num_samples {
            let t = i as f32 / num_samples as f32;
            positions.push((start + direction * length * t).floor().as_ivec2());
        }
        positions
    }

    fn get_interpolated_circle_points(start: Vec2, end: Vec2, radius: f32) -> Vec<IVec2> {
        let mut positions = vec![];
        if start == end {
            let min_x = (start.x - radius).floor() as i32;
            let max_x = (start.x + radius).ceil() as i32;
            let min_y = (start.y - radius).floor() as i32;
            let max_y = (start.y + radius).ceil() as i32;
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let pos = Vec2::new(x as f32, y as f32);
                    if (pos - start).length() <= radius {
                        positions.push(pos.as_ivec2());
                    }
                }
            }
            return positions;
        }

        let length = (end - start).length();
        let direction = (end - start).normalize();

        let min_x = (start.x.min(end.x) - radius).floor() as i32;
        let max_x = (start.x.max(end.x) + radius).ceil() as i32;
        let min_y = (start.y.min(end.y) - radius).floor() as i32;
        let max_y = (start.y.max(end.y) + radius).ceil() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let point = Vec2::new(x as f32, y as f32);

                let to_point = point - start;
                let projected_length = to_point.dot(direction);
                let clamped_length = projected_length.clamp(0.0, length);

                let closest_point = start + direction * clamped_length;
                let distance_to_line = (point - closest_point).length();

                if distance_to_line <= radius {
                    positions.push(IVec2::new(x, y));
                }
            }
        }

        positions
    }
}
