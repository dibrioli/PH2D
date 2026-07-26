//! Testes da `FlipTool` — módulo-irmão pelo cap de LOC (HR-18).
//!
//! Declarado pelo pai via `#[path]`, então `super` é o módulo pai.

use super::*;

#[test]
fn fresh_tool_defaults() {
    let t = FlipTool::new();
    assert_eq!(t.stroke_rgba(), DEFAULT_STROKE);
    assert_eq!(t.width_px(), DEFAULT_WIDTH_PX);
    assert_eq!(t.mode(), FlipMode::Select); // gizmo por default (ADR-0112)
    assert_eq!(t.hardness(), 1.0);
}

#[test]
fn default_snapshot_matches_fresh_tool() {
    // The panel paints `FlipStyleSnapshot::default()` before the shell's 1st
    // push; it must equal the fresh tool's snapshot so nothing "jumps".
    assert_eq!(FlipStyleSnapshot::default(), FlipTool::new().ui_snapshot());
}

#[test]
fn panel_events_drive_mode_and_brush() {
    let mut t = FlipTool::new();
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_MODE_DRAW));
    assert_eq!(t.mode(), FlipMode::Draw);
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_MODE_ERASE));
    assert_eq!(t.mode(), FlipMode::Erase);
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_ERASE_HARD));
    assert_eq!(t.erase_mode(), EraseMode::Hard);
    // Size slider at full track → WIDTH_MAX_PX.
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 1.0));
    assert_eq!(t.width_px(), crate::params::WIDTH_MAX_PX);
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_HARDNESS, 0.25));
    assert!((t.hardness() - 0.25).abs() < 1e-6);
    // A layer-op id (document edit) is ignored by the tool.
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LAYER_ADD));
    assert_eq!(t.mode(), FlipMode::Erase, "layer op didn't touch the tool");
}

/// 🔴 **O toggle Self Overlap chega à tool e ALTERNA** (03 §8): o `Click` do
/// `FLIP_SELF_OVERLAP` inverte o flag, que o snapshot leva ao painel e o `flip_draw`
/// leva ao `FlipStroke`. Nasce OFF (byte-idêntico ao traço de sempre). Mutação que
/// sangra: o arm de Click não alcançar o campo (o chip morre sob o mouse — a lição do
/// slider de Spacing que ficou de fora do arm).
#[test]
fn the_self_overlap_toggle_reaches_the_tool_and_toggles() {
    let mut t = FlipTool::new();
    assert!(!t.self_overlap(), "nasce OFF (o traco de sempre)");
    assert!(!t.ui_snapshot().self_overlap, "e o snapshot concorda");
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_SELF_OVERLAP));
    assert!(t.self_overlap(), "um clique liga");
    assert!(t.ui_snapshot().self_overlap);
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_SELF_OVERLAP));
    assert!(!t.self_overlap(), "o segundo clique desliga (e um toggle)");
}

/// 🔴 **Os sliders de pressão chegam à tool** (T2.6): o `SetValue` de `FLIP_PRESSURE_MIN` /
/// `FLIP_PRESSURE_RESPONSE` grava a fração no campo, que o snapshot leva ao `flip_draw`
/// (`pressure_width_factor`). Mutação que sangra: o arm de SetValue não alcançar o campo (o slider
/// mexe na tela e nunca chega à tool — a lição do slider de Spacing que ficou de fora do arm).
#[test]
fn the_pressure_sliders_reach_the_tool() {
    let mut t = FlipTool::new();
    // Defaults: piso 0.05, resposta 0.5 (linear).
    assert!((t.pressure_min_width() - 0.05).abs() < 1e-6, "min default");
    assert!(
        (t.pressure_response() - 0.5).abs() < 1e-6,
        "response default"
    );
    assert!(
        (t.ui_snapshot().pressure_min_width - 0.05).abs() < 1e-6,
        "snapshot concorda"
    );

    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_PRESSURE_MIN, 0.4));
    assert!(
        (t.pressure_min_width() - 0.4).abs() < 1e-6,
        "min chegou: {}",
        t.pressure_min_width()
    );
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_PRESSURE_RESPONSE, 0.8));
    assert!(
        (t.pressure_response() - 0.8).abs() < 1e-6,
        "response chegou"
    );
    assert!(
        (t.ui_snapshot().pressure_response - 0.8).abs() < 1e-6,
        "snapshot atualizado"
    );
}

