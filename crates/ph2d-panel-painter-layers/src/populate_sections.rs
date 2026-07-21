//! The Brush panel's **collapsible section headers** — which ones collapse, which start collapsed,
//! and which colour dot opens the shared picker.
//!
//! A sibling of `populate.rs` (which hit the 600-LOC panel cap), and cohesive on its own: this is the
//! one table that decides whether a section's chevron and colour dot are alive. Both are *silent*
//! failures when an id is missing — the header still paints its chevron and the dot still paints its
//! colour, they simply do nothing under the mouse (`dispatch_pointer` needs the id marked, not merely
//! drawn). Impasto shipped that way. If you add a section, add it HERE too, and gate it by CLICKING it.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Mark each collapsible Brush section's header (click-to-collapse) + make its colour dot a picker
/// swatch (clicking opens the shared picker to assign the dot's colour). Randomize Color, Color Ramp
/// and Tiling START COLLAPSED; Texture + Stroke start expanded (Enio 2026-06-24).
pub(crate) fn register_collapsible_sections(store: &mut WidgetStore) {
    for (section, color) in [
        (
            core_ids::PAINTER_SHAPE_SECTION,
            core_ids::PAINTER_SHAPE_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
            core_ids::PAINTER_BRUSH_RANDOMIZE_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_TEXTURE_SECTION,
            core_ids::PAINTER_BRUSH_TEXTURE_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
            core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_STROKE_SECTION,
            core_ids::PAINTER_BRUSH_STROKE_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_TILING_SECTION,
            core_ids::PAINTER_BRUSH_TILING_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_BRUSH_SYMMETRY_SECTION,
            core_ids::PAINTER_BRUSH_SYMMETRY_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_SHAPE_RAMP_SECTION,
            core_ids::PAINTER_SHAPE_RAMP_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_WETPAINT_SECTION,
            core_ids::PAINTER_WETPAINT_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_WATERCOLOR_SECTION,
            core_ids::PAINTER_WATERCOLOR_SECTION_COLOR,
        ),
        (
            core_ids::PAINTER_WATERCOLOR_PAPER_SECTION,
            core_ids::PAINTER_WATERCOLOR_PAPER_SECTION_COLOR,
        ),
        // Impasto — MISSING since the section landed: its header painted a chevron that could not
        // collapse and a colour dot that opened nothing, because neither id was here. Same silence
        // as the lamp chips, one row higher up (`tests/seam_impasto_rig.rs`).
        (
            core_ids::PAINTER_IMPASTO_SECTION,
            core_ids::PAINTER_IMPASTO_SECTION_COLOR,
        ),
    ] {
        store.mark_collapsible_section(section);
        store.register_picker_swatch(color);
    }
    for collapsed in [
        core_ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
        core_ids::PAINTER_BRUSH_TILING_SECTION,
        core_ids::PAINTER_BRUSH_SYMMETRY_SECTION,
        core_ids::PAINTER_SHAPE_RAMP_SECTION,
    ] {
        store.set_collapsed(collapsed, true);
    }
}
