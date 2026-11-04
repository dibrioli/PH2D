//! ⭐⭐ **O painel Upscale em cartões: nenhum controlo morreu**, e o campo mostra a entrega.

use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_panel_upscale::UpscalePanel;
use ph2d_panel_upscale::state::{UpscalePanelState, set_current_upscale_snapshot};
use ph2d_tool_upscale::params::{UpscaleAlgorithm, UpscaleUiSnapshot};
use ph2d_tool_upscale::tool::ids as t;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_ui_testkit::mesa::Mesa;

const PANEL: ph2d_a11y::NodeId = ph2d_editor_core::ids::UPS_PANEL;

fn mesa(snap: UpscaleUiSnapshot) -> Mesa<UpscalePanel> {
    force_cards_on_this_thread(Some(true));
    set_current_upscale_snapshot(Some(snap));
    Mesa::nova(UpscalePanelState)
}

#[test]
fn every_id_of_the_old_panel_is_painted_and_the_bar_is_not() {
    let m = mesa(UpscaleUiSnapshot::default());
    let pintados = m.pintados();
    for id in [
        t::UPS_ALGO_LANCZOS3,
        t::UPS_ALGO_NEAREST,
        t::UPS_ALGO_EPX,
        t::UPS_SCALE_NUM,
        t::UPS_RESET,
        t::UPS_CANCEL,
        t::UPS_APPLY,
        t::UPS_SEC_SCALE,
    ] {
        assert!(pintados.contains(&id), "{id:?} nunca pintado");
    }
    assert!(!pintados.contains(&t::UPS_SCALE), "a barra ainda se pinta");
}

/// ⭐ O scrub pede o trilho pela barra (a ferramenta projeta-o) e o Esc devolve.
#[test]
fn scrubbing_the_scale_reaches_the_tool_and_escape_gives_it_back() {
    let snap = UpscaleUiSnapshot {
        algorithm: UpscaleAlgorithm::Nearest,
        ..UpscaleUiSnapshot::default()
    };
    let mut m = mesa(snap);
    let (arrastado, voltou) = m.prova_o_scrub(t::UPS_SCALE_NUM, t::UPS_SCALE, 100.0);
    assert!(arrastado > voltou, "arrastar para a direita aumenta");
}

#[test]
fn typing_the_scale_reaches_the_tool() {
    let mut m = mesa(UpscaleUiSnapshot::default());
    let track = m.prova_a_escrita(t::UPS_SCALE_NUM, t::UPS_SCALE, "4");
    let esperado = ph2d_tool_upscale::params::scale_to_slider(4.0);
    assert!(
        (track as f32 - esperado).abs() < 1e-3,
        "{track} e {esperado}"
    );
}

#[test]
fn every_algorithm_and_verb_answers_a_real_click() {
    let mut m = mesa(UpscaleUiSnapshot::default());
    for id in [
        t::UPS_ALGO_LANCZOS3,
        t::UPS_ALGO_NEAREST,
        t::UPS_ALGO_EPX,
        t::UPS_RESET,
        t::UPS_APPLY,
    ] {
        m.prova_o_clique(id);
    }
    m.prova_o_cancelar(t::UPS_CANCEL);
}

/// ⭐ O EPX arredonda: em repouso a caixa guarda o que o Apply entrega, nunca o pedido.
#[test]
fn the_field_holds_the_delivery_not_the_request() {
    let m = mesa(UpscaleUiSnapshot {
        algorithm: UpscaleAlgorithm::Epx,
        scale_factor: 2.4,
        ..UpscaleUiSnapshot::default()
    });
    let valor = m.host.store().number_value(t::UPS_SCALE_NUM);
    assert_eq!(valor, Some(2.0), "o campo mentiu sobre a entrega");
}

#[test]
fn the_verbs_live_in_the_strip_and_stay_when_the_card_folds() {
    let mut m = mesa(UpscaleUiSnapshot::default());
    m.prova_a_faixa(PANEL, &[t::UPS_SEC_SCALE], &[t::UPS_RESET, t::UPS_APPLY]);
    m.prova_a_dobra(t::UPS_SEC_SCALE, t::UPS_SCALE_NUM);
    assert!(m.rect(t::UPS_APPLY).is_some());
}

#[test]
fn cards_off_paints_the_old_body() {
    force_cards_on_this_thread(Some(false));
    set_current_upscale_snapshot(Some(UpscaleUiSnapshot::default()));
    let mut host = MockPanelHost::with_panel::<UpscalePanel>();
    let mut st = UpscalePanelState;
    let rects = host.paint::<UpscalePanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 340.0, 900.0),
    );
    assert!(rects.iter().any(|(id, _)| *id == t::UPS_SCALE));
    assert!(host.store().card_field(t::UPS_SCALE_NUM).is_none());
    force_cards_on_this_thread(None);
}
