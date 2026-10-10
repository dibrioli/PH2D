//! ⭐ **A cena 14** (`PH2D_OBJECT_MODE_SMOKE=14`, F8 — 10/10): ENQUADRAR TUDO. Depois da cena 9
//! (layout *Vetor*, um rectângulo e uma elipse, a Caneta na mão), três formas nascem FORA da tela
//! (em cima à esquerda, em baixo à direita e em cima) e *Ver ▸ Enquadrar tudo* é clicado pela MESMA
//! porta de um clique real na linha do menu: a vista voa até tudo caber na área visível, com a
//! margem das barras flutuantes.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::ShapeKind;

/// As três formas de fora: o tipo, o centro (m) e a meia-largura (m). A câmera de omissão vê
/// `±5 m` na vertical — todas caem fora dela.
const FORA: [(ShapeKind, [f64; 2], f64); 3] = [
    (ShapeKind::Ellipse, [-11.0, 6.5], 0.8),
    (ShapeKind::Rectangle, [12.0, -6.0], 1.0),
    (ShapeKind::Ellipse, [3.0, 8.0], 0.5),
];

/// Um passo da cena 14; devolve o próximo estágio, se mudou. `60..=62` = uma forma de fora por
/// estágio (cada uma espera a anterior virar objecto) · `63` = o clique no menu · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        s @ 60..=62 if feitas == usize::from(s - 60) + 2 => {
            let (kind, at, half) = FORA[usize::from(s - 60)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, half);
            Some(s + 1)
        }
        63 if feitas == FORA.len() + 2 => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_editor_core::ids::MENUBAR_VIEW_FRAME_ALL,
            ));
            Some(9)
        }
        _ => None,
    }
}
