//! ⭐ **F50 — Boolean e Convert saem do painel para a barra de ações ④** (dono 02/11: «faça como o
//! Figma» · «tirar do painel e deixar os mais comuns na barra de ações e "..." para as menos
//! comuns»). A booleana é sempre viva (a chave «Live» saiu); o permanente é o `Flatten`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{ToolBars, WidgetEvent};
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, bars, ids as p};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot, ids as t};
use ph2d_ui_testkit::MockPanelHost;

const COMUNS: [NodeId; 6] = [
    p::VECTOR_BOOL_UNION,
    p::VECTOR_BOOL_SUBTRACT,
    p::VECTOR_BOOL_INTERSECT,
    p::VECTOR_BOOL_EXCLUDE,
    p::VECTOR_FLATTEN,
    t::VECTOR_EXPAND_OUTLINE_STROKE,
];

const RARAS: [NodeId; 4] = [
    t::VECTOR_BOOL_MINUS_BACK,
    t::VECTOR_BOOL_TRIM,
    t::VECTOR_BOOL_CROP,
    t::VECTOR_BOOL_MERGE,
];

fn barras(sel: usize, grupo: Option<u8>) -> (MockPanelHost, ToolBars) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode: DrawMode::Select,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(sel);
    ph2d_panel_vector::state::set_bool_group(grupo);
    let h = crate::host_aberto();
    let b = bars::tool_bars(h.store(), None, Vec::new());
    (h, b)
}

fn acoes(b: &ToolBars) -> Vec<NodeId> {
    b.actions.iter().flatten().map(|x| x.id).collect()
}

fn mais(b: &ToolBars) -> Vec<Option<NodeId>> {
    b.actions_more.iter().map(ToolRailEntry::node_id).collect()
}

fn chega(h: &mut MockPanelHost, id: NodeId) {
    let out = h.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::Click(id));
    assert!(
        out == EventOutcome::Consumed && !h.drained_actions().is_empty(),
        "{id:?} está na barra e o painel não o despacha"
    );
}

/// Com duas formas a ④ traz as quatro booleanas, o Flatten e o Outline Stroke, nesta ordem e
/// juntos — e cada um chega à porta do painel.
#[test]
fn duas_formas_trazem_as_comuns_na_barra() {
    let (mut h, b) = barras(2, None);
    let ids = acoes(&b);
    let at = ids
        .iter()
        .position(|&i| i == COMUNS[0])
        .expect("a Union na ④");
    assert_eq!(&ids[at..at + COMUNS.len()], &COMUNS);
    for id in COMUNS {
        chega(&mut h, id);
    }
}

/// O «⋯» guarda as raras: as quatro receitas e o Compound/Release, separados por um risco.
#[test]
fn o_mais_guarda_as_raras() {
    let (mut h, b) = barras(2, None);
    let want: Vec<Option<NodeId>> = RARAS
        .iter()
        .map(|&i| Some(i))
        .chain([
            None,
            Some(ph2d_editor_core::ids::VECTOR_COMPOUND_MAKE),
            Some(p::VECTOR_COMPOUND_RELEASE),
        ])
        .collect();
    assert_eq!(mais(&b), want);
    for id in RARAS {
        chega(&mut h, id);
    }
    assert!(
        b.actions_more
            .iter()
            .all(|e| e.label().is_none_or(|l| !l.is_empty())),
        "linha sem nome"
    );
}

/// Uma forma solta: Outline Stroke sim, booleana e Flatten não; o «⋯» só com o Release.
#[test]
fn uma_forma_so_tem_o_outline_e_o_release() {
    let (_, b) = barras(1, None);
    let ids = acoes(&b);
    assert!(ids.contains(&t::VECTOR_EXPAND_OUTLINE_STROKE));
    for id in &COMUNS[..5] {
        assert!(!ids.contains(id), "{id:?} com uma forma só");
    }
    assert_eq!(mais(&b), vec![Some(p::VECTOR_COMPOUND_RELEASE)]);
}

/// A operação do grupo vivo escolhido ACENDE o botão dela — trocar é clicar noutro (Figma).
#[test]
fn a_operacao_do_grupo_acende() {
    for (op, aceso) in [(Some(2), Some(p::VECTOR_BOOL_INTERSECT)), (None, None)] {
        let (_, b) = barras(2, op);
        let acesos: Vec<NodeId> = b
            .actions
            .iter()
            .flatten()
            .filter(|x| x.on && COMUNS.contains(&x.id))
            .map(|x| x.id)
            .collect();
        assert_eq!(
            acesos,
            aceso.into_iter().collect::<Vec<_>>(),
            "grupo {op:?}"
        );
    }
    ph2d_panel_vector::state::set_bool_group(None);
}

/// ⛔ O painel já não pinta Boolean nem Convert — nem com o Advanced aberto.
#[test]
fn o_painel_ja_nao_tem_boolean_nem_convert() {
    let (mut h, _) = barras(2, None);
    let vp = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 6000.0,
    };
    assert!(
        h.painted_rect::<VectorPanel>(&mut VectorPanelState, vp, p::VECTOR_SECTION_ADVANCED)
            .is_some(),
        "controlo: o Advanced pinta"
    );
    for id in COMUNS.into_iter().chain(RARAS) {
        assert!(
            h.painted_rect::<VectorPanel>(&mut VectorPanelState, vp, id)
                .is_none(),
            "{id:?} ainda no painel"
        );
    }
}
