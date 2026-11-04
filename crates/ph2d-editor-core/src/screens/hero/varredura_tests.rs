//! A varredura final (`docs/CartoesUI/17_VARREDURA_FINAL.md`): o documento pára antes das duas listas, o
//! chão cobre a cena, os painéis e o aviso vestem a superfície da lista. `PH2D_CARDS=0` dá o de antes.

use crate::paint::{PaintCtx, fill_rounded_rect, resolve};
use crate::screens::HeroLayout;
use crate::toast::{Toast, ToastQueue};
use crate::widget::card_ink::CardInk;
use crate::widget::cromo_barra::Barra;
use crate::widget::lista_flutuante::{Superficie, force_cromo_on_this_thread};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

const TEMA: Theme = Theme::Dark;

fn janela() -> Rect {
    Rect::new(0.0, 0.0, 1930.0, 1040.0)
}

fn com_listas(esq: bool, dir: bool) -> HeroLayout {
    let mut l = HeroLayout::for_viewport(janela());
    l.docks.float_left = esq;
    l.docks.float_right = dir;
    // Com as listas a flutuar a área de desenho vai de ponta a ponta.
    l.draw_area = Rect::new(0.0, 76.0, 1930.0, 964.0);
    l.float_lists = [
        Rect::new(5.0, 100.0, 300.0, 400.0),
        Rect::new(1630.0, 100.0, 295.0, 400.0),
    ];
    l
}

fn vezes_cor(scene: &VectorScene, cor: Color) -> usize {
    let mut uma = VectorScene::new();
    fill_rounded_rect(&mut uma, Rect::new(0.0, 0.0, 1.0, 1.0), 0.0, cor);
    let dados = uma.inner().encoding().draw_data.clone();
    scene
        .inner()
        .encoding()
        .draw_data
        .windows(dados.len())
        .filter(|w| *w == dados.as_slice())
        .count()
}

#[test]
fn o_documento_para_antes_das_duas_listas() {
    let l = com_listas(true, true);
    let a = crate::screens::hero::board_view::area(&l);
    assert!(a.x >= 305.0, "a esquerda: {a:?}");
    assert!(a.x + a.w <= 1630.0, "a direita: {a:?}");
    assert!(a.w > 0.0);
}

#[test]
fn sem_lista_flutuante_o_documento_fica_como_antes() {
    let l = com_listas(false, false);
    let a = l.clear_of_side_lists(l.draw_area);
    assert_eq!(
        (a.x, a.w),
        (l.draw_area.x, l.draw_area.w),
        "PH2D_CARDS=0 não usa as listas"
    );
}

#[test]
fn lista_fechada_devolve_a_largura() {
    let mut l = com_listas(true, true);
    l.float_lists[1].h = 0.0;
    let a = l.clear_of_side_lists(l.draw_area);
    assert_eq!(a.x + a.w, l.draw_area.x + l.draw_area.w);
}

#[test]
fn o_chao_do_documento_cobre_ate_ao_fim_da_janela_so_em_cartoes() {
    let l = com_listas(true, true);
    let ponta = |cartao| {
        force_cromo_on_this_thread(Some(cartao));
        let mut s = VectorScene::new();
        crate::screens::hero::canvas::paint_document_ground(&l, &mut s, TEMA);
        force_cromo_on_this_thread(None);
        s
    };
    let chao = {
        force_cromo_on_this_thread(Some(true));
        let cor = Barra::de(TEMA).expect("em cartões").chao().with_alpha(1.0);
        force_cromo_on_this_thread(None);
        cor
    };
    assert_eq!(vezes_cor(&ponta(true), chao), 1);
    assert_eq!(vezes_cor(&ponta(false), chao), 0);
}

fn pinta_um_aviso(cartao: bool) -> VectorScene {
    force_cromo_on_this_thread(Some(cartao));
    let mut q = ToastQueue::new();
    q.push(Toast::success("Added Macro"));
    let (mut scene, mut text) = (VectorScene::new(), TextSystem::without_system_fonts());
    let mut ctx = PaintCtx {
        theme: TEMA,
        viewport: janela(),
        text: &mut text,
    };
    crate::paint::Paint::paint(&q, &mut scene, &mut ctx);
    force_cromo_on_this_thread(None);
    scene
}

#[test]
fn o_aviso_veste_a_lista_e_perde_a_faixa_de_cor() {
    let verde = resolve(ColorToken::Success, TEMA);
    let velho = resolve(ColorToken::BgElev, TEMA);
    let novo = CardInk::of_list(TEMA)
        .color(ColorToken::KanbanCardRaised)
        .with_alpha(1.0);
    let barra = CardInk::of_list(TEMA)
        .color(ColorToken::KanbanList)
        .with_alpha(1.0);
    assert_ne!(
        novo, barra,
        "a 1.ª linha do aviso cai sobre a barra de cima"
    );
    let (c, a) = (pinta_um_aviso(true), pinta_um_aviso(false));
    assert_eq!(vezes_cor(&c, novo), 1, "corpo levantado");
    assert_eq!(vezes_cor(&c, barra), 0, "nunca no tom da barra");
    assert_eq!(vezes_cor(&c, velho), 0, "sem o corpo de antes");
    assert!(vezes_cor(&a, velho) >= 1, "PH2D_CARDS=0: o corpo de antes");
    assert!(
        vezes_cor(&a, verde) > vezes_cor(&c, verde),
        "a faixa só no modo de antes"
    );
}

