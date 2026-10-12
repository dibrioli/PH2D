//! ⭐ **A cena 20** (`PH2D_OBJECT_MODE_SMOKE=20`, F13 — 12/10): O MODO FORMA SELECCIONA. No layout
//! *Vetor*, uma estrela e uma elipse; o botão *Forma*; a paleta ARMA um polígono (as alças apagam-se);
//! um clique no meio da estrela selecciona-a e desarma — as alças verdes dela voltam, nenhum nó; a
//! paleta arma de novo e um clique DENTRO da elipse (fora do centro) passa a selecção a ela.
//! `PH2D_CENA20_FICA=0` pára depois do 1.º clique (a foto da estrela).

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_scene::ShapeKind;

/// As duas formas: o tipo e o centro (m); a meia-largura é [`MEIA`].
pub(crate) const FORMAS: [(ShapeKind, [f64; 2]); 2] = [
    (ShapeKind::Star, [-1.5, 0.0]),
    (ShapeKind::Ellipse, [1.5, 0.0]),
];

/// A meia-largura de cada forma (m).
pub(crate) const MEIA: f64 = 1.0;

/// A forma que a paleta arma (a terceira, que nunca chega a nascer).
pub(crate) const ARMADA: ShapeKind = ShapeKind::Polygon;

/// O 2.º clique: dentro da elipse, longe do centro e do contorno (m).
pub(crate) const DENTRO: [f64; 2] = [1.5 + 0.4, -0.3];

/// Metros por pixel da vista de omissão — o mesmo das outras cenas.
const PX: f64 = 0.01;

/// Um CLIQUE da ferramenta de forma em `at` (press e soltar no mesmo ponto) e o que a shell faz no Up
/// (`despacho_clique_vetor_solto`).
pub(crate) fn clique(vec: &mut VecState, scene: &mut ph2d_vec_scene::VecScene, at: [f64; 2]) {
    let c = ph2d_vec_edit::ShapeConstraint::default();
    vec.shape
        .on_press(scene, ARMADA, ARMADA.defaults(), at, PX, c);
    let committed = vec.shape.on_release(scene);
    crate::shape_live::after_the_shape_release(vec, scene, committed);
}

/// O clique no botão da paleta de [`ARMADA`] — o mesmo evento que o dedo do artista dá.
fn arma(hero: &mut HeroScreen) {
    if let Some(i) = ph2d_tool_vector::shapes::SHAPES
        .iter()
        .position(|s| s.kind == ARMADA)
    {
        hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::vector_shape_id(i)));
    }
}

/// Um passo da cena 20; devolve o próximo estágio, se mudou. `130..=131` = uma forma por estágio ·
/// `132` = o modo Forma pelo botão · `133` = a paleta arma · `134` = o clique na estrela · `135` = a
/// paleta arma de novo · `136` = o clique dentro da elipse · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    let nth = |vec: &VecState, k: usize| vec.entities.keys().nth(k).copied();
    match stage {
        s @ 130..=131 if feitas == usize::from(s - 130) => {
            let (kind, at) = FORMAS[usize::from(s - 130)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, MEIA);
            Some(s + 1)
        }
        132 if feitas == FORMAS.len() => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(133)
        }
        133 if vec.draw_config.mode == DrawMode::Shape => {
            arma(hero);
            Some(134)
        }
        134 if vec.shape_armed => {
            clique(vec, scene, FORMAS[0].1);
            Some(135)
        }
        135 if !vec.shape_armed && vec.pen.selected() == nth(vec, 0) => {
            if std::env::var("PH2D_CENA20_FICA").as_deref() == Ok("0") {
                return Some(9);
            }
            arma(hero);
            Some(136)
        }
        136 if vec.shape_armed => {
            clique(vec, scene, DENTRO);
            Some(9)
        }
        _ => None,
    }
}
