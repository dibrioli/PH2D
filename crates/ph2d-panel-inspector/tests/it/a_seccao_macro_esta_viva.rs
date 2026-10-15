//! **A secção MACRO é PINTADA e está VIVA sob o dedo** (ADR-0186, W5d).
//!
//! O gesto é REAL (`Down` + `Up` no rectângulo que a pintura registou): um `WidgetEvent::Click`
//! sintético passaria sobre um controlo ausente do `populate`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::macro_edits::{
    ErroDaMacroView, InspectorMacroInfo, MODO_EXPLICATIVO, MODO_FIEL, MacroFieldEdit as E,
    MacroGravacao,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::censo_dos_avisos;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_macro};
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

fn info(editada_desde: Option<usize>, erro: Option<ErroDaMacroView>) -> InspectorMacroInfo {
    InspectorMacroInfo {
        entity_bits: BITS,
        modo: MODO_FIEL,
        gravacao: MacroGravacao::Lida {
            trilhas: vec![("painter".into(), 3)],
            duracao_us: 1_500_000,
        },
        selected_count: 1,
        etapas: 3,
        editada_desde,
        erro,
    }
}

/// Pinta o Inspector com `i`; devolve o arnês, o estado, os rectângulos e as frases do cartão.
fn pinta(
    i: InspectorMacroInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
    Vec<String>,
) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_macro(Some(i));
    let mut st = InspectorState::default();
    let (rects, avisos) =
        censo_dos_avisos::medindo(|| h.paint::<InspectorPanel>(&mut st, VIEWPORT));
    let frases = avisos
        .into_iter()
        .filter(|a| a.seccao() == "macro_card")
        .map(|a| a.texto)
        .collect();
    (h, st, rects, frases)
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
                edit: ComponentEdit::Macro(e),
            } => {
                assert_eq!(*entity_bits, BITS);
                Some(e.clone())
            }
            _ => None,
        })
        .collect()
}

/// O clique num MODO envia o índice dele; «Edit steps…» abre o painel MACROS e não edita nada.
///
/// **Mutação que deve sangrar:** apagar o `set_panel_visible` do despacho.
#[test]
fn o_modo_edita_e_editar_etapas_abre_o_painel() {
    let (mut h, mut st, rects, _) = pinta(info(None, None));
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MACRO_MODO[1]),
        vec![E::Modo(MODO_EXPLICATIVO)]
    );
    assert!(!h.panel_visible("macros"), "o painel começa fechado");
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MACRO_EDITAR_ETAPAS),
        vec![],
        "abrir o painel não é uma edição"
    );
    assert!(
        h.panel_visible("macros"),
        "«Edit steps…» abre o painel MACROS"
    );
}

/// «Editada desde a etapa N» (a contar de 1) e o erro só existem quando o instantâneo os traz.
///
/// **Mutação que deve sangrar:** pintar a linha sem o `if let Some(n)`, ou com `n` em vez de `n + 1`.
#[test]
fn a_editada_desde_e_o_erro_pintam_se_so_quando_existem() {
    let editada = ph2d_i18n::tr_with("panel.macros.edited_since", &[("n", &3)]);
    let erro = ph2d_i18n::tr_with(
        "panel.macros.error.missing_preset",
        &[("detail", &"Soft Round")],
    );
    let (_h, _st, rects, limpo) = pinta(info(None, None));
    assert!(rect_de(&rects, ids::INSP_MACRO_EDITAR_ETAPAS).is_some());
    assert!(
        !limpo.contains(&editada) && !limpo.contains(&erro),
        "{limpo:?}"
    );
    let (_h, _st, _rects, cheio) = pinta(info(
        Some(2),
        Some(ErroDaMacroView {
            chave: "panel.macros.error.missing_preset",
            detalhe: Some("Soft Round".into()),
        }),
    ));
    assert!(
        cheio.contains(&editada),
        "«editada desde a etapa 3»: {cheio:?}"
    );
    assert!(cheio.contains(&erro), "o erro com o detalhe: {cheio:?}");
    assert_eq!(cheio.len(), limpo.len() + 2, "{limpo:?} → {cheio:?}");
}

/// Uma gravação que não se lê diz só isso — nem resumo, nem etapas.
#[test]
fn uma_gravacao_ilegivel_diz_so_porque() {
    let mut i = info(Some(0), None);
    i.gravacao = MacroGravacao::Ilegivel(ph2d_editor_core::macro_edits::MACRO_ESTRAGADA);
    let (_h, _st, _rects, frases) = pinta(i);
    let estragada = ph2d_i18n::tr(ph2d_editor_core::macro_edits::MACRO_ESTRAGADA);
    assert!(frases.iter().any(|f| f == estragada), "{frases:?}");
    assert!(
        !frases
            .iter()
            .any(|f| f == &ph2d_i18n::tr_with("panel.macros.edited_since", &[("n", &1)])),
        "{frases:?}"
    );
}

/// Sem instantâneo, nada da secção se pinta.
#[test]
fn sem_componente_nao_ha_seccao() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_macro(None);
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_MACRO_MODO[0]).is_none());
    assert!(rect_de(&rects, ids::INSP_MACRO_EDITAR_ETAPAS).is_none());
}
