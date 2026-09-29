//! **A pintura do ecrã esquece as alturas que não voltou a publicar** (D9 da spec
//! `04_a_rolagem_unica`) — o gate da FIAÇÃO: as leis vivem no `scroll_state_tests`, e aqui prova-se
//! que o `paint_hero_screen` abre e fecha o quadro de rolagem à volta de todos os pintores.
//!
//! ⚠️ Os dois lados passam pela pintura REAL: a janela do Input Map publica pela porta enquanto
//! está aberta (o CONTROLO) e deixa de publicar quando fecha.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{HeroScreen, chrome, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Baixa de propósito: a lista da janela tem de transbordar.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 360.0,
};

fn pinta(hero: &mut HeroScreen) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(hero, VIEWPORT, &mut scene, &mut text);
}

#[test]
fn a_janela_fechada_deixa_de_ter_alturas_e_a_roda_nao_a_mexe() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    for i in 0..24 {
        hero.input_map.create(format!("accao_{i}"));
    }
    hero.store.open_input_map(48.0, 8.0);
    chrome::sync_input_map_rows(&mut hero.store, &hero.input_map);
    pinta(&mut hero);
    // CONTROLO: aberta, ela publica pela porta e a lista transborda.
    let content = hero
        .store
        .panel_content_h(ids::INPUT_MAP_SURFACE)
        .expect("a janela aberta publica a altura do conteúdo");
    let visible = hero
        .store
        .panel_visible_h(ids::INPUT_MAP_SURFACE)
        .expect("e a visível");
    assert!(
        content > visible,
        "a fixtura não transborda — o gate não mediria nada"
    );
    hero.store.wheel_panel(ids::INPUT_MAP_SURFACE, -60.0);
    let deixado = hero.store.panel_scroll_target(ids::INPUT_MAP_SURFACE);
    assert!(deixado > 0.0, "CONTROLO: aberta, a roda rola-a");

    hero.store.close_input_map();
    pinta(&mut hero);
    assert_eq!(
        hero.store.panel_content_h(ids::INPUT_MAP_SURFACE),
        None,
        "a janela fechou e a altura do quadro em que ela rolava sobreviveu"
    );
    assert_eq!(hero.store.panel_visible_h(ids::INPUT_MAP_SURFACE), None);
    hero.store.wheel_panel(ids::INPUT_MAP_SURFACE, -600.0);
    assert_eq!(
        hero.store.panel_scroll_target(ids::INPUT_MAP_SURFACE),
        deixado,
        "a roda mexeu numa lista que ninguém desenha"
    );
}

/// Uma altura escrita por um id que NENHUM pintor publica é esquecida pelo quadro seguinte.
#[test]
fn um_id_que_ninguem_pinta_perde_as_alturas_no_quadro_seguinte() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let fantasma = NodeId(0xD9_D9_D9);
    hero.store.set_panel_content_h(fantasma, 900.0);
    hero.store.set_panel_visible_h(fantasma, 100.0);
    pinta(&mut hero);
    assert_eq!(hero.store.panel_content_h(fantasma), None);
    assert_eq!(hero.store.panel_visible_h(fantasma), None);
}
