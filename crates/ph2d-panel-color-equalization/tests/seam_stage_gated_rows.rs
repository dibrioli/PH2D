//! **Four rows that reached no consumer, in the state the panel is born
//! in** — and the gate that keeps them honest.
//!
//! ## The defect
//!
//! **Tile Grid**, **LUT Mix**, **Dither Strength** and **Dither Grain**
//! were painted and hit-indexed unconditionally, over pipeline stages
//! that are OFF at boot:
//!
//! | row | consumer | off because |
//! |---|---|---|
//! | Tile Grid | `clahe(..)` | `clip_limit == CLIP_LIMIT_MIN` |
//! | LUT Mix | `blend_luts(..)` | both LUT slots are `None` |
//! | Dither Strength | `posterize(..)` dither sub-pass | `posterize_levels == 0` |
//! | Dither Grain | idem | idem |
//!
//! The artist reaches all four in the first second and none of them
//! changes a pixel. ⚠️ **The Dither toggle itself is a fifth**, dead for
//! the same reason.
//!
//! ## Why nothing in this repo caught it
//!
//! Each value IS read — it is passed straight into the stage function —
//! so a "who reads this field?" search says *alive*. The click DOES
//! reach the tool, so the sibling `seam.rs` is green. The widget IS
//! focusable, so `architecture_panel_wiring_parity` is green. The id IS
//! asked about, so `the_painted_control_reaches_a_consumer` is green.
//! The only question that finds this shape is the third one: **does the
//! consumer ACT on the value, or discard it?**
//!
//! ## What this gate measures
//!
//! It paints the real panel and reads the HIT INDEX — the artist's
//! actual reach — then compares it, in the same expression, against the
//! pipeline's own stage predicate (`params::stage::*`, the identical
//! call `apply_color_equalization` makes). Painted-but-inert and
//! inert-but-hidden both fail.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_color_equalization::{ColorEqualizationPanel, ColorEqualizationPanelState, ids};
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_color_equalization::params::{
    CLIP_LIMIT_MAX, CLIP_LIMIT_MIN, ColorEqualizationUiSnapshot,
};
use ph2d_tool_color_equalization::stage;
use ph2d_ui_testkit::MockPanelHost;

/// Tall enough that a row is absent because of its predicate, never
/// because the body clipped it away.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Paint the panel with `snapshot` and return the ids the artist can
/// actually hit this frame.
fn reachable(snapshot: ColorEqualizationUiSnapshot) -> Vec<ph2d_a11y::NodeId> {
    let mut host = MockPanelHost::with_panel::<ColorEqualizationPanel>();
    let mut state = ColorEqualizationPanelState;
    ph2d_panel_color_equalization::set_current_snapshot(Some(snapshot));
    let hits = host.paint::<ColorEqualizationPanel>(&mut state, VIEWPORT);
    ph2d_panel_color_equalization::set_current_snapshot(None);
    hits.into_iter().map(|(id, _)| id).collect()
}

/// ⭐ **Tile Grid is reachable exactly when CLAHE runs.**
///
/// Both halves matter: with the stage off the row must be gone (it is
/// the defect), and with the stage on it must be back (a cure that
/// simply deletes the control is not a cure).
#[test]
fn the_tile_grid_row_is_reachable_exactly_when_clahe_runs() {
    for clip in [CLIP_LIMIT_MIN, 1.5, CLIP_LIMIT_MAX] {
        let snap = ColorEqualizationUiSnapshot {
            clip_limit: clip,
            ..ColorEqualizationUiSnapshot::default()
        };
        let ids_hit = reachable(snap);
        let painted = ids_hit.contains(&ids::CEQ_TILE_GRID);
        let runs = stage::clahe_runs(clip);
        assert_eq!(
            painted, runs,
            "clip_limit={clip}: Tile Grid painted={painted} but the CLAHE stage that \
             consumes tile_grid_size runs={runs}"
        );
        // The chip travels with its slider — one control, one fate.
        assert_eq!(
            ids_hit.contains(&ids::CEQ_TILE_GRID_NUM),
            runs,
            "clip_limit={clip}: the Tile Grid CHIP and its slider disagree"
        );
        // Control: Clip itself is the stage's on-switch and must never
        // vanish, or the artist loses the way back.
        assert!(
            ids_hit.contains(&ids::CEQ_CLIP_LIMIT),
            "clip_limit={clip}: Clip is the CLAHE on-switch and must stay reachable"
        );
    }
}

