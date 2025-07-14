use bevy::{input::mouse::MouseWheel, platform::collections::HashSet, prelude::*};
use bfs_core::{Particle, ParticleMap, ParticleSimulationSet, RemoveParticleEvent};

use super::{
    cursor::{update_cursor_position, CursorCoords},
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

impl bevy::prelude::Plugin for BrushPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let keybindings = self.keybindings.clone();

        app.init_state::<BrushState>()
            .init_state::<BrushType>()
            .init_gizmo_group::<BrushGizmos>()
            .add_event::<BrushResizeEvent>()
            .insert_resource(keybindings.clone())
            .add_systems(Startup, setup_brush)
            .add_systems(
                Update,
                (
                    update_brush,
                    ev_resize_brush,
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
                    resize_brush_with_scroll,
                    handle_alt_app_state_transition,
                ),
            );
        app.add_systems(
            Update,
            (
                spawn_particles
                    .run_if(is_brush_input_pressed(keybindings.spawn_despawn_button))
                    .run_if(in_state(BrushState::Spawn))
                    .run_if(in_state(AppState::Canvas))
                    .after(update_cursor_position)
                    .before(ParticleSimulationSet),
                despawn_particles
                    .run_if(is_brush_input_pressed(keybindings.spawn_despawn_button))
                    .run_if(in_state(BrushState::Despawn))
                    .run_if(in_state(AppState::Canvas))
                    .before(ParticleSimulationSet)
                    .after(update_cursor_position)
                    .before(ParticleSimulationSet),
            ),
        );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

/// Unique identifer for a particle type. No two particle types with the same name can exist.
#[derive(Copy, Clone, PartialEq, Debug, Component)]
pub struct Brush {
    pub size: usize,
    pub color: Color,
}

impl Brush {
    pub fn new(size: usize, color: Color) -> Self {
        Brush { size, color }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Brush {
            size: 2,
            color: Color::WHITE,
        }
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrushState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(Event)]
pub struct BrushResizeEvent(pub usize);

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
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

impl BrushType {
    pub fn update_brush(
        &self,
        coords: Vec2,
        brush_size: f32,
        brush_gizmos: &mut Gizmos<BrushGizmos>,
    ) {
        match self {
            BrushType::Line => brush_gizmos.line_2d(
                Vec2::new(coords.x - brush_size * 3. / 2., coords.y),
                Vec2::new(coords.x + brush_size * 3. / 2., coords.y),
                Color::Srgba(Srgba::new(1., 1., 1., 0.3)),
            ),
            BrushType::Circle => {
                brush_gizmos.circle_2d(
                    coords,
                    brush_size,
                    Color::Srgba(Srgba::new(1., 1., 1., 0.3)),
                );
            }
            _ => brush_gizmos.cross_2d(coords, 6., Color::Srgba(Srgba::new(1., 1., 1., 0.3))),
        }
    }

    pub fn spawn_particles(
        &self,
        commands: &mut Commands,
        coords: Res<CursorCoords>,
        brush_size: f32,
        selected_brush_particle: Particle,
    ) {
        let coords = coords.clone();
        let radius = brush_size;
        let half_length = (coords.current - coords.previous).length() / 2.0;

        match self {
            BrushType::Line => {
                let particle = selected_brush_particle.clone();

                if (coords.previous - coords.previous_previous).length() < 1.0 {
                    spawn_line(commands, particle.clone(), coords.previous, brush_size);
                } else {
                    spawn_line_interpolated(
                        commands,
                        particle.clone(),
                        coords.previous,
                        coords.previous_previous,
                        brush_size,
                    );
                }

                if (coords.current - coords.previous).length() < 1.0 {
                    spawn_line(commands, particle, coords.current, brush_size);
                } else {
                    spawn_line_interpolated(
                        commands,
                        particle,
                        coords.previous,
                        coords.current,
                        brush_size,
                    );
                }
            }
            BrushType::Circle => {
                let particle = selected_brush_particle.clone();

                if (coords.previous - coords.previous_previous).length() < 1.0 {
                    spawn_circle(commands, particle.clone(), coords.previous, radius);
                } else {
                    spawn_capsule(
                        commands,
                        particle.clone(),
                        coords.previous,
                        coords.previous_previous,
                        radius,
                        half_length,
                    );
                }

                if (coords.current - coords.previous).length() < 1.0 {
                    spawn_circle(commands, particle, coords.current, radius);
                } else {
                    spawn_capsule(
                        commands,
                        particle,
                        coords.previous,
                        coords.current,
                        radius,
                        half_length,
                    );
                }
            }
            BrushType::Cursor => {
                let particle = selected_brush_particle.clone();

                // Interpolate between previous positions if movement is significant
                if (coords.previous - coords.previous_previous).length() >= 1.0 {
                    spawn_cursor_interpolated(
                        commands,
                        particle.clone(),
                        coords.previous_previous,
                        coords.previous,
                    );
                }

                if (coords.current - coords.previous).length() >= 1.0 {
                    spawn_cursor_interpolated(
                        commands,
                        particle.clone(),
                        coords.previous,
                        coords.current,
                    );
                }

                // Always spawn at current position
                commands.spawn((
                    particle.clone(),
                    Transform::from_xyz(coords.current.x.round(), coords.current.y.round(), 0.0),
                ));
            }
        }
    }

    pub fn remove_particles(
        &self,
        ev_remove_particle: &mut EventWriter<RemoveParticleEvent>,
        coords: IVec2,
        brush_size: f32,
    ) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                for x in min_x * 3..=max_x * 3 {
                    let position = IVec2::new(coords.x + x, coords.y);
                    ev_remove_particle.write(RemoveParticleEvent {
                        position,
                        despawn: true,
                    });
                }
            }
            BrushType::Circle => {
                let mut circle_coords: HashSet<IVec2> = HashSet::default();
                let circle = Circle::new(brush_size);
                for x in min_x * 2..=max_x * 2 {
                    for y in min_y * 2..=max_y * 2 {
                        let mut position = Vec2::new(x as f32, y as f32);
                        position = circle.closest_point(position);
                        circle_coords.insert((position + coords.as_vec2()).as_ivec2());
                    }
                }
                for position in circle_coords {
                    ev_remove_particle.write(RemoveParticleEvent {
                        position,
                        despawn: true,
                    });
                }
            }
            BrushType::Cursor => {
                let position = IVec2::new(coords.x, coords.y);
                ev_remove_particle.write(RemoveParticleEvent {
                    position,
                    despawn: true,
                });
            }
        }
    }
}

