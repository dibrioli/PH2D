//! **Seam de SOLDAR** (plano 39) — o botão existe, é clicável de verdade, e o clique chega ao
//! barramento.
//!
//! ⚠️ **O `Click` sintético não mede a costura que morde.** Um botão pintado, registado no índice
//! de hit e **sem `register` no `populate`** aparece, acende sob o rato e o clique **nunca vira
//! evento** — é o defeito que o `populate_modes` e o `populate_ops` já pagaram cinco vezes entre os
//! dois. O oráculo é o ponteiro REAL sobre o rectângulo que o `paint` devolveu.

use ph2d_editor_core::interaction::WidgetEvent;
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

/// **O botão responde a um ponteiro de verdade.**
#[test]
fn the_weld_button_answers_a_real_pointer() {
    state::set_current_selection_count(1);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PATH_WELD)
        .expect("o botao Weld tem de ser PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_PATH_WELD)),
        "o ponteiro sobre Weld nao virou Click — falta o `register` no `populate_ops`"
    );
}

/// ⚠️ **Sem selecção nenhuma ele não se oferece** — soldar precisa de sujeito, e um botão que só
/// sabe recusar é a lei do controlo morto.
///
/// ⛔ **Mas com UM caminho ele aparece**, ao contrário do *Join* (que exige dois): um caminho
/// sozinho pode ter **auto-cruzamento**, e ali soldar tem o que fazer.
#[test]
fn weld_needs_a_subject_but_one_path_is_already_a_subject() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;

    state::set_current_selection_count(0);
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PATH_WELD)
            .is_none(),
        "sem seleccao o Weld nao tem sujeito"
    );

    state::set_current_selection_count(1);
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PATH_WELD)
            .is_some(),
        "com UM caminho ele ja' se oferece (auto-cruzamento)"
    );
    // …e o Join, o vizinho, continua a exigir dois — é o que separa os dois verbos.
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PATH_JOIN)
            .is_none(),
        "o Join com um caminho so' seria um botao morto"
    );
    state::set_current_selection_count(0);
}
