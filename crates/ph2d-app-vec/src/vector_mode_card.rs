//! ⭐ **A cena 16** (`PH2D_OBJECT_MODE_SMOKE=16`, F10 — 11/10): OS PARÂMETROS NOVOS E O CARTÃO
//! LATERAL. No layout *Vetor*, uma estrela desenhada com a ferramenta de forma na mão; o botão
//! *Forma* (as alças vivem no modo Forma); e, pelo MESMO press e arrasto da shell
//! (`shape_handles_live::on`), três alças: as pontas arredondam, o vale TORCE (a torção, F10) e a
//! aresta ENVERGA (a envergadura, F10). Por fim o chip *Corner* do CARTÃO LATERAL é clicado duas
//! vezes: *Rounded → Straight → Concave* — as pontas mordem para dentro. A foto final mostra a
//! estrela com as alças verdes e, ao lado dela, o cartão com o *Corner* e o *Smooth*.

use crate::shape_handles_live::{Gate, on, view};
use crate::state::VecState;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::shape_handles::{Mods, handles_of};
use ph2d_vec_scene::ShapeKind;

/// A estrela: o centro (m) e a meia-largura (m) — grande, no meio da vista de omissão.
pub(crate) const ESTRELA: ([f64; 2], f64) = ([0.0, 0.0], 2.2);

/// Metros por pixel da vista de omissão — o mesmo das outras cenas.
const PX: f64 = 0.01;

/// A alça da estrela que move o campo `field`.
fn handle_of(field: usize) -> Option<usize> {
    handles_of(ShapeKind::Star)
        .iter()
        .position(|h| h.fields.first() == Some(&field))
}

/// Agarra a alça `handle` da estrela pelo press da shell e leva-a por `path(onde ela está)`.
fn pull(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    handle: usize,
    path: impl Fn([f64; 2]) -> Vec<[f64; 2]>,
) -> bool {
    let Some(&id) = vec.entities.keys().next() else {
        return false;
    };
    vec.pen.select_many(&[id]);
    let Some(at) = view(sim, &vec.entities, &[id], Gate::of(vec), PX, None, None)
        .and_then(|v| v.points.get(handle).copied())
    else {
        return false;
    };
    if !on::press(vec, sim, scene, at, PX, 0) {
        return false;
    }
    for p in path(at) {
        on::drag(vec, sim, scene, p, Mods::default(), PX);
    }
    vec.shape_handles.release()
}

/// Um passo da cena 16; devolve o próximo estágio, se mudou. `80` = a estrela · `81` = o modo
/// Forma pelo botão · `82` = as pontas arredondam · `83` = o vale torce · `84` = a aresta enverga ·
/// `85`/`86` = o *Corner* do cartão, duas vezes · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let (c, half) = ESTRELA;
    match stage {
        80 if vec.entities.is_empty() => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, c, half);
            Some(81)
        }
        81 if vec.entities.len() == 1 => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(82)
        }
        // A ponta: a alça do arredondamento anda para o centro — o raio da ponta cresce.
        82 if vec.draw_config.mode == ph2d_tool_vector::DrawMode::Shape => {
            pull(vec, scene, sim, handle_of(2)?, move |a| {
                (1..=6)
                    .map(|k| {
                        let t = 0.05 * f64::from(k);
                        [a[0] + (c[0] - a[0]) * t, a[1] + (c[1] - a[1]) * t]
                    })
                    .collect()
            })
            .then_some(83)
        }
        // O vale roda 12° à volta do centro, à mesma distância: a estrela torce.
        83 => pull(vec, scene, sim, handle_of(1)?, move |a| {
            let (r, a0) = (
                (a[0] - c[0]).hypot(a[1] - c[1]),
                (a[1] - c[1]).atan2(a[0] - c[0]),
            );
            (1..=6)
                .map(|k| {
                    let t = a0 + (2.0 * f64::from(k)).to_radians();
                    [c[0] + r * t.cos(), c[1] + r * t.sin()]
                })
                .collect()
        })
        .then_some(84),
        // O meio de uma aresta, para FORA: as arestas envergam.
        84 => pull(vec, scene, sim, handle_of(5)?, move |a| {
            (1..=6)
                .map(|k| {
                    let t = 1.0 + 0.03 * f64::from(k);
                    [c[0] + (a[0] - c[0]) * t, c[1] + (a[1] - c[1]) * t]
                })
                .collect()
        })
        .then_some(85),
        // O chip *Corner* do cartão (o MESMO id da fila ②): Rounded → Straight → Concave.
        s @ (85 | 86) => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::vector_shape_choice_id(4),
            ));
            Some(if s == 85 { 86 } else { 9 })
        }
        _ => None,
    }
}