/// 🔴 **O toggle Airbrush chega à tool e ALTERNA** (03 §8): o `Click` do `FLIP_AIRBRUSH` inverte
/// o flag, que o snapshot leva ao painel e o `flip_draw` leva ao `FlipStroke`. Nasce OFF
/// (byte-idêntico). Mutação que sangra: o arm de Click não alcançar o campo.
#[test]
fn the_airbrush_toggle_reaches_the_tool_and_toggles() {
    let mut t = FlipTool::new();
    assert!(!t.airbrush(), "nasce OFF (o pincel de sempre)");
    assert!(!t.ui_snapshot().airbrush, "e o snapshot concorda");
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_AIRBRUSH));
    assert!(t.airbrush(), "um clique liga");
    assert!(t.ui_snapshot().airbrush);
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_AIRBRUSH));
    assert!(!t.airbrush(), "o segundo clique desliga (e um toggle)");
}

/// **O slider Bleed do Colorize chega à tool** (6º smoke): o `SetValue` do
/// `FLIP_COLORIZE_BLEED` grava a fração `colorize_bleed` (`0..1`), que o shell mapeia
/// para o pedágio de aperto do motor. Trap e Bleed são knobs INDEPENDENTES.
#[test]
fn the_colorize_bleed_slider_reaches_the_tool() {
    let mut t = FlipTool::new();
    assert!(
        (t.ui_snapshot().colorize_bleed - 0.5).abs() < 1e-9,
        "o Bleed nasce no meio (o pedágio DEFAULT do 5º smoke)"
    );
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_COLORIZE_BLEED, 0.8));
    assert!((t.ui_snapshot().colorize_bleed - 0.8).abs() < 1e-9);
    // Fora de [0,1] é clampado (o slider nunca sai, mas a porta se defende).
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_COLORIZE_BLEED, 5.0));
    assert!((t.ui_snapshot().colorize_bleed - 1.0).abs() < 1e-9);
    // O Trap é independente: mexer no Bleed não o move.
    assert_eq!(t.ui_snapshot().trap, 0.0, "o Bleed nao pode tocar o Trap");
}

/// **O Trap máximo (6º smoke: 20 → 50) chega à tool** — o range que faltava para selar
/// o vão que o Enio de fato desenha (~100 px de tela ⇒ slider ~50).
#[test]
fn the_trap_slider_reaches_fifty_at_full_track() {
    let mut t = FlipTool::new();
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_TRAP, 1.0));
    assert!(
        (t.ui_snapshot().trap - crate::params::TRAP_MAX_PX).abs() < 1e-9,
        "o Trap no fim do slider tem de valer TRAP_MAX_PX"
    );
    assert!(
        t.ui_snapshot().trap >= 50.0,
        "o range aumentado (50) e o que sela o vao grande do 6o smoke"
    );
}

// ── §4.C — os links da borracha (Unified Paint Settings do Blender) ───────────

/// 🔴 **Linkado (o DEFAULT) a borracha É o pincel** — um número só, como sempre foi.
/// Mexer no Size do pincel move o raio da borracha junto; esse é o comportamento
/// que o projeto tinha antes do §4.C e que o toggle preserva por default.
/// O *tip* pontilhado (03 §8): os 3 botões e o slider de spacing chegam ao tool, e o
/// snapshot (o que o traço desenhado herda) carrega o valor. Default = linha cheia.
#[test]
fn the_tip_toggle_and_spacing_slider_reach_the_tool() {
    let mut t = FlipTool::new();
    assert_eq!(t.tip(), StrokeTip::Continuous, "default = linha cheia");
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_TIP_DOTS));
    assert_eq!(t.tip(), StrokeTip::Dots);
    assert_eq!(
        t.ui_snapshot().tip,
        StrokeTip::Dots,
        "o snapshot leva o tip"
    );
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_TIP_SQUARES));
    assert_eq!(t.tip(), StrokeTip::Squares);
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_TIP_LINE));
    assert_eq!(t.tip(), StrokeTip::Continuous);
    // O slider: track `0..1` → múltiplo do diâmetro `0..DOT_SPACING_MAX`, e chega ao snapshot.
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_DOT_SPACING, 0.5));
    let want = 0.5 * crate::params::DOT_SPACING_MAX;
    assert!(
        (t.ui_snapshot().dot_spacing - want).abs() < 1e-6,
        "spacing {} != {want}",
        t.ui_snapshot().dot_spacing
    );
}

#[test]
fn linked_by_default_the_eraser_follows_the_brush() {
    let mut t = FlipTool::new();
    assert!(t.link_size() && t.link_strength(), "o default e LINKADO");
    assert_eq!(t.eraser_size_px(), t.width_px());
    assert_eq!(t.eraser_strength(), t.opacity());

    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 1.0));
    assert_eq!(
        t.eraser_size_px(),
        crate::params::WIDTH_MAX_PX,
        "linkada, a borracha segue o Size do pincel"
    );
    t.set_opacity(0.25);
    assert!((t.eraser_strength() - 0.25).abs() < 1e-6);
}

