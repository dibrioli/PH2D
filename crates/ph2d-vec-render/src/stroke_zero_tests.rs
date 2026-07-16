//! **Largura zero não desenha traço nenhum** (Enio, 2026-07-16).
//!
//! O slider de Width passou a chegar a `0` — e `0` tem de significar **sem traço**, não "o traço
//! mais fino que der". Um renderer que trate `0` como hairline (vários tratam) faria o slider
//! prometer uma coisa e entregar outra, justo na ponta onde o artista olha.
//!
//! # O oráculo é o que foi ENCODADO, não o que o código diz
//!
//! `VectorScene::inner().encoding().n_paths` conta os caminhos que de fato entraram na cena do
//! Vello. É a aparência, medida — não um espelho da regra do conserto: um gate que perguntasse
//! `s.width > 0.0` estaria a testar o `if` que eu acabei de escrever.

use crate::draw_path;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPath, cook};
// `Affine`/`VectorScene` pelas re-exports de `ph2d-vector`: esta crate NÃO importa vello
// direto (é o que a mantém gate-proof p/ o `vello_kurbo_only_in_ph2d_vector`).
use ph2d_vector::{Affine, VectorScene};

fn square() -> VecPath {
    cook(ShapeKind::Rectangle, [-1.0, -1.0], [1.0, 1.0], &[])
}

/// Quantos caminhos o desenho de `path` encoda.
fn encoded(path: &VecPath) -> u32 {
    let mut scene = VectorScene::new();
    draw_path(path, Affine::IDENTITY, &mut scene);
    scene.inner().encoding().n_paths
}

fn with_width(w: f64) -> VecPath {
    let mut p = square();
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), w));
    p
}

/// **Traço de largura ZERO encoda exatamente o mesmo que traço NENHUM.**
///
/// O par com o gate abaixo é o que dá sentido a este: sozinho, ele ficaria VERDE num renderer que
/// não desenhasse traço nenhum, nunca. [[feedback_absence_gate_needs_a_presence_sibling]]
#[test]
fn a_zero_width_stroke_draws_nothing() {
    let mut none = square();
    none.stroke = None;

    assert_eq!(
        encoded(&with_width(0.0)),
        encoded(&none),
        "um traço de largura 0 encodou caminho a mais que um path SEM traço — o `0` do slider tem \
         de ser 'sem traço', e não 'a hairline mais fina que o renderer conseguir'"
    );
}

/// **E um traço de largura de verdade DESENHA** — o irmão de presença.
#[test]
fn a_real_stroke_still_draws() {
    let mut none = square();
    none.stroke = None;

    assert!(
        encoded(&with_width(3.0)) > encoded(&none),
        "um traço de 3px não encodou nada — o gate irmão (largura 0 não desenha) ficaria verde \
         por não haver traço nenhum a desenhar, e não por causa do zero"
    );
}

/// **O zero é REVERSÍVEL: ele não esquece a cor.**
///
/// É a diferença entre o zero e a swatch None, e é o motivo de o zero ser um `StrokeSpec` de
/// largura 0 em vez de um `stroke: None`. O artista arrasta até o fim, vê o traço sumir, muda de
/// ideia e arrasta de volta — e o traço que volta é o DELE. Fosse `None`, a cor teria ido embora e
/// voltaria preta.
#[test]
fn the_zero_does_not_forget_the_colour() {
    let red = Rgba8::new(200, 30, 30, 255);
    let mut p = square();
    p.stroke = Some(StrokeSpec::new(red, 0.0));

    // O documento guarda a cor mesmo com o traço invisível...
    assert_eq!(p.stroke.expect("stroke").color, red);
    // ...e ela volta intacta quando a largura volta.
    p.stroke = p.stroke.map(|s| StrokeSpec { width: 4.0, ..s });
    assert_eq!(p.stroke.expect("stroke").color, red);
    let mut none = square();
    none.stroke = None;
    assert!(
        encoded(&p) > encoded(&none),
        "a largura voltou e o traço não voltou com ela"
    );
}
