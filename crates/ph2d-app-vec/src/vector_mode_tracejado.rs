//! ⭐ **A cena 24** (`PH2D_OBJECT_MODE_SMOKE=24`, F16c — dono 15/10): O TRACEJADO EM PARES E A SETA
//! POR PONTA. No layout *Vetor*, um rectângulo com o «traço-ponto-ponto» (os três pares: `4 · 1,5`,
//! `0 · 1,5`, `0 · 1,5`, ponta redonda — um traço de comprimento `0` é um ponto) e uma linha com dois
//! pares (`3 · 1`, `1 · 1`) e uma seta pequena no começo (`0,7`) e grande no fim (`2`). A linha fica
//! selecionada e o «Stroke ▾» aberto: as linhas *Dash* / *Gap* mostram os pares dela, e o *Head Size*
//! as duas pontas.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{DashPattern, LineCap, Marker, ShapeKind};

/// As duas formas: o tipo, o centro (m) e a meia-largura (m). A linha é a selecionada.
const FORMAS: [(ShapeKind, [f64; 2], f64); 2] = [
    (ShapeKind::Rectangle, [-1.3, 0.0], 0.8),
    (ShapeKind::Line, [1.3, 0.0], 0.8),
];

/// A largura dos dois traços (m) — o tracejado é em múltiplos dela.
const LARGURA: f64 = 0.05;

/// O primeiro estágio (uma forma por estágio, o estilo, o Select, a seleção, o «Stroke ▾»).
pub(crate) const FIRST: u8 = 240;
const ESTILO: u8 = FIRST + FORMAS.len() as u8;
const SELECT: u8 = ESTILO + 1;
const PICK: u8 = SELECT + 1;
pub(crate) const LAST: u8 = PICK + 1;

/// O traço autorado de cada forma, pela ordem de [`FORMAS`].
fn estilo(i: usize, s: &mut ph2d_vec_scene::StrokeSpec) {
    s.width = LARGURA;
    s.cap = LineCap::Round;
    if i == 0 {
        s.dash = Some(DashPattern {
            pairs: [[4.0, 1.5], [0.0, 1.5], [0.0, 1.5]],
        });
    } else {
        s.dash = Some(DashPattern {
            pairs: [[3.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        });
        (s.marker_start, s.marker_end) = (Marker::Triangle, Marker::Triangle);
        (s.marker_scale_start, s.marker_scale_end) = (0.7, 2.0);
    }
}

/// Um passo da cena 24; devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        s if (FIRST..ESTILO).contains(&s) && feitas == usize::from(s - FIRST) => {
            let (kind, at, half) = FORMAS[usize::from(s - FIRST)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, half);
            Some(s + 1)
        }
        ESTILO if feitas == FORMAS.len() => {
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            for (i, id) in ids.into_iter().enumerate() {
                if let Some(s) = scene.path_mut(id).and_then(|p| p.stroke.as_mut()) {
                    estilo(i, s);
                }
            }
            Some(SELECT)
        }
        SELECT => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(PICK)
        }
        // A troca de ferramenta reduz a seleção: a linha (a mais nova) é escolhida DEPOIS dela.
        PICK if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let linha = vec.entities.keys().last().copied()?;
            vec.pen.select_many(&[linha]);
            Some(LAST)
        }
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

#[cfg(test)]
#[path = "vector_mode_tracejado_tests.rs"]
mod tests;
