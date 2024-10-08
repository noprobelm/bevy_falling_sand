use crate::*;
use ron::de::from_reader;
use std::fs::File;
use std::io::Write;

pub fn save_scene_system(
    particle_query: Query<(&ParticleType, &Coordinates)>,
    mut ev_save_scene: EventReader<SaveSceneEvent>,
) {
    for ev in ev_save_scene.read() {
        let particles: Vec<ParticleData> = particle_query
            .iter()
            .map(|(particle_type, coordinates)| ParticleData {
                particle_type: particle_type.clone(),
                coordinates: *coordinates,
            })
            .collect();

        let particle_scene = ParticleScene { particles };
        let ron_string = ron::to_string(&particle_scene).unwrap();
        File::create(ev.0.clone())
            .and_then(|mut file| file.write(ron_string.as_bytes()))
            .expect("Error while writing scene to file");
    }
}

pub fn load_scene_system(mut commands: Commands, mut ev_load_scene: EventReader<LoadSceneEvent>) {
    for ev in ev_load_scene.read() {
        let file = File::open(ev.0.clone()).expect("Failed to open RON file");
        let particle_scene: ParticleScene = from_reader(file).expect("Failed to load RON file");

        for particle_data in particle_scene.particles {
            let transform = Transform::from_xyz(
                particle_data.coordinates.0.x as f32,
                particle_data.coordinates.0.y as f32,
                0.,
            );

            commands.spawn((
                particle_data.particle_type.clone(),
                SpatialBundle::from_transform(transform),
            ));
        }
    }
}
