//! ⭐⭐⭐ **A secção SCRIPT é PINTADA e está VIVA sob o dedo** (TOP-20 #16, W3).
//!
//! ⚠️ **O gesto é REAL** (`Down` + `Up` no rectângulo que a pintura registou), e é o que falta à cena
//! de smoke: a tela virtual onde ela foi fotografada não aceita eventos sintéticos, então o clique
//! só se prova aqui. *Um `WidgetEvent::Click` sintético passa sobre um botão ausente do `populate`*
//! (a família dos chips do impasto).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::script_edits::{
    InspectorScriptInfo, InspectorScriptOrphan, InspectorScriptProp, InspectorScriptStatus,
    InspectorScriptValue as V, ScriptFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_script};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1400.0,
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

fn prop(name: &str, value: V, own: bool) -> InspectorScriptProp {
    InspectorScriptProp {
        name: name.into(),
        value,
        own,
        min: Some(0.0),
        max: Some(10.0),
        step: Some(0.1),
    }
}

/// A secção do «Bob (tall)» da cena de smoke: um número próprio, um por omissão, uma caixa, um
/// texto e um órfão.
fn info() -> InspectorScriptInfo {
    InspectorScriptInfo {
        entity_bits: 0xB0B,
        source: "/x/bob.luau".into(),
        status: InspectorScriptStatus::Ready,
        props: vec![
            prop("amplitude", V::Number(2.0), true),
            prop("speed", V::Number(2.0), false),
            prop("active", V::Bool(true), false),
            prop("top_signal", V::Text(String::new()), false),
        ],
        orphans: vec![InspectorScriptOrphan {
            name: "height".into(),
            value: V::Number(9.0),
            wants: None,
        }],
        kept: 0,
        failure: None,
        clock_playing: true,
        also_physics: false,
        selected_count: 1,
    }
}

fn host(i: Option<InspectorScriptInfo>) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_script(i);
    (h, InspectorState::default())
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Um clique real no centro de `id`; devolve as acções que chegaram ao barramento.
fn clica(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
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
}

fn edicoes(acoes: &[EditorAction]) -> Vec<E> {
    acoes
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Script(edit),
            } => {
                assert_eq!(*entity_bits, 0xB0B);
                Some(edit.clone())
            }
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ **Cada linha pinta o controlo do TIPO dela, e só esse.**
///
/// **Mutação que deve sangrar:** pintar o campo numérico para toda linha.
#[test]
fn cada_linha_pinta_o_controlo_do_tipo_dela() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for id in [ids::INSP_SCRIPT_SOURCE, ids::INSP_SCRIPT_BROWSE] {
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        assert!(r.w > 0.0 && r.h > 0.0);
    }
    let pintado = |id| rect_de(&rects, id).is_some();
    assert!(pintado(ids::INSP_SCRIPT_NUM[0]) && pintado(ids::INSP_SCRIPT_NUM[1]));
    assert!(pintado(ids::INSP_SCRIPT_BOOL[2]));
    assert!(pintado(ids::INSP_SCRIPT_TEXT[3]));
    for (i, id) in [
        ids::INSP_SCRIPT_BOOL[0],
        ids::INSP_SCRIPT_TEXT[0],
        ids::INSP_SCRIPT_NUM[2],
        ids::INSP_SCRIPT_NUM[3],
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            !pintado(id),
            "caso {i}: um controlo de OUTRO tipo foi pintado"
        );
    }
    // ⭐ O `Reset` só aparece onde o valor é PRÓPRIO.
    assert!(pintado(ids::INSP_SCRIPT_RESET[0]));
    assert!(!pintado(ids::INSP_SCRIPT_RESET[1]));
    assert!(pintado(ids::INSP_SCRIPT_ORPHAN_REMOVE[0]));
    assert!(!pintado(ids::INSP_SCRIPT_ORPHAN_REMOVE[1]));
    set_current_inspector_script(None);
}

#[test]
fn sem_o_componente_nada_da_seccao_e_pintado() {
    let (mut h, mut st) = host(None);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_SCRIPT_BROWSE).is_none());
    assert!(rect_de(&rects, ids::INSP_SCRIPT_NUM[0]).is_none());
}

/// ⭐⭐ **`Reset` larga o valor da SUA linha, pelo NOME** — e `Remove` larga o órfão.
///
/// **Mutação que deve sangrar:** mandar o nome da linha seguinte, ou esquecer o `populate` dos botões.
#[test]
fn reset_e_remove_chegam_ao_barramento_com_o_nome_certo() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_SCRIPT_RESET[0]);
    assert_eq!(edicoes(&a), [E::Forget("amplitude".into())]);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_SCRIPT_ORPHAN_REMOVE[0]);
    assert_eq!(edicoes(&a), [E::Forget("height".into())]);
    set_current_inspector_script(None);
}

#[test]
fn browse_pede_o_dialogo_a_shell() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_SCRIPT_BROWSE);
    assert_eq!(edicoes(&a), [E::Browse]);
    set_current_inspector_script(None);
}

/// ⭐ **A caixa manda o CONTRÁRIO do que o snapshot diz** — e PÕE o valor (D1).
///
/// **Mutação que deve sangrar:** tirar os `INSP_SCRIPT_BOOL` do `populate_script`.
#[test]
fn a_caixa_manda_o_contrario_do_snapshot() {
    let (mut h, mut st) = host(Some(info()));
    // ⚠️ A pintura começa pela SEMENTE (`sync_inspector_from_snapshots`): é ela que põe na caixa o
    // valor do snapshot, e o clique inverte esse — nunca o de partida do `populate`.
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_SCRIPT_BOOL[2]);
    assert_eq!(edicoes(&a), [E::SetBool("active".into(), false)]);
    set_current_inspector_script(None);
}
