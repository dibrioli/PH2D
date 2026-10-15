//! ⭐ **A cena 22** (`PH2D_OBJECT_MODE_SMOKE=22`, dono 15/10): O ARRANGE E O ALINHAR NA ④. No layout
//! *Vetor*, três formas espalhadas e desalinhadas, o *Select* escolhido pelo MESMO clique que o
//! botão dele na coluna ① dá, e as três selecionadas — a foto mostra a ④ com as DUAS linhas: o
//! Arrange inteiro na de cima, o Alinhar e o Distribuir na de baixo; e o painel sem as secções.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{ShapeKind, VecPathId};

/// As três formas: o tipo, o centro (m) e a meia-largura (m) — alturas e tamanhos diferentes, para
/// que cada botão do Alinhar mexa em todas.
const FORMAS: [(ShapeKind, [f64; 2], f64); 3] = [
    (ShapeKind::Rectangle, [-1.6, 0.5], 0.5),
    (ShapeKind::Ellipse, [0.0, -0.4], 0.35),
    (ShapeKind::Star, [1.5, 0.2], 0.6),
];

/// O primeiro estágio (uma forma por estágio, o clique no Select, a seleção das três).
pub(crate) const FIRST: u8 = 220;
const SELECT: u8 = FIRST + FORMAS.len() as u8;
pub(crate) const LAST: u8 = SELECT + 1;

/// Um passo da cena 22; devolve o próximo estágio, se mudou. Cada forma espera a anterior virar
/// objecto; a troca de ferramenta reduz a seleção, por isso as três são selecionadas DEPOIS dela.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        s if (FIRST..SELECT).contains(&s) && feitas == usize::from(s - FIRST) => {
            let (kind, at, half) = FORMAS[usize::from(s - FIRST)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, half);
            Some(s + 1)
        }
        SELECT if feitas == FORMAS.len() => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(LAST)
        }
        LAST if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let all: Vec<VecPathId> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&all);
            Some(9)
        }
        _ => None,
    }
}
