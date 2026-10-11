//! ⭐ **A cena 16** (`PH2D_OBJECT_MODE_SMOKE=16`, F10 — 11/10): A ESTRELA DO AFFINITY E O CARTÃO
//! LATERAL. No layout *Vetor*, uma estrela desenhada com a ferramenta de forma na mão (nasce
//! REGULAR); o botão *Forma*; e, pelo MESMO press e arrasto da shell (`shape_handles_live::on`), os
//! pontos junto da ponta de cima: a ponta arredonda (o ponto desce pelo eixo dela), o vale da
//! esquerda arredonda os vales, e o ⇕ da borda da caixa sobe dois degraus — 7 pontas. A foto final
//! mostra a estrela macia com os pontos verdes e, ao lado, o cartão (Corner · Smooth · Twist).

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
/// Forma pelo botão · `82` = as pontas arredondam (o ponto desce pelo eixo da ponta) · `83` = os
/// vales arredondam · `84` = o ⇕ sobe dois degraus: 7 pontas · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let (c, half) = ESTRELA;
    // A mão anda do ponto `a` na direcção `dir` (mundo), em seis passos até `len`.
    let walk = move |dir: [f64; 2], len: f64| {
        move |a: [f64; 2]| -> Vec<[f64; 2]> {
            (1..=6)
                .map(|k| {
                    let t = len * f64::from(k) / 6.0;
                    [a[0] + dir[0] * t, a[1] + dir[1] * t]
                })
                .collect()
        }
    };
    match stage {
        80 if vec.entities.is_empty() => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, c, half);
            Some(81)
        }
        81 if vec.entities.len() == 1 => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(82)
        }
        // A ponta de cima: o ponto desce 0,25 m pelo eixo dela — as pontas arredondam, macias.
        82 if vec.draw_config.mode == ph2d_tool_vector::DrawMode::Shape => {
            pull(vec, scene, sim, handle_of(2)?, walk([0.0, -1.0], 0.25)).then_some(83)
        }
        // O vale da esquerda: o ponto sobe 0,04 m pelo eixo dele — os vales arredondam de leve.
        83 => {
            let dir = [-(54f64.to_radians().cos()), 54f64.to_radians().sin()];
            pull(vec, scene, sim, handle_of(3)?, walk(dir, 0.04)).then_some(84)
        }
        // O ⇕ das pontas: dois degraus para cima (1/24 da altura cada) — 7 pontas.
        84 => pull(
            vec,
            scene,
            sim,
            handle_of(0)?,
            walk([0.0, 1.0], 2.0 * 2.0 * half / 24.0),
        )
        .then_some(9),
        _ => None,
    }
}