#[test]
fn o_botao_do_painel_tem_o_tom_do_cartao_e_o_de_antes_fora() {
    let ink = CardInk::of_list(TEMA);
    let r = Rect::new(0.0, 0.0, 100.0, 28.0);
    let pinta = |cartao, on, hover| {
        force_cromo_on_this_thread(Some(cartao));
        let mut s = VectorScene::new();
        let fg = Superficie::de(TEMA).botao(&mut s, r, on, hover);
        force_cromo_on_this_thread(None);
        (s, fg)
    };
    assert_eq!(
        vezes_cor(
            &pinta(true, false, false).0,
            ink.color(ColorToken::CardField)
        ),
        1
    );
    assert_eq!(
        vezes_cor(
            &pinta(true, false, true).0,
            ink.color(ColorToken::CardFieldHover)
        ),
        1
    );
    assert_eq!(
        vezes_cor(
            &pinta(true, true, false).0,
            ink.color(ColorToken::KanbanBlueBold)
        ),
        1
    );
    assert_eq!(
        vezes_cor(
            &pinta(false, false, false).0,
            resolve(ColorToken::Bg2, TEMA)
        ),
        1
    );
}

#[test]
fn o_painel_do_documento_e_a_lista_de_cartoes() {
    let r = Rect::new(0.0, 0.0, 240.0, 200.0);
    let pinta = |cartao| {
        force_cromo_on_this_thread(Some(cartao));
        let mut s = VectorScene::new();
        Superficie::de(TEMA).moldura_de_painel(&mut s, r);
        force_cromo_on_this_thread(None);
        s
    };
    let lista = CardInk::of_list(TEMA)
        .color(ColorToken::KanbanList)
        .with_alpha(1.0);
    assert_eq!(vezes_cor(&pinta(true), lista), 1);
    assert_eq!(
        vezes_cor(&pinta(false), resolve(ColorToken::BgElev, TEMA)),
        1
    );
}

#[test]
fn a_barra_de_trabalho_veste_a_mesma_superficie_do_aviso() {
    let pinta = |cartao| {
        force_cromo_on_this_thread(Some(cartao));
        let mut q = crate::progress::JobQueue::new();
        q.push(crate::progress::Progress::new("Working"));
        let (mut scene, mut text) = (VectorScene::new(), TextSystem::without_system_fonts());
        let mut ctx = PaintCtx {
            theme: TEMA,
            viewport: janela(),
            text: &mut text,
        };
        q.paint_below(0, &mut scene, &mut ctx);
        force_cromo_on_this_thread(None);
        scene
    };
    let novo = CardInk::of_list(TEMA)
        .color(ColorToken::KanbanCardRaised)
        .with_alpha(1.0);
    assert_eq!(vezes_cor(&pinta(true), novo), 1);
    assert_eq!(
        vezes_cor(&pinta(true), resolve(ColorToken::BgElev, TEMA)),
        0
    );
    assert!(vezes_cor(&pinta(false), resolve(ColorToken::BgElev, TEMA)) >= 1);
}

/// Um quadro de verdade, pelo pintor do ecrã. Dois quadros: o 2.º lê o que o 1.º mediu.
fn pinta_um_quadro_com_layout(cartao: bool, live: bool) -> (VectorScene, HeroLayout) {
    use crate::screens::hero::HeroScreen;
    force_cromo_on_this_thread(Some(cartao));
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.theme = TEMA;
    let id = hero.documents.create("Board".into());
    hero.documents.activate(Some(id));
    hero.documents.live_view.panel.open = live;
    let (mut scene, mut text) = (VectorScene::new(), TextSystem::without_system_fonts());
    for _ in 0..2 {
        scene = VectorScene::new();
        crate::screens::hero::paint::paint_hero_screen(&mut hero, janela(), &mut scene, &mut text);
    }
    force_cromo_on_this_thread(None);
    (scene, hero.last_layout.expect("pintou"))
}

fn pinta_um_quadro(cartao: bool, live: bool) -> VectorScene {
    pinta_um_quadro_com_layout(cartao, live).0
}

/// O quadro do ecrã traz o rect do chão: a cor sozinha não prova, porque outros pintores a usam.
#[test]
fn abrir_um_documento_pinta_o_chao_por_cima_da_cena() {
    let (scene, layout) = pinta_um_quadro_com_layout(true, false);
    let mut so_chao = VectorScene::new();
    force_cromo_on_this_thread(Some(true));
    crate::screens::hero::canvas::paint_document_ground(&layout, &mut so_chao, TEMA);
    force_cromo_on_this_thread(None);
    let rect = so_chao.inner().encoding().path_data.clone();
    assert!(!rect.is_empty());
    let achados = scene
        .inner()
        .encoding()
        .path_data
        .windows(rect.len())
        .filter(|w| *w == rect.as_slice())
        .count();
    assert_eq!(achados, 1, "o rect do chão no quadro do ecrã");
}

#[test]
fn o_painel_live_e_os_botoes_dele_vestem_o_cartao() {
    let ink = CardInk::of_list(TEMA);
    let lista = ink.color(ColorToken::KanbanList).with_alpha(1.0);
    let campo = ink.color(ColorToken::CardField);
    let (aberto, fechado) = (pinta_um_quadro(true, true), pinta_um_quadro(true, false));
    assert!(
        vezes_cor(&aberto, lista) > vezes_cor(&fechado, lista),
        "a moldura do painel"
    );
    assert!(
        vezes_cor(&aberto, campo) > vezes_cor(&fechado, campo),
        "os botões do painel"
    );
    let antes = pinta_um_quadro(false, true);
    assert!(
        vezes_cor(&antes, resolve(ColorToken::Bg2, TEMA)) >= 1,
        "PH2D_CARDS=0: os de antes"
    );
}

/// «Remove Animation» sem o «x» em letra, e o verbo do lado com o «+» do idioma.
#[test]
fn remover_animacao_nao_leva_o_x_em_letra() {
    let t = ph2d_i18n::tr("panel.inspector.animation.x_remove_animation");
    assert_eq!(t, "Remove Animation");
}