#[derive(Resource)]
pub struct SelectedBrushParticle(pub Particle);

pub fn toggle_brush_state(
    mut brush_state: ResMut<NextState<BrushState>>,
    current_state: Res<State<BrushState>>,
) {
    match current_state.get() {
        BrushState::Spawn => brush_state.set(BrushState::Despawn),
        BrushState::Despawn => brush_state.set(BrushState::Spawn),
    }
}

pub fn resize_brush_with_scroll(
    mut scroll_events: EventReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    keybindings: Res<BrushKeybindings>,
    mut brush_resize_events: EventWriter<BrushResizeEvent>,
    brush_query: Query<&Brush>,
) {
    if keys.pressed(keybindings.resize_modifier_key) {
        if let Ok(brush) = brush_query.single() {
            for event in scroll_events.read() {
                let current_size = brush.size as i32;
                let new_size = if event.y > 0.0 {
                    // Scroll up - increase size
                    (current_size + 1).min(50) // Cap at 50
                } else if event.y < 0.0 {
                    // Scroll down - decrease size
                    (current_size - 1).max(1) // Minimum size of 1
                } else {
                    current_size
                };

                if new_size != current_size {
                    brush_resize_events.write(BrushResizeEvent(new_size as usize));
                }
            }
        }
    }
}

pub fn handle_alt_app_state_transition(
    keys: Res<ButtonInput<KeyCode>>,
    keybindings: Res<BrushKeybindings>,
    mut app_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    if keys.pressed(keybindings.resize_modifier_key) {
        // LALT is held - transition to Ui if not already there
        if current_state.get() == &AppState::Canvas {
            app_state.set(AppState::Ui)
        }
    }
}

pub fn cycle_selected_particle(
    mut particle_spawn_list: ResMut<ParticleSpawnList>,
    mut selected_particle: ResMut<SelectedBrushParticle>,
) {
    if let Some(next_particle) = particle_spawn_list.cycle_next() {
        selected_particle.0 = next_particle.clone();
    }
}

pub fn cycle_brush_type(
    mut brush_type: ResMut<NextState<BrushType>>,
    current_type: Res<State<BrushType>>,
) {
    let next_type = current_type.get().cycle_next();
    brush_type.set(next_type);
}

pub fn setup_brush(
    mut commands: Commands,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = Brush::new(2, Color::WHITE);
    let brush_size = brush.size;
    commands.spawn(brush);
    brush_type.update_brush(cursor_coords.current, brush_size as f32, &mut brush_gizmos);
}

