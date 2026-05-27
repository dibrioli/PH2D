//! Widget `NodeId`s for the Color Equalization panel.
//!
//! Defined here in the tool crate (NOT in `ph2d-editor-core::ids`) so the
//! fan-out path (DIRETRIZ §3.8) stays pure: the new tool drops its own
//! pasta + the panel crate-irmão re-exports these via `pub use`. Tool's
//! `handle_panel_event` matches against `crate::ids::*`; the panel's
//! `populate` / `event` consume `ph2d_tool_color_equalization::ids::*`.
//!
//! All ids are derived via `hash_node_id("color_eq.<chip>")` (FNV-1a 64);
//! the [`node_id_collisions`] arch test in `ph2d-tool-registry` catches
//! accidental hash collisions across the project.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Panel root ────────────────────────────────────────────────────
pub const CEQ_PANEL: NodeId = hash_node_id("panel.color_equalization");

// ── Sliders + paired numeric chips ────────────────────────────────
// Track is normalized 0..1; the chip stores the displayed value in
// the slider's natural unit (clip limit, tile count, EV stops, …).
// link_slider_number couples them in 0..1 space; chip paint uses
// `display_override` to format the natural unit.
pub const CEQ_CLIP_LIMIT: NodeId = hash_node_id("color_eq.clip_limit");
pub const CEQ_CLIP_LIMIT_NUM: NodeId = hash_node_id("color_eq.clip_limit.num");

pub const CEQ_TILE_GRID: NodeId = hash_node_id("color_eq.tile_grid_size");
pub const CEQ_TILE_GRID_NUM: NodeId = hash_node_id("color_eq.tile_grid_size.num");

pub const CEQ_EXPOSURE: NodeId = hash_node_id("color_eq.exposure");
pub const CEQ_EXPOSURE_NUM: NodeId = hash_node_id("color_eq.exposure.num");

pub const CEQ_TEMPERATURE: NodeId = hash_node_id("color_eq.temperature");
pub const CEQ_TEMPERATURE_NUM: NodeId = hash_node_id("color_eq.temperature.num");

pub const CEQ_TINT: NodeId = hash_node_id("color_eq.tint");
pub const CEQ_TINT_NUM: NodeId = hash_node_id("color_eq.tint.num");

pub const CEQ_BRIGHTNESS: NodeId = hash_node_id("color_eq.brightness");
pub const CEQ_BRIGHTNESS_NUM: NodeId = hash_node_id("color_eq.brightness.num");

pub const CEQ_CONTRAST: NodeId = hash_node_id("color_eq.contrast");
pub const CEQ_CONTRAST_NUM: NodeId = hash_node_id("color_eq.contrast.num");

pub const CEQ_VIBRANCE: NodeId = hash_node_id("color_eq.vibrance");
pub const CEQ_VIBRANCE_NUM: NodeId = hash_node_id("color_eq.vibrance.num");

pub const CEQ_SATURATION: NodeId = hash_node_id("color_eq.saturation");
pub const CEQ_SATURATION_NUM: NodeId = hash_node_id("color_eq.saturation.num");

// ── Phase 2 effects ───────────────────────────────────────────────
pub const CEQ_SHARPEN_AMOUNT: NodeId = hash_node_id("color_eq.sharpen_amount");
pub const CEQ_SHARPEN_AMOUNT_NUM: NodeId = hash_node_id("color_eq.sharpen_amount.num");

pub const CEQ_SHARPEN_RADIUS: NodeId = hash_node_id("color_eq.sharpen_radius");
pub const CEQ_SHARPEN_RADIUS_NUM: NodeId = hash_node_id("color_eq.sharpen_radius.num");

// ── Phase 3 LUT color grading ─────────────────────────────────────
// Two grouped-select dropdowns (legacy `createGroupedSelect` parity:
// Cinematic / Atmosphere / Vintage / Stylized header rows separating
// the 15 presets + an explicit `None` slot) + two slider+chip pairs
// for intensity and mix. Chip paint reads the active preset's label
// from the snapshot.
pub const CEQ_LUT_1_DROPDOWN: NodeId = hash_node_id("color_eq.lut_1.dropdown");
pub const CEQ_LUT_2_DROPDOWN: NodeId = hash_node_id("color_eq.lut_2.dropdown");