/// 🔴 **Deslinkada, cada uma tem a sua** — e mexer numa NÃO move a outra.
///
/// Mutação que sangra: `eraser_size_px` devolver `width_px` incondicionalmente (ou
/// o toggle não inverter o flag) — o raio da borracha passa a seguir o pincel e os
/// dois `assert` de independência caem.
#[test]
fn unlinked_the_eraser_and_the_brush_keep_their_own_numbers() {
    let mut t = FlipTool::new();
    // O toggle na linha do Size desliga o link.
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_STRENGTH));
    assert!(!t.link_size() && !t.link_strength());

    // Deslinkar NÃO move número nenhum: os próprios nascem nos defaults do pincel,
    // então o 1º deslink não pula na cara do artista.
    assert_eq!(t.eraser_size_px(), DEFAULT_WIDTH_PX);
    assert_eq!(t.eraser_strength(), DEFAULT_OPACITY);

    // O slider PRÓPRIO da borracha move só ela.
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_STRENGTH, 0.5));
    assert_eq!(t.eraser_size_px(), crate::params::WIDTH_MAX_PX);
    assert!((t.eraser_strength() - 0.5).abs() < 1e-6);
    assert_eq!(t.width_px(), DEFAULT_WIDTH_PX, "o PINCEL nao se mexeu");
    assert_eq!(
        t.opacity(),
        DEFAULT_OPACITY,
        "a forca do pincel nao se mexeu"
    );

    // E o inverso: mexer no pincel não toca a borracha deslinkada.
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 0.0));
    assert_eq!(t.width_px(), crate::params::WIDTH_MIN_PX);
    assert_eq!(
        t.eraser_size_px(),
        crate::params::WIDTH_MAX_PX,
        "a borracha deslinkada ignora o Size do pincel"
    );
}

/// **Re-linkar devolve a borracha ao pincel** — e o valor próprio dela SOBREVIVE,
/// então deslinkar de novo o recupera (o modelo do Blender: cada pincel guarda o
/// seu; o link só escolhe QUEM responde).
#[test]
fn relinking_returns_to_the_brush_and_the_own_value_survives() {
    let mut t = FlipTool::new();
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
    assert_eq!(t.eraser_size_px(), crate::params::WIDTH_MAX_PX);

    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE)); // re-linka
    assert_eq!(t.eraser_size_px(), t.width_px(), "voltou a seguir o pincel");

    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE)); // deslinka de novo
    assert_eq!(
        t.eraser_size_px(),
        crate::params::WIDTH_MAX_PX,
        "o valor PROPRIO da borracha sobreviveu ao re-link"
    );
}

/// O snapshot publica os valores **EFETIVOS** (link já resolvido) — é o que o anel
/// do cursor e o `flip_erase` leem, e por isso eles nunca re-derivam a regra.
#[test]
fn the_snapshot_publishes_effective_eraser_values() {
    let mut t = FlipTool::new();
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
    t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
    let s = t.ui_snapshot();
    assert_eq!(
        s.erase_px,
        crate::params::WIDTH_MAX_PX,
        "efetivo = o proprio"
    );
    assert!(!s.link_size, "o snapshot carrega o estado do toggle");
    assert_eq!(
        s.width_px, DEFAULT_WIDTH_PX,
        "o Size do pincel vai separado"
    );
    // Linkado, o efetivo volta a ser o do pincel.
    t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
    assert_eq!(t.ui_snapshot().erase_px, DEFAULT_WIDTH_PX);
}

#[test]
fn set_mode_and_stroke() {
    let mut t = FlipTool::new();
    t.set_mode(FlipMode::Draw);
    assert_eq!(t.mode(), FlipMode::Draw);
    t.set_stroke_rgba([220, 60, 60, 255]);
    assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
}

#[test]
fn ui_snapshot_round_trips_style() {
    let mut t = FlipTool::new();
    t.set_stroke_rgba([1, 2, 3, 255]);
    t.set_mode(FlipMode::Erase);
    let s = t.ui_snapshot();
    assert_eq!(s.stroke, [1, 2, 3, 255]);
    assert_eq!(s.mode, FlipMode::Erase);
    assert_eq!(s.width_px, t.width_px());
}

#[test]
fn id_label_icon_stable() {
    let t = FlipTool::new();
    assert_eq!(t.id(), ToolId::new("flip"));
    assert_eq!(t.label(), "Flip");
    assert_eq!(t.icon_slug(), "flip");
}
