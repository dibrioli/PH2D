//! ⭐⭐⭐ Seam do botão **Weld** (plano 39) — ele está vivo sob o MOUSE, e o clique chega ao bus.
//!
//! # Porque este arquivo nasceu depois do motor
//!
//! Report do Enio (2026-09-01): *"não funcionou ainda o Weld"* — sobre um motor cujos gates de lei
//! estavam todos verdes. ⚠️ **Nenhum deles atravessava o painel**: eles chamam `apply_vec_weld`
//! directamente, e um verbo cujo BOTÃO não fala com ninguém lê-se exactamente como um motor
//! partido. É a lição que a fileira de chips da booleana já custou uma wave (`seam_bool.rs`).
//!
//! O gesto é REAL (Down+Up sobre o rectângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a forma em que um controlo nasce *pintado, hit-registrado e morto sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// **Com UMA forma seleccionada o botão existe, responde ao ponteiro e o Click chega ao bus.**
///
/// ⚠️ **Uma só, e não duas**: ao contrário do *Join*, soldar não tem piso de dois — um caminho
/// sozinho pode ter AUTO-cruzamento, e ali o verbo tem o que fazer. Se este gate passar a exigir
/// duas, foi o produto que mudou.
#[test]
fn the_weld_button_is_alive_under_the_pointer_and_reaches_the_bus() {
    state::set_current_selection_count(1);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_PATH_WELD)
        .expect("o botao Weld nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_PATH_WELD)),
        "o ponteiro sobre o Weld nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate_ops)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_PATH_WELD
        )),
        "o Click do Weld nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a linha \
         na allowlist do event_clicks)"
    );
}

/// ⛔ **Com a selecção VAZIA o botão não existe** — a metade da ausência.
///
/// A seção Path é um COMANDO sobre a selecção (`section_scope::WHEN_SELECTED`): sem alvo, ela seria
/// um cabeçalho com botões que só sabem recusar.
#[test]
fn with_nothing_selected_there_is_no_weld_button() {
    state::set_current_selection_count(0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_PATH_WELD);
    state::set_current_selection_count(1);
    assert!(
        r.is_none(),
        "o Weld subiu com a selecao vazia — um botao que so' sabe recusar"
    );
}
