//! ⭐⭐⭐ **A secção GATILHO é PINTADA e está VIVA sob o dedo** (suplente #24).
//!
//! # ⚠️ O gesto é REAL, e não um `WidgetEvent::Click`
//!
//! Um `Click` sintético entra por cima da checagem de focabilidade do despachante, logo passa sobre
//! um controlo **pintado, hit-registado e ausente do `populate`** — a família que esta crate já
//! pagou **sete** vezes. Aqui o `Down`+`Up` cai no rectângulo que a pintura de facto registou.
//!
//! # ⛔ E a ISCA é o que torna o gate honesto
//!
//! A linha alvo é a **SEGUNDA** da lista e a aresta alvo é a **TERCEIRA** das três. Com uma só de
//! cada, um despacho que devolvesse sempre `0` — ou que lesse a tabela de ids errada — ficaria
//! **inobservável**.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::action_trigger_edits::{
    ActionTriggerFieldEdit as E, InspectorActionTriggerInfo, InspectorTriggerRow,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_action_trigger};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0x6A_71_10;
/// A linha que se edita — ⚠️ **não é `0` de propósito** (ver o cabeçalho).
const ALVO: usize = 1;
/// A aresta que se escolhe — ⚠️ **`Hold`, a última das três**, pela mesma razão.
const EDGE_ALVO: usize = 2;

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

fn linha(action: &str, existe: bool) -> InspectorTriggerRow {
    InspectorTriggerRow {
        action: action.into(),
        edge: 0,
        signal: "shoot".into(),
        accao_existe: existe,
    }
}

/// Duas linhas: a primeira ÓRFÃ (o nome não casa com acção nenhuma do mapa), a segunda sã.
///
/// ⚠️ **A órfã está lá de propósito** — ela é o sujeito do aviso que esta secção existe para dar, e
/// o silêncio dela é DUPLO: a lei do motor também a cala.
fn info() -> InspectorActionTriggerInfo {
    InspectorActionTriggerInfo {
        entity_bits: BITS,
        rows: vec![linha("fier", false), linha("fire", true)],
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: Option<InspectorActionTriggerInfo>) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_action_trigger(i);
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
            EditorAction::InspectorActionTriggerEdit { entity_bits, edit } => {
                assert_eq!(*entity_bits, BITS);
                Some(edit.clone())
            }
            _ => None,
        })
        .collect()
}

/// ⚠️ **Sem o componente não se pinta nada da secção** — ADR-0166.
#[test]
fn sem_o_componente_nada_da_seccao_e_pintado() {
    let (mut h, mut st) = host(None);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_TRIGGER_ADD).is_none());
    assert!(rect_de(&rects, ids::INSP_TRIGGER_EDGE_PICK).is_none());
}

/// ⭐⭐ **Clicar numa LINHA abre-a — e NÃO vai ao barramento.**
///
/// ⚠️ As duas metades: sem a primeira o editor mostra sempre a linha `0`; sem a segunda, escolher
/// uma linha vira um passo de `Ctrl+Z` sobre um facto que a cena nem tem onde guardar.
#[test]
fn clicar_numa_linha_abre_a_e_nao_vai_ao_barramento() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let acoes = clica(&mut h, &mut st, &rects, ids::INSP_TRIGGER_ROW[ALVO]);
    assert_eq!(st.trigger_selected, ALVO, "a linha nao abriu");
    assert!(
        edicoes(&acoes).is_empty(),
        "escolher uma linha nao e' uma edicao do documento"
    );
    set_current_inspector_action_trigger(None);
}

/// ⭐⭐⭐ **O CHIP da aresta abre, põe cada opção sob o ponteiro, e a escolhida chega ao barramento
/// com o índice CERTO.**
///
/// ⚠️ **Três juntas que ninguém vê da chamada:** `set_pending_trigger_dd` →
/// `take_pending_trigger_dd` → o passe diferido. Qualquer uma partida dá o mesmo sintoma —
/// *«abre e não dá para escolher»* — e **compila**.
#[test]
fn o_chip_da_aresta_abre_e_a_escolha_chega_com_o_indice_certo() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // Primeiro abre-se a linha alvo, que é a que o editor mostra.
    let _ = clica(&mut h, &mut st, &rects, ids::INSP_TRIGGER_ROW[ALVO]);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    let r = rect_de(&rects, ids::INSP_TRIGGER_EDGE_PICK).expect("o chip nao foi pintado");
    assert!(r.w > 0.0 && r.h > 0.0, "chip sem area: {r:?}");
    assert_eq!(
        h.dropdown_is_open(ids::INSP_TRIGGER_EDGE_PICK),
        Some(false),
        "o chip nao esta' registado como Dropdown — o despachante nao o sabe abrir"
    );

    h.set_dropdown_open(ids::INSP_TRIGGER_EDGE_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (i, &id) in ids::INSP_TRIGGER_EDGE_OPT.iter().enumerate() {
        let r = rect_de(&rects, id)
            .unwrap_or_else(|| panic!("a opcao {i} nao chegou ao indice de acerto"));
        assert!(r.w > 0.0 && r.h > 0.0, "opcao {i} sem area");
    }

    let acoes = clica(
        &mut h,
        &mut st,
        &rects,
        ids::INSP_TRIGGER_EDGE_OPT[EDGE_ALVO],
    );
    assert_eq!(
        edicoes(&acoes),
        vec![E::Edge(
            u8::try_from(ALVO).unwrap(),
            u8::try_from(EDGE_ALVO).unwrap()
        )],
        "a escolha tem de carregar a LINHA aberta e a ARESTA escolhida"
    );
    assert_eq!(
        h.dropdown_is_open(ids::INSP_TRIGGER_EDGE_PICK),
        Some(false),
        "escolher nao fechou o popover"
    );
    set_current_inspector_action_trigger(None);
}

/// ⭐⭐ **`+ Add Trigger` e `x Remove Trigger` estão vivos, e o `+` abre o que nasceu.**
#[test]
fn os_dois_botoes_estao_vivos_e_o_mais_abre_o_que_nasceu() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    let acoes = clica(&mut h, &mut st, &rects, ids::INSP_TRIGGER_ADD);
    assert_eq!(
        edicoes(&acoes),
        vec![E::Add],
        "o `+` nao chegou ao barramento"
    );
    assert_eq!(
        st.trigger_selected,
        info().rows.len(),
        "o `+` tem de abrir a linha que nasceu — senao ele parece nao ter feito nada"
    );

    st.trigger_selected = ALVO;
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let acoes = clica(&mut h, &mut st, &rects, ids::INSP_TRIGGER_REMOVE);
    assert_eq!(
        edicoes(&acoes),
        vec![E::Remove(u8::try_from(ALVO).unwrap())],
        "o `x` tem de apagar a linha ABERTA"
    );
    assert_eq!(
        st.trigger_selected,
        ALVO - 1,
        "depois de apagar, o indice aberto tem de recuar — senao o editor some"
    );
    set_current_inspector_action_trigger(None);
}
