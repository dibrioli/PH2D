//! ⛔⛔⛔ **O CHÃO pinta À VOLTA da área, nunca por baixo dela — e a wave 31 falhou por não saber isso.**
//!
//! Report do dono: *«parece que nada mudou»*, sobre uma wave que trocou o fundo da janela.
//!
//! # A causa, e ela estava escrita no ficheiro ao lado
//!
//! O [`paint_canvas_bg`] **só corre em modo FIXTURA**. Em modo vivo — que é sempre, no produto — o
//! compositor mostra o `game_rt` por baixo de onde o vello tem `α = 0`, e o `HeroScreen` **salta**
//! aquele pintor. O doc do `shells/desktop/src/canvas_clear.rs` di-lo em três linhas:
//!
//! > *«Quem procurar a cor do fundo no painter vai encontrar código que o produto não corre.»*
//!
//! *Sétima vez nesta jornada que a resposta já estava escrita e eu não a fui ler* — e desta vez o
//! aviso era literalmente sobre a armadilha em que caí.
//!
//! # A lei que fica
//!
//! O chão **não pode ser um fill por baixo da área**: isso taparia o desenho, que é a razão de
//! aquele pintor ser saltado. Ele é a janela **menos** a área, recortada com regra PAR-ÍMPAR — e
//! dentro do furo o vello não pinta nada, deixando o compositor mostrar o conteúdo. *É o furo que
//! faz da área um cartão em vez de um buraco.*

use ph2d_editor_core::screens::hero::paint_window_ground;
use ph2d_editor_core::screens::layout::{
    CenterSplit, ChromeBands, DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn layout() -> HeroLayout {
    HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        false,
        ChromeBands::DEFAULT,
        CenterSplit::None,
        DockSides::BOTH,
    )
}

/// ⭐ **Ele pinta** — e num tema moderno, onde o chão difere do painel.
#[test]
fn the_ground_paints_something() {
    let l = layout();
    let mut scene = VectorScene::new();
    paint_window_ground(&l, &mut scene, Theme::Dark);
    assert!(
        scene.inner().encoding().n_path_segments > 0,
        "o chao nao pintou nada: as areas nao te^m sobre o que assentar, e a divisoria de 4 px \
         volta a mostrar a cor do canvas"
    );
}

/// ⛔⛔ **E o furo existe: a geometria emitida NÃO é um simples rectângulo.**
///
/// ⚠️ A régua é a contagem de segmentos: um rectângulo só emite os lados dele; a janela **com um
/// furo arredondado** emite os dois contornos. Se um dia alguém trocar o recorte por um fill
/// simples, o desenho desaparece atrás do chão — e é isso que esta contagem apanha.
#[test]
fn the_ground_has_a_hole_and_not_just_a_rectangle() {
    let l = layout();
    let mut com_furo = VectorScene::new();
    paint_window_ground(&l, &mut com_furo, Theme::Dark);

    let mut so_rect = VectorScene::new();
    so_rect.fill_rect(
        ph2d_editor_core::paint::rect_to_vello(l.viewport),
        ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::WindowGround, Theme::Dark),
    );

    assert!(
        com_furo.inner().encoding().n_path_segments > so_rect.inner().encoding().n_path_segments,
        "o chao emite tanta geometria como um rectangulo simples: o FURO desapareceu, e com ele o \
         desenho fica tapado — que e' exactamente porque o pintor do fundo e' saltado no produto"
    );
}

/// ⚠️ **Uma área vazia não pinta chão nenhum** — senão a janela ficava toda do tom do chão.
#[test]
fn an_empty_area_paints_no_ground() {
    let mut l = layout();
    l.draw_area = Rect::new(0.0, 0.0, 0.0, 0.0);
    let mut scene = VectorScene::new();
    paint_window_ground(&l, &mut scene, Theme::Dark);
    assert_eq!(scene.inner().encoding().n_path_segments, 0);
}
