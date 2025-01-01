use bevy::prelude::Component;

pub trait ParticleBlueprint: Component {
    type Data: Component;

    fn data(&self) -> &Self::Data;
}

#[macro_export]
macro_rules! impl_particle_blueprint {
    ($struct_name:ident, $data_type:ty) => {
        impl ParticleBlueprint for $struct_name {
            type Data = $data_type;

            fn data(&self) -> &Self::Data {
                &self.0
            }
        }
    };
}
