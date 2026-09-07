//! Os gates do enquadramento da receita — [`super::framed`].
//!
//! ⚠️ **A lei é pura de propósito**: o quadro que a chama tem uma `HeroScreen` com store, uma
//! surface de janela real e trinta e cinco argumentos. *Uma lei alcançável só pelo produto é uma
//! lei sem gate.*

use super::{FILL, framed};
use ph2d_editor::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

const WINDOW: WindowSize = WindowSize::new(1600, 900);

/// A janela com uma coluna docada de `400 px` à esquerda — a área visível é o resto.
///
/// ⚠️ **É a fixtura do REPORT**: com as duas colunas iguais o defeito seria invisível (o centro da
/// área e o da janela coincidem), e o gate ficaria verde sobre o que o Enio viu.
const AREA: Rect = Rect {
    x: 400.0,
    y: 0.0,
    w: 1200.0,
    h: 900.0,
};

/// Onde uma caixa de mundo cai na tela, com esta câmera.
fn centre_px(cam: &Camera2d, min: [f32; 2], max: [f32; 2]) -> (f32, f32) {
    cam.world_to_screen([(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5], WINDOW)
}

/// ⭐⭐⭐ **O REPORT: a receita cai no centro da ÁREA VISÍVEL, não no da janela.**
///
/// Com uma coluna docada de 400 px, os dois centros estão a **200 px** um do outro — e a diferença
/// é exactamente a metade da coluna, que é o que põe a receita debaixo dela.
///
/// **Mutação que deve sangrar:** passar a janela em vez da área (`framed(.., janela, ..)` no
/// chamador, ou o `usable` a devolver sempre o viewport).
#[test]
fn the_recipe_lands_at_the_centre_of_the_visible_area_not_the_window() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    // Uma receita pequena, longe da origem — a vista tem de a ir buscar.
    let (min, max) = ([20.0, -8.0], [21.0, -7.0]);
    let out = framed(cam, WINDOW, AREA, min, max);
    let (px, py) = centre_px(&out, min, max);
    let want = (AREA.x + AREA.w * 0.5, AREA.y + AREA.h * 0.5);
    assert!(
        (px - want.0).abs() < 0.5 && (py - want.1).abs() < 0.5,
        "a receita caiu em ({px:.1}, {py:.1}) e o centro da area visivel e' \
         ({:.1}, {:.1}) — a janela teria dito ({:.1}, {:.1})",
        want.0,
        want.1,
        WINDOW.width as f32 * 0.5,
        WINDOW.height as f32 * 0.5,
    );
}

/// ⭐⭐ **Uma receita que já cabe não mexe no zoom** — abrir uma receita não é um gesto de zoom.
///
/// **Mutação que deve sangrar:** o `if need > out.height_world` virar uma atribuição incondicional
/// (a vista passaria a APROXIMAR-se numa receita pequena, mudando a escala de trabalho).
#[test]
fn a_recipe_that_already_fits_leaves_the_zoom_alone() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let out = framed(cam, WINDOW, AREA, [0.0, 0.0], [0.5, 0.5]);
    assert!(
        (out.height_world - cam.height_world).abs() < 1e-6,
        "o zoom mexeu-se numa receita que ja' cabia: {} -> {}",
        cam.height_world,
        out.height_world
    );
}

/// ⭐⭐⭐ **Uma receita maior que a área faz a vista AFASTAR até ela caber** — com folga, e a folga
/// é a [`FILL`].
///
/// ⚠️ **A régua é em PIXELS**, e não em `height_world`: é a tela que tem de conter a caixa, e o
/// caminho de mundo→tela passa pelo aspecto da janela — uma barra escrita em metros deixaria o
/// caso largo (a receita comprida numa área estreita) passar.
///
/// **Mutação que deve sangrar:** apagar o braço da largura (`need_w`), que é justamente o que uma
/// coluna docada torna dominante.
#[test]
fn a_recipe_bigger_than_the_area_pulls_the_view_back_until_it_fits() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    // Larga e baixa: só o braço da LARGURA a acusa, e a área é estreita por causa da coluna.
    let (min, max) = ([-30.0, -1.0], [30.0, 1.0]);
    let out = framed(cam, WINDOW, AREA, min, max);
    let a = out.world_to_screen(min, WINDOW);
    let b = out.world_to_screen(max, WINDOW);
    let w_px = (b.0 - a.0).abs();
    let h_px = (b.1 - a.1).abs();
    assert!(
        w_px <= AREA.w * FILL + 0.5 && h_px <= AREA.h * FILL + 0.5,
        "a receita ocupa {w_px:.1} x {h_px:.1} px numa area de {} x {} — o teto e' {FILL} dela",
        AREA.w,
        AREA.h
    );
    // E continua centrada: afastar sem centrar seria meio trabalho.
    let (px, _) = centre_px(&out, min, max);
    assert!(
        (px - (AREA.x + AREA.w * 0.5)).abs() < 0.5,
        "afastou e descentrou: {px:.1}"
    );
}

/// ⛔ **Uma área degenerada não produz uma câmera `NaN`.**
///
/// O primeiro quadro publica `0 x 0` (o painel ainda não pintou), e uma divisão por zero ali daria
/// uma câmera que nenhum gesto do artista recupera — a tela fica preta para sempre.
#[test]
fn an_area_that_was_not_published_yet_falls_back_to_the_window() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let out = framed(
        cam,
        WINDOW,
        Rect::new(0.0, 0.0, 0.0, 0.0),
        [4.0, 4.0],
        [5.0, 5.0],
    );
    assert!(
        out.center[0].is_finite() && out.center[1].is_finite() && out.height_world.is_finite(),
        "camera nao-finita: {:?}",
        out
    );
    let (px, py) = centre_px(&out, [4.0, 4.0], [5.0, 5.0]);
    assert!(
        (px - 800.0).abs() < 0.5 && (py - 450.0).abs() < 0.5,
        "sem area publicada o centro tem de ser o da JANELA: ({px:.1}, {py:.1})"
    );
}
