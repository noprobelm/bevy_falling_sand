use crate::*;

/// Provides gizmos rendering for visualizing dead/alive chunks
pub fn color_chunks(map: Res<ChunkMap>, mut chunk_gizmos: Gizmos<DebugGizmos>) {
    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.sleeping() == true {
            chunk_gizmos.rect_2d(rect.center(), 0., rect.size() + Vec2::splat(1.), Color::srgba(0.67, 0.21, 0.24, 1.));
        }
    });

    map.iter_chunks().for_each(|chunk| {
        let rect = Rect::from_corners(chunk.min().as_vec2(), chunk.max().as_vec2());
        if chunk.sleeping() == false {
            chunk_gizmos.rect_2d(
                rect.center(),
                0.,
                rect.size() + Vec2::splat(1.),
                Color::srgba(0.52,0.80,0.51, 1.0),
            );
        }
    });
}
