//! Vector-tool Style UI vocabulary — the snapshot the docked
//! `ph2d-panel-vector` paints, plus the Width slider ↔ px mapping shared by
//! the panel (populate/paint) and the tool (`handle_panel_event`).
//!
//! Mirrors `ph2d_tool_padding::params`: the tool owns the authoritative Style,
//! projects it into a [`VectorStyleSnapshot`] each frame (published by the
//! shell bridge → the panel reads it), and both sides agree on the affine
//! slider mapping so a drag and the tool stay in lock-step.

/// Minimum / maximum stroke width in screen pixels (inclusive range the Width
/// slider spans).
pub const WIDTH_MIN_PX: f64 = 1.0;
pub const WIDTH_MAX_PX: f64 = 20.0;

/// Affine slider mapping `display_px = track * SCALE + OFFSET` (track `0..=1`),
/// consumed by `WidgetStore::link_slider_number_mapped` so the px chip mirrors
/// the slider. `SCALE = MAX - MIN`, `OFFSET = MIN`.
pub const WIDTH_SLIDER_SCALE: f32 = (WIDTH_MAX_PX - WIDTH_MIN_PX) as f32;
pub const WIDTH_SLIDER_OFFSET: f32 = WIDTH_MIN_PX as f32;

/// Normalized slider track `0..=1` → stroke width px `MIN..=MAX`.
#[must_use]
pub fn slider_to_px(track: f32) -> f64 {
    WIDTH_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (WIDTH_MAX_PX - WIDTH_MIN_PX)
}

/// Stroke width px → normalized slider track `0..=1` (inverse of
/// [`slider_to_px`]). Used to seed the slider knob from the tool's authoritative
/// width so it renders correctly before the first drag.
#[must_use]
pub fn px_to_slider(px: f64) -> f32 {
    (((px - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32).clamp(0.0, 1.0)
}

/// Per-frame projection of the tool's Style, published by the shell bridge for
/// the docked panel to paint. `stroke` / `fill` are sRGB8; `fill[3] == 0` ⇒ no
/// fill ("None").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorStyleSnapshot {
    pub stroke: [u8; 4],
    pub fill: [u8; 4],
    pub stroke_width_px: f64,
}

impl Default for VectorStyleSnapshot {
    fn default() -> Self {
        Self {
            stroke: [240, 240, 245, 255],
            fill: [90, 150, 230, 255],
            stroke_width_px: super::tool::DEFAULT_STROKE_WIDTH_PX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_px_round_trip_endpoints() {
        assert_eq!(slider_to_px(0.0), WIDTH_MIN_PX);
        assert_eq!(slider_to_px(1.0), WIDTH_MAX_PX);
        assert!((px_to_slider(WIDTH_MIN_PX) - 0.0).abs() < 1e-6);
        assert!((px_to_slider(WIDTH_MAX_PX) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn slider_mapping_matches_affine_consts() {
        // The panel's chip display uses `track * SCALE + OFFSET`; it must equal
        // the tool's `slider_to_px` for the chip to mirror the slider exactly.
        for &t in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let via_affine = f64::from(t * WIDTH_SLIDER_SCALE + WIDTH_SLIDER_OFFSET);
            assert!((via_affine - slider_to_px(t)).abs() < 1e-6);
        }
    }
}
