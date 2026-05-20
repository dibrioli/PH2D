//! Project-level settings exposed by the editor.
//!
//! Holds the configuration that applies to the whole open project /
//! scene rather than to a single entity or panel: the canonical
//! example is `pixels_per_meter`, the source-image-to-world-space
//! scale used by the import pipeline.
//!
//! Stored on [`crate::HeroScreen`] (`hero.project`); read by the
//! shell when processing image imports (M14.4c, M14.4d) so a
//! `512×512` PNG lands as a sprite of `512 / px_per_meter` world
//! meters per side instead of a hardcoded `[1.5, 1.5]`. UI surface
//! lives in the TopBar Settings cluster (gear icon).
//!
//! Per-asset overrides (Inspector slider on a selected sprite) are
//! deferred — added when there's a real case of mixing scales in a
//! single scene.

pub use ph2d_host::ImageFilterMode;
use ph2d_vector::ImageQuality;

/// Defaults — kept as `pub const` so tests + UI presets can reference
/// the canonical value without re-typing the literal.
pub const DEFAULT_PIXELS_PER_METER: f32 = 100.0;

/// Min / max accepted by the UI slider. Outside this range the import
/// math either overflows world coordinates (px/m too low → a single
/// sprite spans kilometers) or collapses sprites to sub-pixel sizes
/// (px/m too high → 4096-px sprite becomes 4 millimeters wide).
pub const MIN_PIXELS_PER_METER: f32 = 1.0;
pub const MAX_PIXELS_PER_METER: f32 = 4096.0;

/// M14.7 F: snap-to-grid step in world meters used by the gizmo's
/// Ctrl/Cmd-translate path. 0.16 m = 16 px @ default 100 px/m, a
/// reasonable pixel-art tile size. `0.0` disables snap.
pub const DEFAULT_SNAP_MOVE_METERS: f32 = 0.16;

/// M14.7 F: snap-to-angle step in degrees used by the gizmo's
/// Shift-rotate path. 15° matches Figma/Affinity convention (15°,
/// 30°, 45°, … as the user holds Shift while rotating). `0.0`
/// disables rotation snap.
pub const DEFAULT_SNAP_ROTATE_DEG: f32 = 15.0;

/// User-visible unit for length / position readouts. Sim values are
/// always stored in **meters** (the world-space canonical) — this
/// enum only changes how those values are FORMATTED for the user in
/// panels (Inspector, Grid Settings, Gizmo tooltip).
///
/// Conversion uses `ProjectSettings::pixels_per_meter`:
///   pixels = meters × pixels_per_meter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayUnit {
    Meters,
    Pixels,
}

impl DisplayUnit {
    /// Convert a sim-stored meter value to the user-visible value in
    /// the active display unit.
    pub fn from_meters(self, meters: f32, pixels_per_meter: f32) -> f32 {
        match self {
            DisplayUnit::Meters => meters,
            DisplayUnit::Pixels => meters * pixels_per_meter,
        }
    }

    /// Inverse of [`Self::from_meters`] — convert a value the user
    /// typed (in display unit) back to sim-space meters before
    /// writing into ECS.
    pub fn to_meters(self, value: f32, pixels_per_meter: f32) -> f32 {
        match self {
            DisplayUnit::Meters => value,
            DisplayUnit::Pixels => {
                if pixels_per_meter > 0.0 {
                    value / pixels_per_meter
                } else {
                    value
                }
            }
        }
    }

    /// One-character suffix to show alongside a formatted value
    /// (`"m"` / `"px"`). Used by panel painters for "1.50 m" /
    /// "150 px" style readouts.
    pub fn suffix(self) -> &'static str {
        match self {
            DisplayUnit::Meters => "m",
            DisplayUnit::Pixels => "px",
        }
    }
}

