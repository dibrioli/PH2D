//! **As leis dos cinco seletores de cor** (o menu ≡ da paleta do Aseprite 1.3.18, observado no
//! Trial): cantos, centro, a rotação das rodas e a ida-e-volta clique → cor → marcador.

use ph2d_editor_core::pixel_panel::PixelSelector as S;
use ph2d_panel_pixel_tools::seletor::{NORMAL_FUNDO, Params, cor, na_barra, no_quadro, ponto};

const P: Params = Params { a: 0.0, b: 1.0, c: 1.0 };

#[test]
fn tint_shade_tone_is_saturation_across_and_value_down() {
    assert_eq!(cor(S::TintShadeTone, no_quadro(S::TintShadeTone, P, 0.0, 0.0)), [255, 255, 255]);
    assert_eq!(cor(S::TintShadeTone, no_quadro(S::TintShadeTone, P, 1.0, 0.0)), [255, 0, 0]);
    assert_eq!(cor(S::TintShadeTone, no_quadro(S::TintShadeTone, P, 1.0, 1.0)), [0, 0, 0]);
    let verde = na_barra(S::TintShadeTone, P, 1.0 / 3.0);
    assert_eq!(cor(S::TintShadeTone, verde), [0, 255, 0], "a barra é a matiz");
}

#[test]
fn spectrum_is_hue_across_white_top_black_bottom() {
    let topo = no_quadro(S::Spectrum, P, 0.3, 0.0);
    assert_eq!(cor(S::Spectrum, topo), [255, 255, 255]);
    assert_eq!(cor(S::Spectrum, no_quadro(S::Spectrum, P, 0.3, 1.0)), [0, 0, 0]);
    assert_eq!(cor(S::Spectrum, no_quadro(S::Spectrum, P, 0.0, 0.5)), [255, 0, 0], "meio = cor pura");
}

#[test]
fn the_rgb_wheel_has_red_at_minus_30_degrees_green_on_top_and_grey_in_the_centre() {
    let ang = |g: f64| {
        let r = g.to_radians();
        (0.5 + 0.5 * r.cos(), 0.5 - 0.5 * r.sin())
    };
    let (u, v) = ang(-30.0);
    assert_eq!(cor(S::RgbWheel, no_quadro(S::RgbWheel, P, u, v)), [255, 0, 0]);
    let (u, v) = ang(90.0);
    assert_eq!(cor(S::RgbWheel, no_quadro(S::RgbWheel, P, u, v)), [0, 255, 0]);
    let c = cor(S::RgbWheel, no_quadro(S::RgbWheel, P, 0.5, 0.5));
    assert!(c[0] == c[1] && c[1] == c[2], "o centro é cinza: {c:?}");
    let escuro = na_barra(S::RgbWheel, P, 0.0);
    assert_eq!(cor(S::RgbWheel, escuro), [0, 0, 0], "a barra é o brilho");
}

#[test]
fn the_ryb_wheel_puts_yellow_where_the_rgb_wheel_puts_green() {
    let c = cor(S::RybWheel, no_quadro(S::RybWheel, P, 0.5, 0.0));
    assert!(c[0] > 200 && c[1] > 200 && c[2] < 60, "em cima é amarelo: {c:?}");
}

#[test]
fn the_normal_map_wheel_encodes_the_direction() {
    assert_eq!(cor(S::NormalMap, no_quadro(S::NormalMap, P, 0.5, 0.5)), NORMAL_FUNDO, "centro = a normal de frente");
    let dir = cor(S::NormalMap, no_quadro(S::NormalMap, P, 1.0, 0.5));
    assert!(dir[0] == 255 && dir[1] == 128, "direita = +x: {dir:?}");
    let cima = cor(S::NormalMap, no_quadro(S::NormalMap, P, 0.5, 0.0));
    assert!(cima[1] == 255 && cima[0] == 128, "cima = +y: {cima:?}");
}

#[test]
fn the_marker_falls_where_the_click_was() {
    for m in [S::TintShadeTone, S::Spectrum, S::RgbWheel, S::RybWheel, S::NormalMap] {
        for (u, v) in [(0.3, 0.4), (0.7, 0.2), (0.55, 0.6)] {
            let p = no_quadro(m, P, u, v);
            let (pu, pv) = ponto(m, p);
            assert!((pu - u).abs() < 1e-9 && (pv - v).abs() < 1e-9, "{m:?} ({u},{v}) → ({pu},{pv})");
        }
    }
}

#[test]
fn the_wheel_edges_are_smooth_not_stairs() {
    use ph2d_panel_pixel_tools::seletor::imagem_do_quadro;
    // smoke do dono 25/10: «a roda de cor ficou com bordas pixeladas»
    let lado = 256;
    let px = imagem_do_quadro(S::RgbWheel, P, lado);
    let meios = px.chunks_exact(4).filter(|p| p[3] > 0 && p[3] < 255).count();
    assert!(meios > 200, "a borda da roda RGB tem de ter alfa intermédio: {meios}");
    let nm = imagem_do_quadro(S::NormalMap, P, lado);
    let linha = (lado / 2) as usize;
    let fundo = NORMAL_FUNDO;
    let borda: Vec<[u8; 3]> = (0..lado as usize)
        .map(|x| {
            let i = (linha * lado as usize + x) * 4;
            [nm[i], nm[i + 1], nm[i + 2]]
        })
        .collect();
    let primeiro = borda.iter().position(|c| *c != fundo).expect("o disco");
    let c = borda[primeiro];
    let cheio = cor(S::NormalMap, no_quadro(S::NormalMap, P, 0.0, 0.5));
    assert!(c != cheio, "o primeiro píxel do disco mistura com o fundo: {c:?}");
}
