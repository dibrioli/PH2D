//! ⭐⭐ **O painel Background Removal em cartões: nenhum controlo morreu.**

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_panel_bgremoval::BgRemovalPanel;
use ph2d_panel_bgremoval::state::{BgRemovalPanelState, set_current_bgremoval_snapshot};
use ph2d_tool_bgremoval::ids as t;
use ph2d_tool_bgremoval::params::BgRemovalUiSnapshot;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_ui_testkit::mesa::Mesa;

const PANEL: NodeId = ph2d_editor_core::ids::BGR_PANEL;
const NUMEROS: [(NodeId, NodeId); 4] = [
    (t::BGR_TOLERANCE_NUM, t::BGR_TOLERANCE),
    (t::BGR_FEATHER_NUM, t::BGR_FEATHER),
    (t::BGR_REFINE_NUM, t::BGR_REFINE),
    (t::BGR_GROW_NUM, t::BGR_GROW),
];

fn mesa(snap: BgRemovalUiSnapshot) -> Mesa<BgRemovalPanel> {
    force_cards_on_this_thread(Some(true));
    set_current_bgremoval_snapshot(Some(snap));
    Mesa::nova(BgRemovalPanelState)
}

fn tudo_ligado() -> BgRemovalUiSnapshot {
    BgRemovalUiSnapshot {
        separate_islands: true,
        protect_brush_armed: true,
        has_protect_mask: true,
        extra_colors: vec![[255, 0, 0], [0, 255, 0]],
        ..BgRemovalUiSnapshot::default()
    }
}

fn com_sujeito() -> BgRemovalUiSnapshot {
    BgRemovalUiSnapshot {
        auto_protect_subject: true,
        has_force_remove_mask: true,
        ..BgRemovalUiSnapshot::default()
    }
}

/// ⭐ Lei da compactação: a caixa e o que ela liga dividem a linha, sem cruzar colunas.
/// *Mutação: pintar um por linha ⇒ `y` diferente.*
#[test]
fn irmaos_curtos_dividem_a_linha() {
    let m0 = mesa(tudo_ligado());
    for (a, b) in [
        (t::BGR_SEPARATE_ISLANDS, t::BGR_MIN_ISLAND_PX_NUM),
        (t::BGR_AUTO_PROTECT_SUBJECT, t::BGR_EYEDROPPER),
    ] {
        let (ra, rb) = (m0.rect(a).expect("a"), m0.rect(b).expect("b"));
        assert_eq!(ra.y, rb.y, "{a:?} e {b:?} na mesma linha");
        assert!(ra.x + ra.w <= rb.x, "{a:?} e {b:?} sem sobreposição");
    }
}

/// Tudo o que o corpo de antes pintava pinta-se em ALGUM estado; as barras nunca.
#[test]
fn every_id_of_the_old_panel_is_painted_in_some_state_and_no_bar_ever() {
    let mut pintados = Vec::new();
    for snap in [BgRemovalUiSnapshot::default(), tudo_ligado(), com_sujeito()] {
        pintados.extend(mesa(snap).pintados());
    }
    for id in [
        t::BGR_SEPARATE_ISLANDS,
        t::BGR_MIN_ISLAND_PX_NUM,
        t::BGR_ADD_AREA,
        t::BGR_ADD_AREA_CLEAR,
        t::BGR_EYEDROPPER,
        ph2d_editor_core::ids::BGR_SWATCHES[0],
        ph2d_editor_core::ids::BGR_SWATCHES[1],
        t::BGR_AUTO_PROTECT_SUBJECT,
        t::BGR_PROTECT,
        t::BGR_BRUSH_SIZE_NUM,
        t::BGR_FALLOFF_SMOOTH,
        t::BGR_FALLOFF_SPHERE,
        t::BGR_FALLOFF_SHARP,
        t::BGR_FALLOFF_CONSTANT,
        t::BGR_SHOW_MASK,
        t::BGR_PROTECT_CLEAR,
        t::BGR_RESET,
        t::BGR_CANCEL,
        t::BGR_APPLY,
    ]
    .into_iter()
    .chain(NUMEROS.map(|(chip, _)| chip))
    {
        assert!(pintados.contains(&id), "{id:?} nunca pintado");
    }
    for barra in NUMEROS
        .map(|(_, b)| b)
        .into_iter()
        .chain([t::BGR_MIN_ISLAND_PX, t::BGR_BRUSH_SIZE])
    {
        assert!(
            !pintados.contains(&barra),
            "a barra {barra:?} ainda se pinta"
        );
    }
}