pub fn spawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    selected: Res<SelectedBrushParticle>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
) -> Result {
    let brush = brush_query.single()?;
    let brush_type = brush_type.get();
    brush_type.spawn_particles(
        &mut commands,
        cursor_coords,
        brush.size as f32,
        selected.0.clone(),
    );
    Ok(())
}

pub fn despawn_particles(
    mut ev_remove_particle: EventWriter<RemoveParticleEvent>,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
) -> Result {
    let brush = brush_query.single()?;
    let brush_size = brush.size;

    brush_type.remove_particles(
        &mut ev_remove_particle,
        cursor_coords.current.as_ivec2(),
        brush_size as f32,
    );
    Ok(())
}

pub fn update_brush(
    brush_query: Query<&Brush>,
    cursor_coords: Res<CursorCoords>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushType>>,
) -> Result {
    let brush = brush_query.single()?;
    brush_type.update_brush(cursor_coords.current, brush.size as f32, &mut brush_gizmos);
    Ok(())
}

fn sample_hovered(
    cursor_coords: Res<CursorCoords>,
    chunk_map: Res<ParticleMap>,
    particle_query: Query<&Particle>,
    mut selected_brush_particle: ResMut<SelectedBrushParticle>,
    mut brush_state: ResMut<NextState<BrushState>>,
) {
    if let Some(entity) = chunk_map.get(&cursor_coords.current.as_ivec2()) {
        let particle = particle_query.get(*entity).unwrap();
        selected_brush_particle.0 = particle.clone();
        brush_state.set(BrushState::Spawn);
    }
}

pub fn ev_resize_brush(
    mut ev_brush_resize: EventReader<BrushResizeEvent>,
    mut brush_query: Query<&mut Brush>,
) -> Result {
    let mut brush = brush_query.single_mut()?;
    for ev in ev_brush_resize.read() {
        brush.size = ev.0;
    }
    Ok(())
}

fn spawn_circle(commands: &mut Commands, particle: Particle, center: Vec2, radius: f32) {
    let mut points: HashSet<IVec2> = HashSet::default();

    let min_x = (center.x - radius).floor() as i32;
    let max_x = (center.x + radius).ceil() as i32;
    let min_y = (center.y - radius).floor() as i32;
    let max_y = (center.y + radius).ceil() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let point = Vec2::new(x as f32, y as f32);
            if (point - center).length() <= radius {
                points.insert(point.as_ivec2());
            }
        }
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn spawn_capsule(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    radius: f32,
    half_length: f32,
) {
    let capsule = Capsule2d {
        radius,
        half_length,
    };

    let points = points_within_capsule(&capsule, start, end);
    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn points_within_capsule(capsule: &Capsule2d, start: Vec2, end: Vec2) -> Vec<IVec2> {
    let mut points_inside = Vec::new();

    let min_x = (start.x.min(end.x) - capsule.radius).floor() as i32;
    let max_x = (start.x.max(end.x) + capsule.radius).ceil() as i32;
    let min_y = (start.y.min(end.y) - capsule.radius).floor() as i32;
    let max_y = (start.y.max(end.y) + capsule.radius).ceil() as i32;
    let capsule_direction = (end - start).normalize();

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let point = Vec2::new(x as f32, y as f32);

            let to_point = point - start;
            let projected_length = to_point.dot(capsule_direction);
            let clamped_length = projected_length.clamp(-capsule.half_length, capsule.half_length);

            let closest_point = start + capsule_direction * clamped_length;
            let distance_to_line = (point - closest_point).length();

            if distance_to_line <= capsule.radius {
                points_inside.push(IVec2::new(x, y));
            }
        }
    }

    points_inside
}

fn spawn_line(commands: &mut Commands, particle: Particle, center: Vec2, brush_size: f32) {
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size / 2.0) as i32;

    commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
        (
            particle.clone(),
            Transform::from_xyz((center.x + x as f32).round(), center.y.round(), 0.0),
        )
    }));
}

fn spawn_line_interpolated(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    brush_size: f32,
) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size / 2.0) as i32;

    // Sample points along the interpolated line
    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        // For each sample point, spawn a line
        for x in min_x * 3..=max_x * 3 {
            let position = Vec2::new((sample_point.x + x as f32).round(), sample_point.y.round());
            points.insert(position.as_ivec2());
        }
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn spawn_cursor_interpolated(commands: &mut Commands, particle: Particle, start: Vec2, end: Vec2) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();

    // Sample points along the interpolated path
    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        points.insert(IVec2::new(
            sample_point.x.round() as i32,
            sample_point.y.round() as i32,
        ));
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}
