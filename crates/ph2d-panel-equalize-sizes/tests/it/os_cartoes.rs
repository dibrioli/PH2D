//! ⭐⭐ **O painel Equalize Sizes em cartões: nenhum controlo morreu.**

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_panel_equalize_sizes::EqualizeSizesPanel;
use ph2d_panel_equalize_sizes::state::{
    EqualizeSizesPanelState, set_current_equalize_sizes_snapshot,
};
use ph2d_tool_equalize_sizes::ids as t;
use ph2d_tool_equalize_sizes::params::{EqualizeSizesUiSnapshot, TargetMode};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_ui_testkit::mesa::Mesa;

const PANEL: NodeId = ph2d_editor_core::ids::EQS_PANEL;

fn mesa(snap: EqualizeSizesUiSnapshot) -> Mesa<EqualizeSizesPanel> {
    force_cards_on_this_thread(Some(true));
    set_current_equalize_sizes_snapshot(Some(snap));
    Mesa::nova(EqualizeSizesPanelState)
}

fn fixo() -> EqualizeSizesUiSnapshot {
    EqualizeSizesUiSnapshot {
        target_mode: TargetMode::Fixed,
        upscale_if_smaller: true,
        ..EqualizeSizesUiSnapshot::default()
    }
}

fn grelha() -> EqualizeSizesUiSnapshot {
    EqualizeSizesUiSnapshot {
        target_mode: TargetMode::GridUnit,
        grid_unit: 64,
        ..EqualizeSizesUiSnapshot::default()
    }
}

/// ⭐ Lei da compactação: as duas caixas do Result dividem a linha. A caixa do Offset desce: não cabe
/// na coluna (lei 5). *Mutação: um por linha ⇒ vermelho.*
#[test]
fn irmaos_curtos_dividem_a_linha() {
    let m0 = mesa(fixo());
    let (a, b) = (
        m0.rect(t::EQS_UPSCALE_IF_SMALLER).expect("upscale"),
        m0.rect(t::EQS_RASTERIZE_AFTER).expect("rasterize"),
    );
    assert_eq!(a.y, b.y, "Upscale e Rasterize na mesma linha");
    assert!(a.x + a.w <= b.x, "sem sobreposição");
    let m1 = mesa(grelha());
    assert!(m1.rect(t::EQS_GRID_OFFSET_NUM).is_some(), "o Offset vive");
    assert!(m1.rect(t::EQS_ARRANGE_ON_GRID).is_some(), "o Arrange vive");
}

/// Tudo o que o corpo de antes pintava pinta-se em ALGUM modo; a barra do Offset nunca.
#[test]
fn every_id_of_the_old_panel_is_painted_in_some_mode_and_the_bar_never() {
    let mut pintados = Vec::new();
    for snap in [EqualizeSizesUiSnapshot::default(), fixo(), grelha()] {
        pintados.extend(mesa(snap).pintados());
    }
    for id in [
        t::EQS_MODE_MAX,
        t::EQS_MODE_FIXED,
        t::EQS_MODE_GRID,
        t::EQS_FIXED_W,
        t::EQS_FIXED_H,
        t::EQS_GRID_OFFSET_NUM,
        t::EQS_ARRANGE_ON_GRID,
        t::EQS_UPSCALE_IF_SMALLER,
        t::EQS_ALG_LANCZOS,
        t::EQS_ALG_NEAREST,
        t::EQS_ALG_EPX,
        t::EQS_RASTERIZE_AFTER,
        t::EQS_RESET,
        t::EQS_CANCEL,
        t::EQS_APPLY,
        t::EQS_SEC_TARGET,
        t::EQS_SEC_RESULT,
    ] {
        assert!(pintados.contains(&id), "{id:?} nunca pintado");
    }
    assert!(!pintados.contains(&t::EQS_GRID_OFFSET));
}

/// ⭐ W e H não têm barra: é o próprio chip que fala com a ferramenta, e o Esc tem de a avisar.
#[test]
fn scrubbing_width_and_height_reaches_the_tool_and_escape_gives_it_back() {
    for chip in [t::EQS_FIXED_W, t::EQS_FIXED_H] {
        let mut m = mesa(fixo());
        m.prova_o_scrub(chip, chip, 40.0);
    }
}

/// O Offset fala pelo chip, e o evento espelha-o na barra.
#[test]
fn scrubbing_the_offset_reaches_the_tool_and_escape_gives_it_back() {
    let mut m = mesa(grelha());
    let chip = t::EQS_GRID_OFFSET_NUM;
    let (arrastado, voltou) = m.prova_o_scrub(chip, chip, 8.0);
    assert!(arrastado > voltou);
}

#[test]
fn typing_a_size_reaches_the_tool() {
    let mut m = mesa(fixo());
    assert_eq!(
        m.prova_a_escrita(t::EQS_FIXED_W, t::EQS_FIXED_W, "128"),
        128.0
    );
}

#[test]
fn every_mode_check_algorithm_and_verb_answers_a_real_click() {
    let mut m = mesa(fixo());
    for id in [
        t::EQS_MODE_MAX,
        t::EQS_MODE_FIXED,
        t::EQS_MODE_GRID,
        t::EQS_UPSCALE_IF_SMALLER,
        t::EQS_ALG_LANCZOS,
        t::EQS_ALG_NEAREST,
        t::EQS_ALG_EPX,
        t::EQS_RASTERIZE_AFTER,
        t::EQS_RESET,
        t::EQS_APPLY,
    ] {
        m.prova_o_clique(id);
    }
    m.prova_o_cancelar(t::EQS_CANCEL);
    let mut m = mesa(grelha());
    m.prova_o_clique(t::EQS_ARRANGE_ON_GRID);
}

#[test]
fn the_cards_fold_and_the_verbs_stay() {
    let mut m = mesa(fixo());
    m.prova_a_faixa(
        PANEL,
        &[t::EQS_SEC_TARGET, t::EQS_SEC_RESULT],
        &[t::EQS_RESET, t::EQS_APPLY],
    );
    m.prova_a_dobra(t::EQS_SEC_TARGET, t::EQS_FIXED_W);
    m.prova_a_dobra(t::EQS_SEC_RESULT, t::EQS_RASTERIZE_AFTER);
    assert!(m.rect(t::EQS_APPLY).is_some());
}

#[test]
fn cards_off_paints_the_old_body() {
    force_cards_on_this_thread(Some(false));
    set_current_equalize_sizes_snapshot(Some(grelha()));
    let mut host = MockPanelHost::with_panel::<EqualizeSizesPanel>();
    let mut st = EqualizeSizesPanelState;
    let rects = host.paint::<EqualizeSizesPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 340.0, 900.0),
    );
    assert!(rects.iter().any(|(id, _)| *id == t::EQS_GRID_OFFSET));
    assert!(host.store().card_field(t::EQS_GRID_OFFSET_NUM).is_none());
    force_cards_on_this_thread(None);
}