/// ⭐ Cada número pelo scrub, com o Esc a devolver, pela barra que fala com a ferramenta.
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, barra) in NUMEROS {
        mesa(BgRemovalUiSnapshot::default()).prova_o_scrub(chip, barra, 40.0);
    }
    for (chip, barra) in [
        (t::BGR_MIN_ISLAND_PX_NUM, t::BGR_MIN_ISLAND_PX),
        (t::BGR_BRUSH_SIZE_NUM, t::BGR_BRUSH_SIZE),
    ] {
        mesa(tudo_ligado()).prova_o_scrub(chip, barra, 40.0);
    }
}

#[test]
fn typing_a_number_reaches_the_tool() {
    let mut m = mesa(BgRemovalUiSnapshot::default());
    let v = m.prova_a_escrita(t::BGR_GROW_NUM, t::BGR_GROW, "0.5");
    assert!((v - 0.75).abs() < 1e-3, "Grow 0.5 é o trilho 0.75: {v}");
}

#[test]
fn every_check_option_and_verb_answers_a_real_click() {
    let mut m = mesa(tudo_ligado());
    for id in [
        t::BGR_SEPARATE_ISLANDS,
        t::BGR_EYEDROPPER,
        t::BGR_AUTO_PROTECT_SUBJECT,
        t::BGR_PROTECT,
        t::BGR_FALLOFF_SMOOTH,
        t::BGR_FALLOFF_SPHERE,
        t::BGR_FALLOFF_SHARP,
        t::BGR_FALLOFF_CONSTANT,
        t::BGR_SHOW_MASK,
        t::BGR_PROTECT_CLEAR,
        t::BGR_RESET,
        t::BGR_APPLY,
    ] {
        m.prova_o_clique(id);
    }
    m.prova_o_cancelar(t::BGR_CANCEL);
    let mut m = mesa(com_sujeito());
    m.prova_o_clique(t::BGR_ADD_AREA);
    m.prova_o_clique(t::BGR_ADD_AREA_CLEAR);
}

/// Uma amostra de cor extra responde no rect pintado, com o id da posição dela.
#[test]
fn an_extra_colour_swatch_is_hit_by_its_own_id() {
    let m = mesa(tudo_ligado());
    let id = ph2d_editor_core::ids::BGR_SWATCHES[1];
    let r = m.rect(id).expect("a amostra pinta-se");
    assert_eq!(m.host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5), Some(id));
}

#[test]
fn the_cards_fold_and_the_verbs_stay() {
    let mut m = mesa(tudo_ligado());
    m.prova_a_faixa(
        PANEL,
        &[t::BGR_SEC_MASK, t::BGR_SEC_PROTECT],
        &[t::BGR_RESET, t::BGR_APPLY],
    );
    m.prova_a_dobra(t::BGR_SEC_MASK, t::BGR_TOLERANCE_NUM);
    m.prova_a_dobra(t::BGR_SEC_COLORS, t::BGR_EYEDROPPER);
    m.prova_a_dobra(t::BGR_SEC_ISLANDS, t::BGR_MIN_ISLAND_PX_NUM);
    m.prova_a_dobra(t::BGR_SEC_PROTECT, t::BGR_BRUSH_SIZE_NUM);
    assert!(m.rect(t::BGR_APPLY).is_some());
}

#[test]
fn cards_off_paints_the_old_body() {
    force_cards_on_this_thread(Some(false));
    set_current_bgremoval_snapshot(Some(BgRemovalUiSnapshot::default()));
    let mut host = MockPanelHost::with_panel::<BgRemovalPanel>();
    let mut st = BgRemovalPanelState;
    let rects = host.paint::<BgRemovalPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 340.0, 900.0),
    );
    assert!(rects.iter().any(|(id, _)| *id == t::BGR_TOLERANCE));
    assert!(host.store().card_field(t::BGR_TOLERANCE_NUM).is_none());
    force_cards_on_this_thread(None);
}
