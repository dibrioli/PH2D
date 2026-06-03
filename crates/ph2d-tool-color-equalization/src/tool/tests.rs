//! Unit tests for [`ColorEqualizationTool`] — split out of `tool/mod.rs`.
//! Covers panel-event routing, preview lifecycle, drain flags, full-res
//! bake, and the `RasterEditTool` generic-channel impl (ADR-0041).

#![cfg(test)]

use super::{ColorEqualizationTool, PREVIEW_MAX_DIM};
use crate::ids;
use crate::params::{ColorEqualizationUiEdit, ColorEqualizationUiSnapshot};
use ph2d_editor_core::floating_panel::ToolId;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};

fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    v
}

#[test]
fn default_tool_has_no_source_and_no_pending() {
    let mut t = ColorEqualizationTool::default();
    assert!(!t.has_source());
    assert_eq!(t.preview_rgba().0.len(), 0);
    assert!(!t.take_pending_apply());
    assert!(!t.take_params_dirty());
}

#[test]
fn id_label_icon() {
    let t = ColorEqualizationTool::default();
    assert_eq!(t.id(), ToolId::new("color_equalization"));
    assert_eq!(t.label(), "Color Equalization");
    assert_eq!(t.icon_slug(), "color-equalization");
}

#[test]
fn ui_snapshot_mirrors_defaults() {
    let t = ColorEqualizationTool::default();
    let s = t.ui_snapshot();
    let dft = ColorEqualizationUiSnapshot::default();
    assert_eq!(s, dft);
}

#[test]
fn slider_event_updates_params() {
    let mut t = ColorEqualizationTool::default();
    // Brightness slider mid-range (0.75 → 0.5 brightness).
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 0.75));
    assert!((t.params.brightness - 0.5).abs() < 1e-5);
    // Tile grid chip clamps.
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_TILE_GRID_NUM, 100.0));
    assert_eq!(t.params.tile_grid_size, 16);
}

#[test]
fn apply_arms_pending_once() {
    let mut t = ColorEqualizationTool::default();
    assert!(!t.take_pending_apply());
    t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
    assert!(t.take_pending_apply());
    assert!(!t.take_pending_apply());
}

#[test]
fn auto_wb_toggle_event_flips_param() {
    let mut t = ColorEqualizationTool::default();
    assert!(!t.params.auto_wb);
    t.handle_panel_event(PanelEvent::Click(ids::CEQ_AUTO_WB));
    assert!(t.params.auto_wb);
    t.handle_panel_event(PanelEvent::Toggle(ids::CEQ_AUTO_WB, false));
    assert!(!t.params.auto_wb);
}

#[test]
fn deactivate_clears_pending_but_keeps_params() {
    let mut t = ColorEqualizationTool::default();
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
    t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
    t.on_deactivate();
    assert!(!t.take_pending_apply());
    assert!(!t.take_params_dirty());
    // Params persist.
    assert!((t.params.brightness - 1.0).abs() < 1e-5);
}

#[test]
fn set_source_snapshot_marks_has_source_true() {
    let mut t = ColorEqualizationTool::default();
    let buf = solid(8, 8, [120, 80, 200]);
    t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 8, 8);
    assert!(t.has_source());
    assert_eq!(t.source_size(), (8, 8));
}

#[test]
fn preview_is_built_lazily_after_param_edit() {
    let mut t = ColorEqualizationTool::default();
    t.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(8, 8, [180, 120, 60])),
        8,
        8,
    );
    // Drain the initial dirty marker that source-push armed.
    let _ = t.preview_rgba();
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
    let (rgba, w, h) = t.preview_rgba();
    assert_eq!(w, 8);
    assert_eq!(h, 8);
    assert!(!rgba.is_empty());
    // Pixels should have been lifted by brightness.
    assert!(rgba[0] > 180);
}

#[test]
fn preview_caps_at_max_dim_for_large_sources() {
    let mut t = ColorEqualizationTool::default();
    // 1024² source → preview at 512² (PREVIEW_MAX_DIM).
    t.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(1024, 1024, [128, 128, 128])),
        1024,
        1024,
    );
    let (rgba, w, h) = t.preview_rgba();
    assert_eq!(w, PREVIEW_MAX_DIM);
    assert_eq!(h, PREVIEW_MAX_DIM);
    assert_eq!(rgba.len(), (PREVIEW_MAX_DIM * PREVIEW_MAX_DIM * 4) as usize);
}

#[test]
fn run_full_resolution_returns_source_dims() {
    let mut t = ColorEqualizationTool::default();
    t.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(7, 11, [50, 100, 200])),
        7,
        11,
    );
    let mut out = Vec::new();
    let (w, h) = t.run_full_resolution(&mut out);
    assert_eq!((w, h), (7, 11));
    assert_eq!(out.len(), 7 * 11 * 4);
}

