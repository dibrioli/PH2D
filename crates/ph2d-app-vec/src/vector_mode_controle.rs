//! ⭐ **A cena 23** (`PH2D_OBJECT_MODE_SMOKE=23`, F16 — dono 15/10): A BARRA DE CONTROLE. No layout
//! *Vetor*, uma estrela e um rectângulo, o *Select* escolhido pelo MESMO clique que o botão dele na
//! coluna ① dá, e a estrela selecionada — a fila ② de cima é a *Control bar* do Illustrator (o nome
//! da forma, as duas cores, «Stroke ▾», a largura, o perfil, a tinta e a opacidade, sem os botões
//! de antes), e o «Stroke ▾» aberto mostra o painel do traço por baixo dela.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::ShapeKind;

/// As duas formas: o tipo, o centro (m) e a meia-largura (m). A estrela é a selecionada.
const FORMAS: [(ShapeKind, [f64; 2], f64); 2] = [
    (ShapeKind::Rectangle, [-1.4, 0.0], 0.6),
    (ShapeKind::Star, [1.0, 0.0], 0.8),
];

/// O primeiro estágio (uma forma por estágio, o clique no Select, a seleção, o «Stroke ▾»).
pub(crate) const FIRST: u8 = 230;
const SELECT: u8 = FIRST + FORMAS.len() as u8;
const PICK: u8 = SELECT + 1;
pub(crate) const LAST: u8 = PICK + 1;

/// Um passo da cena 23; devolve o próximo estágio, se mudou.
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
            Some(PICK)
        }
        // A troca de ferramenta reduz a seleção: a estrela (a mais nova) é escolhida DEPOIS dela.
        PICK if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let estrela = vec.entities.keys().last().copied()?;
            vec.pen.select_many(&[estrela]);
            Some(LAST)
        }
        // O «Stroke ▾» abre-se pelo clique que a paleta re-toca — depois de o chip estar pintado, que
        // é onde o painel do traço se ancora.
        LAST if hero
            .store
            .tool_row_slot(ph2d_panel_vector::ids::VECTOR_BAR_STROKE)
            .is_some() =>
        {
            hero.apply_event(WidgetEvent::Click(
                ph2d_panel_vector::ids::VECTOR_BAR_STROKE,
            ));
            Some(9)
        }
        _ => None,
    }
}
