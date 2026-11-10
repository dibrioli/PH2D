//! **A pintura do ecrã esquece as alturas que não voltou a publicar** (D9 da spec
//! `04_a_rolagem_unica`) — o gate da FIAÇÃO: as leis vivem no `scroll_state_tests`, e aqui prova-se
//! que o `paint_hero_screen` abre e fecha o quadro de rolagem à volta de todos os pintores.
//!
//! ⚠️ Os dois lados passam pela pintura REAL: a lista do dropdown dos kernels do Convolution Matrix
//! publica pela porta enquanto está aberta (o CONTROLO) e deixa de publicar quando fecha. (Até à
//! R2a do Input Map, 10/11, a superfície era a janela flutuante do mapa, que saiu.)

use crate::the_pixel_edit_dialogs_are_alive::Ecra;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::pixel_panel::PixelDialogKind;
use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Kernels de sobra para a lista transbordar a janela baixa.
const NOMES: &[&str] = &[
    "k00", "k01", "k02", "k03", "k04", "k05", "k06", "k07", "k08", "k09", "k10", "k11", "k12",
    "k13", "k14", "k15", "k16", "k17", "k18", "k19", "k20", "k21", "k22", "k23", "k24", "k25",
    "k26", "k27", "k28", "k29", "k30", "k31", "k32", "k33", "k34", "k35", "k36", "k37", "k38",
];

#[test]
fn a_lista_fechada_deixa_de_ter_alturas_e_a_roda_nao_a_mexe() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut e = Ecra {
        hero: HeroScreen::new(NodeId(1)),
        text: TextSystem::without_system_fonts(),
        janela: Some(Rect::new(0.0, 0.0, 1280.0, 420.0)),
    };
    e.hero.store.set_pixel_filter_kernels(NOMES);
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::ConvolutionMatrix, (32, 24));
    e.pinta();
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    assert!(
        e.hero.store.pixel_filter_kernel_list_open(),
        "fixtura: a lista abriu"
    );
    let lista = ids::PIXEL_FILTER_KERNEL;
    // CONTROLO: aberta, ela publica pela porta e a lista transborda.
    let content = e
        .hero
        .store
        .panel_content_h(lista)
        .expect("a lista aberta publica a altura do conteúdo");
    let visible = e.hero.store.panel_visible_h(lista).expect("e a visível");
    assert!(
        content > visible,
        "a fixtura não transborda — o gate não mediria nada"
    );
    e.hero.store.wheel_panel(lista, -60.0);
    let deixado = e.hero.store.panel_scroll_target(lista);
    assert!(deixado > 0.0, "CONTROLO: aberta, a roda rola-a");

    assert!(e.hero.store.close_pixel_filter_kernel_list());
    e.pinta();
    assert_eq!(
        e.hero.store.panel_content_h(lista),
        None,
        "a lista fechou e a altura do quadro em que ela rolava sobreviveu"
    );
    assert_eq!(e.hero.store.panel_visible_h(lista), None);
    e.hero.store.wheel_panel(lista, -600.0);
    assert_eq!(
        e.hero.store.panel_scroll_target(lista),
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
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(
        &mut hero,
        Rect::new(0.0, 0.0, 1600.0, 360.0),
        &mut scene,
        &mut text,
    );
    assert_eq!(hero.store.panel_content_h(fantasma), None);
    assert_eq!(hero.store.panel_visible_h(fantasma), None);
}