#[test]
fn run_full_resolution_works_after_per_sprite_source_swap() {
    // Mirrors the shell drain pattern: one ColorEqualizationTool
    // instance bakes N sprites in sequence via
    // set_source_snapshot → run_full_resolution per entity. Each
    // bake must reflect the CURRENT snapshot, not leftover state
    // from a previous sprite. This is the contract `drain_color_
    // equalization` depends on for multi-select Apply.
    let mut t = ColorEqualizationTool::default();
    // Apply non-trivial params so the pipeline actually runs (defaults
    // would still bake CLAHE, but we force a tonal change too).
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));

    // Sprite 1: red source.
    t.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(4, 4, [200, 50, 30])),
        4,
        4,
    );
    let mut out1 = Vec::new();
    let (w1, h1) = t.run_full_resolution(&mut out1);
    assert_eq!((w1, h1), (4, 4));
    let red_first_px = out1[0];

    // Sprite 2: different size + colour. The tool MUST re-bake
    // against the fresh snapshot, not reuse out1's dims/pixels.
    t.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(8, 6, [30, 200, 50])),
        8,
        6,
    );
    let mut out2 = Vec::new();
    let (w2, h2) = t.run_full_resolution(&mut out2);
    assert_eq!((w2, h2), (8, 6));
    assert_eq!(out2.len(), 8 * 6 * 4);
    // Pixel 0 of sprite 2 starts from green source — must differ
    // from sprite 1's red pixel under the same brightness boost.
    assert_ne!(red_first_px, out2[0], "per-sprite source swap leaked state");
    // Sprite 2 should have a green-dominant pixel after boost.
    assert!(out2[1] > out2[0]);
    assert!(out2[1] > out2[2]);
}

#[test]
fn on_activate_resets_params_and_arms_panel_repopulate() {
    // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` MUST
    // route through `apply_ui_edit::ResetAll` so (a) params snap to
    // defaults AND (b) `pending_panel_reset` arms so the shell
    // bridge re-runs `Panel::populate(store)` and the slider knobs
    // visually snap back. Direct mutation of `self.params` bypasses
    // the flag and leaves sliders stuck — that bug shipped twice
    // this session.
    let mut t = ColorEqualizationTool::default();
    // Dirty the state — simulate a prior session.
    t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
    t.handle_panel_event(PanelEvent::Click(ids::CEQ_AUTO_WB));
    assert!((t.params.brightness - 1.0).abs() < 1e-5);
    assert!(t.params.auto_wb);
    // Drain any stray reset flag first.
    let _ = t.take_pending_panel_reset();

    t.on_activate();

    let dft = ColorEqualizationUiSnapshot::default();
    assert_eq!(t.ui_snapshot(), dft, "on_activate must reset params");
    assert!(
        t.take_pending_panel_reset(),
        "on_activate must arm pending_panel_reset"
    );
}

#[test]
fn downcast_via_as_any_mut_round_trips() {
    // Mirrors the shell's downcast path for raster I/O (DIRETRIZ
    // §3.8.3.1 — production tools still use this pattern).
    let mut boxed: Box<dyn Tool> = Box::new(ColorEqualizationTool::default());
    let any = boxed.as_any_mut();
    let tool = any.downcast_mut::<ColorEqualizationTool>().unwrap();
    tool.set_source_snapshot(
        bytemuck::allocation::cast_vec(solid(4, 4, [10, 20, 30])),
        4,
        4,
    );
    assert!(tool.has_source());
}

// ─────────────────────────────────────────────────────────────────────
// Wave 10 / Etapa 2 — RasterEditTool impl tests (ADR-0041 follow-up)
// ─────────────────────────────────────────────────────────────────────

#[test]
fn as_raster_edit_mut_returns_some_for_ceq() {
    let mut t = ColorEqualizationTool::default();
    assert!(<dyn Tool as Tool>::as_raster_edit_mut(&mut t).is_some());
}

#[test]
fn raster_edit_set_source_delegates() {
    let mut t = ColorEqualizationTool::default();
    RasterEditTool::set_source(&mut t, solid(8, 8, [100, 150, 200]), 8, 8);
    assert!(t.has_source());
    assert_eq!(t.source_size(), (8, 8));
}

#[test]
fn raster_edit_current_preview_drains_dirty() {
    let mut t = ColorEqualizationTool::default();
    RasterEditTool::set_source(&mut t, solid(8, 8, [128, 128, 128]), 8, 8);
    // First call after set_source: dirty drains, preview returned.
    let frame = RasterEditTool::current_preview(&mut t);
    assert!(
        frame.is_some(),
        "first call after set_source must return Some"
    );
    let (pixels, w, h) = frame.unwrap();
    assert!(pixels.len() >= 4 && w > 0 && h > 0);
    // Second call without new edit: drained, returns None.
    let frame2 = RasterEditTool::current_preview(&mut t);
    assert!(frame2.is_none(), "dirty drained — no new frame");
}

#[test]
fn raster_edit_current_preview_none_without_source() {
    let mut t = ColorEqualizationTool {
        params_dirty: true,
        ..ColorEqualizationTool::default()
    };
    assert!(RasterEditTool::current_preview(&mut t).is_none());
}

#[test]
fn raster_edit_take_pending_commit_drains() {
    let mut t = ColorEqualizationTool::default();
    t.apply_ui_edit(ColorEqualizationUiEdit::Apply);
    assert!(RasterEditTool::take_pending_commit(&mut t));
    assert!(!RasterEditTool::take_pending_commit(&mut t));
}

#[test]
fn raster_edit_run_full_returns_owned_buffer() {
    let mut t = ColorEqualizationTool::default();
    RasterEditTool::set_source(&mut t, solid(4, 4, [50, 100, 150]), 4, 4);
    let (out, w, h) = RasterEditTool::run_full(&mut t);
    assert!(w > 0 && h > 0);
    assert_eq!(out.len(), (w as usize) * (h as usize) * 4);
}

#[test]
fn raster_edit_deactivate_clears_transient_flags() {
    let mut t = ColorEqualizationTool::default();
    t.apply_ui_edit(ColorEqualizationUiEdit::Apply);
    // params_dirty + pending_apply both should be set after Apply.
    // (apply_ui_edit marks dirty for every edit + arms pending_apply).
    RasterEditTool::deactivate(&mut t);
    assert!(!t.take_pending_apply());
    assert!(!t.take_params_dirty());
}
