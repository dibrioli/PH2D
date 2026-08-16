//! **A lei do scrub conhece o intervalo que o COMMIT enforça.**
//!
//! O `apply_value_changed` deste painel clampa várias caixas num intervalo declarado, e até
//! 2026-08-16 a lei do arrasto (`WidgetStore::number_scrub_law`) não sabia de nenhum: o campo
//! era **clampado num intervalo conhecido e arrastado pelo atalho histórico**, que é a mesma
//! forma do defeito que a wave 4 curou na família do slider ligado, uma camada mais fora.
//!
//! ⚠️ **O oráculo NÃO é a constante que o `populate` regista** — comparar as duas seria a
//! constante contra si mesma, verde sob qualquer lei. Ele é o **intervalo MEDIDO**: o gate
//! dirige o commit muito acima e muito abaixo, lê o estado resultante, e exige que a faixa
//! registada seja exactamente onde o commit parou. Mudar a const num sítio só sangra.
//!
//! *Mutações que sangram:* apagar o laço de `set_number_range` do `populate` (a faixa some ⇒
//! `bounds` vira `None`) · mover um dos extremos da const (o commit e a faixa deixam de
//! coincidir) · devolver o atalho no `number_scrub_law` (a travessia deixa de ser 250 px).

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::{GridSnapState, ids as gs_ids};
use ph2d_editor_core::interaction::drag::DRAG_RANGE_PX_H;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, PanelRegistry};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_panel_grid_snap::GridSnapPanel;
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<GridSnapPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn setup_hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    // Os cinco campos deste gate são ADIMENSIONAIS (contagens, componentes sRGB), então a
    // unidade do projecto não os toca — mas pinar Meters mantém a fixture igual à da irmã e
    // torna a premissa declarada em vez de herdada.
    hero.project.display_unit = DisplayUnit::Meters;
    hero
}

/// Escreve `v` na caixa e dispara o commit REAL do painel.
fn commit(hero: &mut HeroScreen, id: NodeId, v: f64) {
    if let Some(InteractiveState::NumberInput { value, .. }) = hero.store.get_mut(id) {
        *value = v;
    }
    let _ = hero.apply_event(WidgetEvent::ValueChanged(id));
}

/// Um campo declarado: o nome para a mensagem, o id, e como se lê no estado o número que o
/// commit deixou.
type DeclaredField = (&'static str, NodeId, fn(&GridSnapState) -> f64);

/// Os cinco campos e como se lê, no estado, o número que o commit deixou.
fn declared_fields() -> Vec<DeclaredField> {
    vec![
        ("lloyd", gs_ids::GS_CFG_VORONOI_LLOYD_ITERS, |s| {
            f64::from(s.voronoi_cfg.lloyd_iterations)
        }),
        ("color_r", gs_ids::GS_CFG_COLOR_R, |s| {
            f64::from(s.color_rgba[0])
        }),
        ("color_g", gs_ids::GS_CFG_COLOR_G, |s| {
            f64::from(s.color_rgba[1])
        }),
        ("color_b", gs_ids::GS_CFG_COLOR_B, |s| {
            f64::from(s.color_rgba[2])
        }),
        ("subdivisions", gs_ids::GS_CFG_SNAP_SUBDIVISIONS, |s| {
            f64::from(s.snap_subdivisions)
        }),
    ]
}

/// ⭐ **A faixa registada é EXACTAMENTE onde o commit para.**
#[test]
fn the_scrub_law_knows_the_interval_the_commit_enforces() {
    for (name, id, read) in declared_fields() {
        // O intervalo REAL, medido pela porta do produto: empurra o commit para fora dos dois
        // lados e lê onde ele parou.
        let mut hero = setup_hero();
        commit(&mut hero, id, 1.0e6);
        let ceiling = read(&hero.grid.snap_state);
        commit(&mut hero, id, -1.0e6);
        let floor = read(&hero.grid.snap_state);
        assert!(
            floor < ceiling,
            "{name}: o commit tem de declarar um intervalo NAO-degenerado; veio [{floor}, {ceiling}]"
        );

        let law = hero.store.number_scrub_law(id, 1.0);
        let bounds = law
            .bounds
            .unwrap_or_else(|| panic!("{name}: a caixa e' clampada e o arrasto nao sabe a faixa"));
        assert!(
            (bounds.0 - floor).abs() < 1e-9 && (bounds.1 - ceiling).abs() < 1e-9,
            "{name}: a faixa do arrasto {bounds:?} tem de ser onde o commit para, [{floor}, {ceiling}]"
        );
    }
}

/// **E ela atravessa o campo inteiro no alvo de desenho, não numa fração de pixel.**
///
/// O número que a wave existe para mover: a `[0, 8]` das iterações de Lloyd era cruzada em
/// `8 / 50 = 0,16 px` pelo atalho — um pixel de arrasto saturava o campo seis vezes.
#[test]
fn a_declared_interval_is_crossed_in_the_design_target_not_in_a_fraction_of_a_pixel() {
    let hero = setup_hero();
    for (name, id, _) in declared_fields() {
        let law = hero.store.number_scrub_law(id, 1.0);
        let (lo, hi) = law.bounds.expect("faixa registada");
        let px = (hi - lo) / law.rate_x;
        assert!(
            (px - DRAG_RANGE_PX_H).abs() < 1e-6,
            "{name}: atravessa-se em {px:.2} px; o alvo e' {DRAG_RANGE_PX_H:.0}"
        );
    }
}

/// **O CONTROLE: quem o commit NÃO clampa continua no atalho.**
///
/// Sem ele, *"dei faixa a quem o commit já clampava"* seria indistinguível de *"dei faixa a
/// tudo"* — e dar faixa a uma origem (que não tem fim) inventaria uma fronteira, que é o que a
/// §0 do `CLAUDE.md` proíbe. É o espelho exacto do controle que a wave 4 escreveu.
#[test]
fn a_field_the_commit_does_not_clamp_still_has_no_interval() {
    let hero = setup_hero();
    for (name, id) in [
        ("origin_x", gs_ids::GS_CFG_ORIGIN_X),
        ("probe_a_x", gs_ids::GS_PROBE_A_X),
        ("voronoi_rng_seed", gs_ids::GS_CFG_VORONOI_RNG_SEED),
    ] {
        assert!(
            hero.store.number_scrub_law(id, 1.0).bounds.is_none(),
            "{name}: nao tem fim no commit, logo nao pode ganhar fronteira aqui"
        );
    }
}
