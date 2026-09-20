//! ⭐⭐⭐ **A caixa «Keep on Restart» do contador é PINTADA e está VIVA sob o dedo** (2026-09-20).
//!
//! # ⚠️ O gesto é REAL, e não um `WidgetEvent::Toggled` sintético
//!
//! Um evento sintético entra por cima da checagem de focabilidade do despachante, logo passa sobre
//! um controlo **pintado, hit-registado e ausente do `populate`** — a família que esta crate já
//! pagou **sete** vezes. Aqui o `Down`+`Up` cai no rectângulo que a pintura de facto registou.
//!
//! # ⛔ E a ISCA é o que torna o gate honesto
//!
//! A secção HUD tem **DUAS** caixas de componentes diferentes (`Disabled` do `UiButton`,
//! `Keep on Restart` do `Counter`), e o braço do `Toggled` era um `if` de um ramo só. Com uma caixa
//! de cada vez um despacho que devolvesse sempre a mesma edição ficaria **inobservável** ⇒ o
//! instantâneo traz as duas, e cada metade afirma que a OUTRA não se mexeu.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::hud_edits::{HudFieldEdit as E, InspectorHudInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_hud};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0x_C0_17_3A;

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

/// Um objecto com **botão E contador** — a isca do cabeçalho.
///
/// ⚠️ As duas caixas nascem **DESLIGADAS**, logo um clique em qualquer uma delas tem de chegar ao
/// barramento com `true`: *um lado a começar ligado deixaria a régua incapaz de distinguir «a
/// caixa certa mudou» de «a leitura veio do sítio errado»*.
fn info() -> InspectorHudInfo {
    InspectorHudInfo {
        entity_bits: BITS,
        has_button: true,
        signal: "bateu".into(),
        disabled: false,
        has_counter: true,
        counter_name: "pontos".into(),
        counter_start: 0.0,
        counter_keep: false,
        counter_value: 7,
        ..InspectorHudInfo::default()
    }
}

fn host(i: Option<InspectorHudInfo>) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_hud(i);
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
                edit: ComponentEdit::Hud(edit),
            } => {
                assert_eq!(*entity_bits, BITS);
                Some(edit.clone())
            }
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ **Carregar na caixa do contador chega ao barramento como `CounterKeep(true)`.**
///
/// **Mutação que deve sangrar:** apagar o `store.register` dela do `populate_hud` (o ponteiro
/// deixa de virar evento) · ou o braço do `event_hud` devolver `E::Disabled`.
#[test]
fn a_caixa_do_contador_chega_ao_barramento() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let acoes = clica(&mut h, &mut st, &rects, ids::INSP_HUD_COUNTER_KEEP);
    assert_eq!(
        edicoes(&acoes),
        vec![E::CounterKeep(true)],
        "a caixa do CONTADOR tem de chegar como CounterKeep"
    );
    set_current_inspector_hud(None);
}

/// ⚠️ **CONTROLO: a caixa do BOTÃO continua a chegar como `Disabled`.**
///
/// Sem esta metade, um braço que mandasse **tudo** como `CounterKeep` ficaria verde acima.
#[test]
fn a_caixa_do_botao_continua_a_chegar_como_disabled() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let acoes = clica(&mut h, &mut st, &rects, ids::INSP_HUD_DISABLED);
    assert_eq!(
        edicoes(&acoes),
        vec![E::Disabled(true)],
        "a caixa do BOTÃO nao pode ter sido roubada pelo braço novo"
    );
    set_current_inspector_hud(None);
}

/// ⚠️ **Sem `Counter` a caixa NÃO é pintada** — ADR-0166: o Inspector mostra o que o objecto TEM.
#[test]
fn sem_contador_a_caixa_nao_e_pintada() {
    let mut i = info();
    i.has_counter = false;
    let (mut h, mut st) = host(Some(i));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rect_de(&rects, ids::INSP_HUD_COUNTER_KEEP).is_none(),
        "a caixa do contador foi pintada num objecto sem contador"
    );
    assert!(
        rect_de(&rects, ids::INSP_HUD_DISABLED).is_some(),
        "CONTROLO: a do botao continua la'"
    );
    set_current_inspector_hud(None);
}
