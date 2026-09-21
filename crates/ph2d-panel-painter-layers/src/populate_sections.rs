//! The Brush panel's **collapsible section headers** — which ones collapse, which start collapsed,
//! and which colour dot opens the shared picker.
//!
//! A sibling of `populate.rs` (which hit the 600-LOC panel cap), and cohesive on its own: this is the
//! one table that decides whether a section's chevron and colour dot are alive. Both are *silent*
//! failures when an id is missing — the header still paints its chevron and the dot still paints its
//! colour, they simply do nothing under the mouse (`dispatch_pointer` needs the id marked, not merely
//! drawn). Impasto shipped that way. If you add a section, add it HERE too, and gate it by CLICKING it.

use ph2d_editor_core::interaction::WidgetStore;

/// Mark each collapsible Brush section's header (click-to-collapse) + make its colour dot a picker
/// swatch (clicking opens the shared picker to assign the dot's colour). Randomize Color, Color Ramp
/// and Tiling START COLLAPSED; Texture + Stroke start expanded (Enio 2026-06-24).
pub(crate) fn register_collapsible_sections(store: &mut WidgetStore) {
    for (section, color) in [
        (
            ph2d_tool_painter::ids::PAINTER_SHAPE_SECTION,
            ph2d_tool_painter::ids::PAINTER_SHAPE_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_RAMP_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_STROKE_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_STROKE_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SECTION,
            ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION,
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_SECTION,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION,
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION_COLOR,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_SECTION,
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_SECTION_COLOR,
        ),
        // Impasto — MISSING since the section landed: its header painted a chevron that could not
        // collapse and a colour dot that opened nothing, because neither id was here. Same silence
        // as the lamp chips, one row higher up (`tests/seam_impasto_rig.rs`).
        (
            ph2d_tool_painter::ids::PAINTER_IMPASTO_SECTION,
            ph2d_tool_painter::ids::PAINTER_IMPASTO_SECTION_COLOR,
        ),
    ] {
        store.mark_collapsible_section(section);
        store.register_picker_swatch(color);
    }
    // ⭐ **A decisão do DONO, de 2026-06-24** — `Randomize Color`, `Color Ramp` e `Tiling` nascem
    //    recolhidas; `Texture` e `Stroke` nascem abertas. ⛔ Ela não se toca.
    for collapsed in [
        ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
        ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_SECTION,
    ] {
        store.set_collapsed_if_unchosen(collapsed, true);
    }

    // ⭐⭐⭐ **AS QUE CHEGARAM DEPOIS DAQUELA DECISÃO NASCEM RECOLHIDAS.**
    //
    // ⛔⛔ **A decisão do dono nomeia CINCO secções e o painel tem TREZE**, e a diferença tem
    //    DATA: medido por `git log -S`, o `Texture` e o `Stroke` nasceram **no dia** dela
    //    (`2026-06-24`) e as outras **depois** — `Shape` e `Shape Ramp` a `06-25`, o `Symmetry` a
    //    `06-29`, o `Watercolor Paper` a `07-05`. ⇒ *o maior bloco do painel não existia quando
    //    ele decidiu*, e as que aqui entram são as que nunca tiveram decisão nenhuma.
    //
    // ⚠️⚠️ **O `Shape` sozinho vale `807 px`** — `0,92` de uma dobra de `880`. Com estas quatro
    //    recolhidas o painel abre com `1 479 px` em vez de `2 709` (`3,1 → 1,7` ecrãs).
    //
    // ⛔ **Ele NÃO passa a caber, e o que falta é DECISÃO e não código:** o `Texture` (`450 px`) e
    //    o `Stroke` (`368`) que o dono mandou nascer abertos, mais os `~700 px` de cromo fora de
    //    secção nenhuma, somam mais do que a dobra. *Com tudo recolhido ele mediria `736 px`.*
    //
    // ⚠️ `_if_unchosen`: a escolha do artista manda, e sobrevive à sessão.
    for collapsed in [
        ph2d_tool_painter::ids::PAINTER_SHAPE_SECTION,
        ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION,
        ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SECTION,
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_SECTION,
    ] {
        store.set_collapsed_if_unchosen(collapsed, true);
    }
}
