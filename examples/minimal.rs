use std::time::Duration;

use bevy::prelude::*;
use bevy_falling_sand::prelude::FallingSandPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default()
                .with_length_unit(8.0)
                .with_spatial_refresh_frequency(Duration::from_millis(50)),
        ))
        .run();
}
