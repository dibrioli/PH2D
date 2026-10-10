//! **A secção MOTION INPUT é PINTADA e está VIVA sob o dedo** (ciclo 19 do Motion).
//!
//! O gesto é REAL (`Down` + `Up` no rectângulo que a pintura registou): um `WidgetEvent::Click`
//! sintético passaria sobre um controlo ausente do `populate`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MotionInputFieldEdit as E, SOURCE_COLOR, SOURCE_HEALTH, SOURCE_VALUE,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_motion_input};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0x000A_11CE;

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

fn info(source: u8) -> InspectorMotionInputInfo {
    InspectorMotionInputInfo {
        entity_bits: BITS,
        name: "input".into(),
        source,
        value: 0.5,
        color: [1.0; 4],
        nomes_do_grafo: vec!["energy".into(), "speed".into()],
        tem_vida: true,
        selected_count: 1,
    }
}

fn pinta(
    i: InspectorMotionInputInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_input(Some(i));
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    (h, st, rects)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Um clique real no centro de `id`; devolve as edições da secção que chegaram ao barramento.
fn clica(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<E> {
    let r = rect_de(rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre {id:?} não virou evento — pintado e ausente do `populate`"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::MotionInput(e),
            } => {
                assert_eq!(*entity_bits, BITS);
                Some(e.clone())
            }
            _ => None,
        })
        .collect()
}

/// As linhas `Value` e `Color` só existem com a fonte que as lê; nome, chips e fonte sempre.
///
/// **Mutação que deve sangrar:** pintar a linha do número sem o `if i.le_valor()`.
#[test]
fn so_a_fonte_que_le_um_campo_o_mostra() {
    for (fonte, valor, cor) in [
        (SOURCE_VALUE, true, false),
        (SOURCE_COLOR, false, true),
        (SOURCE_HEALTH, false, false),
    ] {
        let (_h, _st, rects) = pinta(info(fonte));
        let pintado = |id| rect_de(&rects, id).is_some();
        assert!(
            pintado(ids::INSP_MI_NAME),
            "o nome pinta-se (fonte {fonte})"
        );
        for id in ids::INSP_MI_SOURCE {
            assert!(pintado(id), "a fonte pinta-se (fonte {fonte})");
        }
        for id in &ids::INSP_MI_GRAFO[..2] {
            assert!(pintado(*id), "um chip por nome do grafo (fonte {fonte})");
        }
        assert!(
            !pintado(ids::INSP_MI_GRAFO[2]),
            "só há dois nomes no grafo — o terceiro chip não existe"
        );
        assert_eq!(pintado(ids::INSP_MI_VALUE), valor, "número (fonte {fonte})");
        assert_eq!(pintado(ids::INSP_MI_COLOR), cor, "cor (fonte {fonte})");
    }
}

/// O clique numa FONTE envia o índice dela; o clique num CHIP do grafo escreve aquele nome.
#[test]
fn a_fonte_e_o_chip_do_grafo_editam_o_componente() {
    let (mut h, mut st, rects) = pinta(info(SOURCE_VALUE));
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_SOURCE[3]),
        vec![E::Source(3)]
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_GRAFO[1]),
        vec![E::Name("speed".into())],
        "o chip escreve o nome que o grafo lê"
    );
}

/// Sem instantâneo, nada da secção se pinta.
#[test]
fn sem_componente_nao_ha_seccao() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_input(None);
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_MI_NAME).is_none());
}