/// Map the app-wide [`ImageFilterMode`] (defined in `ph2d-host`) onto
/// the peniko [`ImageQuality`] used by the Vello image preview
/// (`VectorScene::draw_image_rgba`). Companion of
/// `ph2d_render::wgpu_filter`, which maps the SAME enum onto the wgpu
/// sprite sampler — keeping the on-canvas BG-Removal preview consistent
/// with the baked sprite.
///
/// - `PixelArt` → [`ImageQuality::Low`]  (nearest-neighbor, crisp)
/// - `Smooth`   → [`ImageQuality::High`] (bicubic, smooth)
pub fn image_quality_for(mode: ImageFilterMode) -> ImageQuality {
    match mode {
        ImageFilterMode::PixelArt => ImageQuality::Low,
        ImageFilterMode::Smooth => ImageQuality::High,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectSettings {
    /// Source-image pixels per world meter. Default `100.0` matches
    /// Godot's convention: a `64×64` PNG becomes a `0.64 × 0.64 m`
    /// sprite in world space. Increase for HD 2D (256+ for hand-drawn
    /// 1080p assets); decrease for pixel art with large tile-to-meter
    /// ratios (16–32 for classic tile-based games).
    pub pixels_per_meter: f32,
    /// M14.7 F: snap step in world meters for Ctrl/Cmd-modified
    /// gizmo translate. Setting to `0.0` disables the snap (the
    /// modifier becomes a no-op). Configurable per-project so a
    /// pixel-art project can pin to 16/100 = 0.16 while a HD scene
    /// can crank it to 1.0 (= 1 m grid).
    pub snap_move_meters: f32,
    /// M14.7 F: snap step in degrees for Shift-modified gizmo
    /// rotate. `0.0` disables the snap.
    pub snap_rotate_deg: f32,
    /// User-visible unit for length / position readouts. Sim storage
    /// is always meters; this flips the FORMAT in Inspector / Grid
    /// Settings / Gizmo readouts between "m" and "px". Default
    /// `Meters` matches Godot / Bevy convention.
    pub display_unit: DisplayUnit,
    /// App-wide image sampling mode (Config → "Image filter"). The
    /// editor stores it so the Settings submenu can show the active
    /// pick with a checkmark; the SHELL is the source of truth for the
    /// GPU sampler state and drives it via `EditorAction::SetImageFilter`
    /// then `SpriteRenderer::set_filter_mode`. Default `PixelArt` — PH2D
    /// is a pixel-art-first editor.
    pub image_filter: ImageFilterMode,
}

impl ProjectSettings {
    /// Replace `pixels_per_meter`, clamping to the supported range so
    /// the UI can wire a NumberInput without re-implementing bounds
    /// checks. Returns the clamped value the field actually took.
    pub fn set_pixels_per_meter(&mut self, value: f32) -> f32 {
        let clamped = value.clamp(MIN_PIXELS_PER_METER, MAX_PIXELS_PER_METER);
        self.pixels_per_meter = clamped;
        clamped
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            pixels_per_meter: DEFAULT_PIXELS_PER_METER,
            snap_move_meters: DEFAULT_SNAP_MOVE_METERS,
            snap_rotate_deg: DEFAULT_SNAP_ROTATE_DEG,
            display_unit: DisplayUnit::Meters,
            image_filter: ImageFilterMode::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_godot_convention() {
        assert_eq!(ProjectSettings::default().pixels_per_meter, 100.0);
    }

    #[test]
    fn set_pixels_per_meter_clamps_below_min() {
        let mut s = ProjectSettings::default();
        let returned = s.set_pixels_per_meter(0.0);
        assert_eq!(returned, MIN_PIXELS_PER_METER);
        assert_eq!(s.pixels_per_meter, MIN_PIXELS_PER_METER);
    }

    #[test]
    fn set_pixels_per_meter_clamps_above_max() {
        let mut s = ProjectSettings::default();
        let returned = s.set_pixels_per_meter(10_000.0);
        assert_eq!(returned, MAX_PIXELS_PER_METER);
        assert_eq!(s.pixels_per_meter, MAX_PIXELS_PER_METER);
    }

    #[test]
    fn set_pixels_per_meter_passes_through_valid_values() {
        let mut s = ProjectSettings::default();
        for v in [1.0, 16.0, 32.0, 100.0, 256.0, 1024.0] {
            assert_eq!(s.set_pixels_per_meter(v), v);
            assert_eq!(s.pixels_per_meter, v);
        }
    }

    #[test]
    fn display_unit_round_trips_through_meters() {
        // Sim values are meters → to_meters(from_meters(m)) == m for
        // either unit, so the user can flip the toggle without losing
        // precision.
        for unit in [DisplayUnit::Meters, DisplayUnit::Pixels] {
            for m in [0.0, 0.16, 1.5, 64.0] {
                let displayed = unit.from_meters(m, 100.0);
                let back = unit.to_meters(displayed, 100.0);
                assert!(
                    (m - back).abs() < 1e-4,
                    "round trip failed for {unit:?} @ m={m}: displayed={displayed}, back={back}"
                );
            }
        }
    }

    #[test]
    fn display_unit_pixels_conversion_matches_pixels_per_meter() {
        // 1 m at 100 px/m → 100 px.
        assert_eq!(DisplayUnit::Pixels.from_meters(1.0, 100.0), 100.0);
        // 1 m at 32 px/m → 32 px.
        assert_eq!(DisplayUnit::Pixels.from_meters(1.0, 32.0), 32.0);
        // Meters mode ignores the px_per_m parameter.
        assert_eq!(DisplayUnit::Meters.from_meters(1.5, 999.0), 1.5);
    }

    #[test]
    fn display_unit_suffix() {
        assert_eq!(DisplayUnit::Meters.suffix(), "m");
        assert_eq!(DisplayUnit::Pixels.suffix(), "px");
    }

    #[test]
    fn default_display_unit_is_meters() {
        assert_eq!(ProjectSettings::default().display_unit, DisplayUnit::Meters);
    }
}