/// ⭐ **LUT Mix is reachable exactly when there are TWO cubes to blend.**
///
/// The subtle half: one filled slot is NOT enough. The pipeline applies
/// that cube directly and the mix is discarded — the fully-wired,
/// projected-out shape that no "is this field read?" probe can see.
#[test]
fn the_lut_mix_row_is_reachable_exactly_when_two_cubes_are_blended() {
    for a in [LutPreset::None, LutPreset::Warm] {
        for b in [LutPreset::None, LutPreset::Cool] {
            for intensity in [0.0f32, 1.0] {
                let snap = ColorEqualizationUiSnapshot {
                    lut_preset_1: a,
                    lut_preset_2: b,
                    lut_intensity: intensity,
                    ..ColorEqualizationUiSnapshot::default()
                };
                let ids_hit = reachable(snap);
                let painted = ids_hit.contains(&ids::CEQ_LUT_MIX);
                let runs = stage::lut_blend_runs(intensity, a, b);
                assert_eq!(
                    painted, runs,
                    "presets=({a:?},{b:?}) intensity={intensity}: LUT Mix painted={painted} \
                     but the blend that consumes lut_mix runs={runs}"
                );
                assert_eq!(
                    ids_hit.contains(&ids::CEQ_LUT_MIX_NUM),
                    runs,
                    "presets=({a:?},{b:?}) intensity={intensity}: chip and slider disagree"
                );
            }
        }
    }
}

/// ⭐ **The Dither pair needs TWO facts, and so does its row.**
///
/// Posterize on AND Dither on. The toggle itself is gated on the first
/// fact alone — it is the control that supplies the second.
#[test]
fn the_dither_rows_are_reachable_exactly_when_the_dither_pass_runs() {
    for levels in [0u32, 4, 16] {
        for dithering in [false, true] {
            let snap = ColorEqualizationUiSnapshot {
                posterize_levels: levels,
                posterize_dithering: dithering,
                ..ColorEqualizationUiSnapshot::default()
            };
            let ids_hit = reachable(snap);
            let pass_runs = stage::dither_runs(levels, dithering);
            for id in [
                ids::CEQ_POSTERIZE_DITHER_STRENGTH,
                ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
                ids::CEQ_POSTERIZE_DITHER_GRAIN,
                ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            ] {
                assert_eq!(
                    ids_hit.contains(&id),
                    pass_runs,
                    "levels={levels} dithering={dithering}: a dither knob is painted={} \
                     while the dither pass that consumes it runs={pass_runs}",
                    ids_hit.contains(&id)
                );
            }
            // The toggle is one door up: it only needs a Posterize stage
            // to modify.
            assert_eq!(
                ids_hit.contains(&ids::CEQ_POSTERIZE_DITHERING),
                stage::posterize_runs(levels),
                "levels={levels}: the Dither toggle and the Posterize stage disagree"
            );
            // Control: Posterize is the on-switch and never vanishes.
            assert!(
                ids_hit.contains(&ids::CEQ_POSTERIZE_DROPDOWN),
                "levels={levels}: Posterize is the on-switch and must stay reachable"
            );
        }
    }
}

/// ⚠️ **The state the artist is actually born in.** Pinned on its own so
/// a regression reads as one line: opening the panel and touching
/// nothing must not offer five controls that change no pixel.
#[test]
fn the_panel_as_it_is_born_offers_no_inert_control() {
    let ids_hit = reachable(ColorEqualizationUiSnapshot::default());
    for (id, name) in [
        (ids::CEQ_TILE_GRID, "Tile Grid"),
        (ids::CEQ_TILE_GRID_NUM, "Tile Grid chip"),
        (ids::CEQ_LUT_MIX, "LUT Mix"),
        (ids::CEQ_LUT_MIX_NUM, "LUT Mix chip"),
        (ids::CEQ_POSTERIZE_DITHERING, "Dither toggle"),
        (ids::CEQ_POSTERIZE_DITHER_STRENGTH, "Dither Strength"),
        (ids::CEQ_POSTERIZE_DITHER_GRAIN, "Dither Grain"),
    ] {
        assert!(
            !ids_hit.contains(&id),
            "{name} is reachable in the default panel state and its stage does not run"
        );
    }
    // And the panel is not empty — the on-switches are all there.
    for (id, name) in [
        (ids::CEQ_CLIP_LIMIT, "Clip"),
        (ids::CEQ_LUT_INTENSITY, "LUT Intensity"),
        (ids::CEQ_POSTERIZE_DROPDOWN, "Posterize"),
    ] {
        assert!(
            ids_hit.contains(&id),
            "{name} must stay reachable — it is how the artist turns the stage on"
        );
    }
}
