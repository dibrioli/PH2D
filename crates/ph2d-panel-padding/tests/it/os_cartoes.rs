//! ⭐⭐ **O painel Padding em cartões: nenhum controlo morreu.** Cada id pelo gesto REAL, nos rects
//! pintados, e o valor até à `spec()` da ferramenta.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::tool::Tool;
use ph2d_panel_padding::state::{PaddingPanelState, set_current_padding_snapshot};
use ph2d_panel_padding::{PaddingPanel, ids};
use ph2d_tool_padding::PaddingTool;
use ph2d_tool_padding::ids as t;
use ph2d_tool_padding::params::PaddingUiSnapshot;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_ui_testkit::mesa::Mesa;

const PANEL: ph2d_a11y::NodeId = ph2d_editor_core::ids::PAD_PANEL;
const LADOS: [(ph2d_a11y::NodeId, ph2d_a11y::NodeId); 4] = [
    (t::PAD_TOP_NUM, t::PAD_TOP),
    (t::PAD_RIGHT_NUM, t::PAD_RIGHT),
    (t::PAD_BOTTOM_NUM, t::PAD_BOTTOM),
    (t::PAD_LEFT_NUM, t::PAD_LEFT),
];

fn mesa(snap: PaddingUiSnapshot) -> Mesa<PaddingPanel> {
    force_cards_on_this_thread(Some(true));
    set_current_padding_snapshot(Some(snap));
    Mesa::nova(PaddingPanelState)
}

/// Tudo o que o corpo de antes pintava pinta-se aqui, salvo as barras (a pega faz-lhes o trabalho).
#[test]
fn every_id_of_the_old_panel_is_painted_and_no_bar_is() {
    let mesa = mesa(PaddingUiSnapshot::default());
    let pintados = mesa.pintados();
    for id in LADOS.map(|(chip, _)| chip).into_iter().chain([
        t::PAD_PIVOT_RECENTER,
        t::PAD_RESET,
        t::PAD_APPLY,
        ids::PAD_CANCEL,
        ids::PAD_SEC_EDGES,
    ]) {
        assert!(pintados.contains(&id), "{id:?} nunca pintado");
    }
    for (_, barra) in LADOS {
        assert!(
            !pintados.contains(&barra),
            "a barra {barra:?} ainda se pinta"
        );
    }
}

/// ⭐ Cada lado pelo scrub: o gesto pede o valor pela barra, o Esc devolve, e a ferramenta recebe
/// esse lado, e só esse. *Mutação: tirar o chip da tabela de `prepare` ⇒ não há scrub.*
#[test]
fn scrubbing_each_edge_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, barra) in LADOS {
        let mut m = mesa(PaddingUiSnapshot::default());
        let mut tool = PaddingTool::default();
        let ev = m.arrasta_a_pega(chip, 40.0);
        for a in m.aplica(ev) {
            if let EditorAction::ToolPanelEvent(pe) = a {
                tool.handle_panel_event(pe);
            }
        }
        let (top, right, bottom, left) = tool.spec();
        let mexidos = [top, right, bottom, left]
            .iter()
            .filter(|v| **v > 0)
            .count();
        assert_eq!(mexidos, 1, "{chip:?} escreveu {:?}", tool.spec());
        let ev = m.esc();
        for a in m.aplica(ev) {
            if let EditorAction::ToolPanelEvent(pe) = a {
                tool.handle_panel_event(pe);
            }
        }
        assert_eq!(tool.spec(), (0, 0, 0, 0), "o Esc de {barra:?} não devolveu");
    }
}

#[test]
fn typing_a_number_reaches_the_tool() {
    let mut m = mesa(PaddingUiSnapshot::default());
    let mut tool = PaddingTool::default();
    let ev = m.host.type_into_number(t::PAD_LEFT_NUM, "12");
    for a in m.aplica(ev) {
        if let EditorAction::ToolPanelEvent(pe) = a {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(tool.spec(), (0, 0, 0, 12));
}

/// A caixa do pivô, o Reset e o Apply chegam à ferramenta; o Cancel da faixa cancela.
#[test]
fn every_check_and_verb_answers_a_real_click() {
    let mut m = mesa(PaddingUiSnapshot::default());
    m.prova_o_clique(t::PAD_PIVOT_RECENTER);
    m.prova_o_clique(t::PAD_RESET);
    m.prova_o_clique(t::PAD_APPLY);
    m.prova_o_cancelar(ids::PAD_CANCEL);
}

/// ⭐ A caixa mostra o que a ferramenta tem: a marca segue a `recenter_pivot`, e o campo o número dela.
#[test]
fn the_card_shows_what_the_tool_holds() {
    let m = mesa(PaddingUiSnapshot {
        top: 7,
        ..PaddingUiSnapshot::default()
    });
    assert_eq!(m.host.store().number_value(t::PAD_TOP_NUM), Some(7.0));
}

/// ⭐ Os verbos ficam na faixa: abaixo dos cartões e dentro do rect do painel.
#[test]
fn the_verbs_live_in_the_strip_under_the_cards() {
    let m = mesa(PaddingUiSnapshot::default());
    m.prova_a_faixa(PANEL, &[ids::PAD_SEC_EDGES], &[t::PAD_RESET, t::PAD_APPLY]);
}

#[test]
fn the_card_folds_and_its_verbs_stay() {
    let mut m = mesa(PaddingUiSnapshot::default());
    m.prova_a_dobra(ids::PAD_SEC_EDGES, t::PAD_TOP_NUM);
    assert!(m.rect(t::PAD_APPLY).is_some(), "o Apply foi com o cartão");
}

/// `PH2D_CARDS=0`: o corpo de antes, com as barras e sem campo de cartão.
#[test]
fn cards_off_paints_the_old_body() {
    force_cards_on_this_thread(Some(false));
    set_current_padding_snapshot(Some(PaddingUiSnapshot::default()));
    let mut host = MockPanelHost::with_panel::<PaddingPanel>();
    let mut st = PaddingPanelState;
    let rects = host.paint::<PaddingPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 340.0, 900.0),
    );
    assert!(rects.iter().any(|(id, _)| *id == t::PAD_TOP));
    assert!(host.store().card_field(t::PAD_TOP_NUM).is_none());
    force_cards_on_this_thread(None);
}
