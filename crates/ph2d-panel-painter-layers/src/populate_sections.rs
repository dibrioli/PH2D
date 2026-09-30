//! The Brush panel's **collapsible section headers** — which ones collapse and which start
//! collapsed. (⚠️ Until 2026-09-29 each also had a colour dot; the owner retired it.)
//!
//! A sibling of `populate.rs` (which hit the 600-LOC panel cap), and cohesive on its own: this is the
//! one table that decides whether a section's chevron is alive. It is a *silent* failure when an id
//! is missing — the header still paints its chevron, it simply does nothing under the mouse
//! (`dispatch_pointer` needs the id marked, not merely drawn). Impasto shipped that way. If you add a section, add it HERE too, and gate it by CLICKING it.

use ph2d_editor_core::interaction::WidgetStore;

/// Mark each collapsible Brush section's header (click-to-collapse). Randomize Color, Color Ramp
/// and Tiling START COLLAPSED; Texture + Stroke start expanded (Enio 2026-06-24).
pub(crate) fn register_collapsible_sections(store: &mut WidgetStore) {
    // ⚠️ Uma chamada por secção, e não um laço sobre uma lista: o gate
    //    `the_painted_control_reaches_a_consumer` lê o id na chamada que o consome, e através de uma
    //    variável de laço ele não o vê (2026-09-29, quando os pares `(secção, ponto de cor)` saíram).
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_SHAPE_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_RAMP_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_STROKE_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_WETPAINT_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION);
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_SECTION);
    // Impasto — MISSING since the section landed: its header painted a chevron that could not
    // collapse, because the id was not here. Same silence as the lamp chips, one row higher up
    // (`tests/seam_impasto_rig.rs`).
    store.mark_collapsible_section(ph2d_tool_painter::ids::PAINTER_IMPASTO_SECTION);
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