// Per-slot option NodeIds (16 presets × 2 slots = 32). Hit-registered
// by paint while the popover is open; routed back to
// `SetLutPreset1` / `SetLutPreset2` by `handle_panel_event`.
//
// The arrays are aligned to [`LutPreset::ALL`] — index 0 = None,
// index 1 = Cinematic, etc. — so the panel can look up an option's id
// without a per-preset match arm.
pub const CEQ_LUT_1_OPTS: [NodeId; 16] = [
    hash_node_id("color_eq.lut_1.opt.none"),
    hash_node_id("color_eq.lut_1.opt.cinematic"),
    hash_node_id("color_eq.lut_1.opt.blockbuster"),
    hash_node_id("color_eq.lut_1.opt.film-noir"),
    hash_node_id("color_eq.lut_1.opt.warm"),
    hash_node_id("color_eq.lut_1.opt.cool"),
    hash_node_id("color_eq.lut_1.opt.golden-hour"),
    hash_node_id("color_eq.lut_1.opt.moonlight"),
    hash_node_id("color_eq.lut_1.opt.vintage"),
    hash_node_id("color_eq.lut_1.opt.sepia"),
    hash_node_id("color_eq.lut_1.opt.faded-film"),
    hash_node_id("color_eq.lut_1.opt.polaroid"),
    hash_node_id("color_eq.lut_1.opt.vibrant"),
    hash_node_id("color_eq.lut_1.opt.matte"),
    hash_node_id("color_eq.lut_1.opt.bleach-bypass"),
    hash_node_id("color_eq.lut_1.opt.cross-process"),
];
pub const CEQ_LUT_2_OPTS: [NodeId; 16] = [
    hash_node_id("color_eq.lut_2.opt.none"),
    hash_node_id("color_eq.lut_2.opt.cinematic"),
    hash_node_id("color_eq.lut_2.opt.blockbuster"),
    hash_node_id("color_eq.lut_2.opt.film-noir"),
    hash_node_id("color_eq.lut_2.opt.warm"),
    hash_node_id("color_eq.lut_2.opt.cool"),
    hash_node_id("color_eq.lut_2.opt.golden-hour"),
    hash_node_id("color_eq.lut_2.opt.moonlight"),
    hash_node_id("color_eq.lut_2.opt.vintage"),
    hash_node_id("color_eq.lut_2.opt.sepia"),
    hash_node_id("color_eq.lut_2.opt.faded-film"),
    hash_node_id("color_eq.lut_2.opt.polaroid"),
    hash_node_id("color_eq.lut_2.opt.vibrant"),
    hash_node_id("color_eq.lut_2.opt.matte"),
    hash_node_id("color_eq.lut_2.opt.bleach-bypass"),
    hash_node_id("color_eq.lut_2.opt.cross-process"),
];

pub const CEQ_LUT_INTENSITY: NodeId = hash_node_id("color_eq.lut_intensity");
pub const CEQ_LUT_INTENSITY_NUM: NodeId = hash_node_id("color_eq.lut_intensity.num");

pub const CEQ_LUT_MIX: NodeId = hash_node_id("color_eq.lut_mix");
pub const CEQ_LUT_MIX_NUM: NodeId = hash_node_id("color_eq.lut_mix.num");

// ── Toggles + buttons ─────────────────────────────────────────────
pub const CEQ_AUTO_LEVELS: NodeId = hash_node_id("color_eq.auto_levels");
pub const CEQ_AUTO_CONTRAST: NodeId = hash_node_id("color_eq.auto_contrast");
pub const CEQ_AUTO_COLORS: NodeId = hash_node_id("color_eq.auto_colors");
pub const CEQ_AUTO_WB: NodeId = hash_node_id("color_eq.auto_wb");
pub const CEQ_APPLY: NodeId = hash_node_id("color_eq.apply");
pub const CEQ_CANCEL: NodeId = hash_node_id("color_eq.cancel");
/// Reset-all button: returns every param to its default in a single
/// click. Fires `ColorEqualizationUiEdit::ResetAll` (see params.rs);
/// the tool also runs this from `on_activate` so reopening the panel
/// never inherits a previous session's slider state.
pub const CEQ_RESET: NodeId = hash_node_id("color_eq.reset");

// ── Phase 5 Posterize / Quantize ──────────────────────────────────
// Posterize: dropdown chip (Off / 2 / 3 / 4 / 6 / 8 / 16) + dithering
// toggle. Quantize: dropdown chip (Off / 4 / 8 / 16 / 32 / 64 / 128 /
// 256). Same dropdown pattern as LUT slots — chip toggles `open`,
// option click routes to `SetPosterizeLevels` / `SetQuantizeColors`
// and stages a one-shot close.
pub const CEQ_POSTERIZE_DROPDOWN: NodeId = hash_node_id("color_eq.posterize.dropdown");
pub const CEQ_POSTERIZE_DITHERING: NodeId = hash_node_id("color_eq.posterize.dithering");
/// Dither strength slider (Enio 2026-05-26): controla a intensidade da
/// difusão de erro Floyd-Steinberg sobre o posterize.
pub const CEQ_POSTERIZE_DITHER_STRENGTH: NodeId =
    hash_node_id("color_eq.posterize.dither_strength");
pub const CEQ_POSTERIZE_DITHER_STRENGTH_NUM: NodeId =
    hash_node_id("color_eq.posterize.dither_strength.num");
