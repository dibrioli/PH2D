//! Demo-tile generation: the golden-angle HSV swatches seeded by
//! [`TextureAtlas::dummy`] into keys `0..DEMO_TILE_COUNT`. Pure pixel
//! synthesis, no GPU — split out so the orchestrator only carries atlas
//! state, not the demo content recipe.
//!
//! [`TextureAtlas::dummy`]: super::TextureAtlas::dummy

/// Build a `DEMO_TILE_PX × DEMO_TILE_PX` RGBA tile filled with the
/// `index`-th golden-angle HSV color. Used by [`TextureAtlas::dummy`].
///
/// [`TextureAtlas::dummy`]: super::TextureAtlas::dummy
pub(super) fn make_hsv_tile(index: u32, side: u32) -> Vec<u8> {
    let hue = (index as f32 * 0.618_034) % 1.0;
    let (r, g, b) = hsv_to_rgb(hue, 0.85, 0.95);
    let mut out = vec![0u8; (side * side * 4) as usize];
    for chunk in out.chunks_exact_mut(4) {
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
        chunk[3] = 255;
    }
    out
}

pub(super) fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h6 = h * 6.0;
    let i = h6.floor() as i32 % 6;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::super::{DEMO_TILE_COUNT, DEMO_TILE_PX};
    use super::*;

    #[test]
    fn hsv_to_rgb_yields_distinct_colors_per_index() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for i in 0..DEMO_TILE_COUNT {
            let hue = (i as f32 * 0.618_034) % 1.0;
            seen.insert(hsv_to_rgb(hue, 0.85, 0.95));
        }
        assert_eq!(
            seen.len(),
            DEMO_TILE_COUNT as usize,
            "expected {DEMO_TILE_COUNT} distinct golden-angle hues, got {}",
            seen.len()
        );
    }

    #[test]
    fn make_hsv_tile_has_correct_size() {
        let tile = make_hsv_tile(0, DEMO_TILE_PX);
        assert_eq!(tile.len(), (DEMO_TILE_PX * DEMO_TILE_PX * 4) as usize);
        // Every pixel uniform — solid color.
        for chunk in tile.chunks_exact(4) {
            assert_eq!(chunk[0], tile[0]);
            assert_eq!(chunk[1], tile[1]);
            assert_eq!(chunk[2], tile[2]);
            assert_eq!(chunk[3], 255);
        }
    }
}