/// Dither grain slider (Enio 2026-05-26): tile size `1..=8` para grão
/// chunky de dither (estilo pixel-art).
pub const CEQ_POSTERIZE_DITHER_GRAIN: NodeId = hash_node_id("color_eq.posterize.dither_grain");
pub const CEQ_POSTERIZE_DITHER_GRAIN_NUM: NodeId =
    hash_node_id("color_eq.posterize.dither_grain.num");
pub const CEQ_QUANTIZE_DROPDOWN: NodeId = hash_node_id("color_eq.quantize.dropdown");

/// Posterize options in panel order. Index 0 = off (level `0`); else
/// `2..=16` matching the legacy panel's discrete picks.
pub const CEQ_POSTERIZE_LEVELS: [u32; 7] = [0, 2, 3, 4, 6, 8, 16];
pub const CEQ_POSTERIZE_OPTS: [NodeId; 7] = [
    hash_node_id("color_eq.posterize.opt.off"),
    hash_node_id("color_eq.posterize.opt.2"),
    hash_node_id("color_eq.posterize.opt.3"),
    hash_node_id("color_eq.posterize.opt.4"),
    hash_node_id("color_eq.posterize.opt.6"),
    hash_node_id("color_eq.posterize.opt.8"),
    hash_node_id("color_eq.posterize.opt.16"),
];

/// Quantize options in panel order. Index 0 = off (colours `0`); else
/// `4..=256` matching the legacy panel.
pub const CEQ_QUANTIZE_COLORS: [u32; 8] = [0, 4, 8, 16, 32, 64, 128, 256];
pub const CEQ_QUANTIZE_OPTS: [NodeId; 8] = [
    hash_node_id("color_eq.quantize.opt.off"),
    hash_node_id("color_eq.quantize.opt.4"),
    hash_node_id("color_eq.quantize.opt.8"),
    hash_node_id("color_eq.quantize.opt.16"),
    hash_node_id("color_eq.quantize.opt.32"),
    hash_node_id("color_eq.quantize.opt.64"),
    hash_node_id("color_eq.quantize.opt.128"),
    hash_node_id("color_eq.quantize.opt.256"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct() {
        let all = [
            CEQ_PANEL,
            CEQ_CLIP_LIMIT,
            CEQ_CLIP_LIMIT_NUM,
            CEQ_TILE_GRID,
            CEQ_TILE_GRID_NUM,
            CEQ_EXPOSURE,
            CEQ_EXPOSURE_NUM,
            CEQ_TEMPERATURE,
            CEQ_TEMPERATURE_NUM,
            CEQ_TINT,
            CEQ_TINT_NUM,
            CEQ_BRIGHTNESS,
            CEQ_BRIGHTNESS_NUM,
            CEQ_CONTRAST,
            CEQ_CONTRAST_NUM,
            CEQ_VIBRANCE,
            CEQ_VIBRANCE_NUM,
            CEQ_SATURATION,
            CEQ_SATURATION_NUM,
            CEQ_SHARPEN_AMOUNT,
            CEQ_SHARPEN_AMOUNT_NUM,
            CEQ_SHARPEN_RADIUS,
            CEQ_SHARPEN_RADIUS_NUM,
            CEQ_LUT_1_DROPDOWN,
            CEQ_LUT_2_DROPDOWN,
            CEQ_LUT_INTENSITY,
            CEQ_LUT_INTENSITY_NUM,
            CEQ_LUT_MIX,
            CEQ_LUT_MIX_NUM,
            CEQ_AUTO_LEVELS,
            CEQ_AUTO_CONTRAST,
            CEQ_AUTO_COLORS,
            CEQ_AUTO_WB,
            CEQ_APPLY,
            CEQ_CANCEL,
            CEQ_RESET,
            CEQ_POSTERIZE_DROPDOWN,
            CEQ_POSTERIZE_DITHERING,
            CEQ_POSTERIZE_DITHER_STRENGTH,
            CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            CEQ_POSTERIZE_DITHER_GRAIN,
            CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            CEQ_QUANTIZE_DROPDOWN,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate Color EQ NodeId: {a:?} == {b:?}");
            }
        }
        // Option ids are also distinct from `all` AND distinct from each
        // other across slots (LUT 1 vs LUT 2 must never collide — a
        // click on LUT 2's "Cool" option must NOT route to LUT 1).
        // Same property holds for Posterize / Quantize option ids.
        let opts: Vec<NodeId> = CEQ_LUT_1_OPTS
            .iter()
            .chain(CEQ_LUT_2_OPTS.iter())
            .chain(CEQ_POSTERIZE_OPTS.iter())
            .chain(CEQ_QUANTIZE_OPTS.iter())
            .copied()
            .collect();
        for (i, a) in opts.iter().enumerate() {
            for b in opts.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate Color EQ option NodeId: {a:?} == {b:?}");
            }
            for b in all.iter() {
                assert_ne!(
                    a, b,
                    "Color EQ option NodeId collides with chrome id: {a:?} == {b:?}"
                );
            }
        }
    }
}
