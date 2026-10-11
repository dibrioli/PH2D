//! ⭐ **A cena 15** (`PH2D_OBJECT_MODE_SMOKE=15`, F9 — 11/10): AS ALÇAS DAS FORMAS. No layout
//! *Vetor*, um rectângulo arredondado, uma estrela e uma pizza; a Seleção (a seta preta) pelo clique
//! do botão dela; e cada forma, sozinha na seleção, com UMA alça arrastada pelo MESMO press e o MESMO
//! arrasto que a shell chama (`shape_handles_live::on`): o canto arredonda mais, o vale da estrela
//! abre, e o fim da pizza dá a volta até aos 300°. A foto final mostra a pizza seleccionada, com as
//! três alças dela à vista.

use crate::shape_handles_live::{on, view};
use crate::state::VecState;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::shape_handles::{Law, Mods, handles_of};
use ph2d_vec_scene::{ShapeKind, VecPathId};

/// As três formas: o tipo, o centro (m) e a meia-largura (m) — lado a lado na vista de omissão.
pub(crate) const FORMAS: [(ShapeKind, [f64; 2], f64); 3] = [
    (ShapeKind::RoundRect, [-3.2, 0.0], 1.1),
    (ShapeKind::Star, [0.0, 0.0], 1.2),
    (ShapeKind::Pie, [3.2, 0.0], 1.1),
];

/// Metros por pixel da vista de omissão (`±5 m` na vertical) — o mesmo das outras cenas.
const PX: f64 = 0.01;

/// A forma `k` (pela ordem de nascimento — os ids crescem).
fn nth(vec: &VecState, k: usize) -> Option<VecPathId> {
    vec.entities.keys().nth(k).copied()
}

/// Agarra a alça `handle` da forma `id` (sozinha na seleção) pelo press da shell e leva-a pelos
/// `path` (mundo) com o arrasto da shell; solta. `false` = a alça não estava lá.
fn pull(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    id: VecPathId,
    handle: usize,
    path: impl Fn([f64; 2]) -> Vec<[f64; 2]>,
) -> bool {
    vec.pen.select_many(&[id]);
    let Some(at) =
        view(sim, &vec.entities, &[id], PX, None, None).and_then(|v| v.points.get(handle).copied())
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

/// Um passo da cena 15; devolve o próximo estágio, se mudou. `70..=72` = uma forma por estágio
/// (cada uma espera a anterior virar objecto) · `73` = a Seleção pelo botão · `74` = o canto ·
/// `75` = o vale · `76` = o fim da pizza · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        s @ 70..=72 if feitas == usize::from(s - 70) => {
            let (kind, at, half) = FORMAS[usize::from(s - 70)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, half);
            Some(s + 1)
        }
        73 if feitas == FORMAS.len() => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(74)
        }
        // O canto de cima à esquerda, 0,45 m para dentro ao longo da aresta: os quatro arredondam.
        74 if vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select => {
            let id = nth(vec, 0)?;
            pull(vec, scene, sim, id, 0, |a| {
                (1..=6)
                    .map(|k| [a[0] + 0.075 * f64::from(k), a[1]])
                    .collect()
            })
            .then_some(75)
        }
        // O vale da estrela, para FORA ao longo do raio dele (o centro é o da forma).
        75 => {
            let id = nth(vec, 1)?;
            let inner = handles_of(ShapeKind::Star)
                .iter()
                .position(|h| h.law == Law::Radial)?;
            let c = FORMAS[1].1;
            pull(vec, scene, sim, id, inner, move |a| {
                (1..=6)
                    .map(|k| {
                        let t = 1.0 + 0.1 * f64::from(k);
                        [c[0] + (a[0] - c[0]) * t, c[1] + (a[1] - c[1]) * t]
                    })
                    .collect()
            })
            .then_some(76)
        }
        // O fim da pizza (180°) dá a volta, em passos de 10°, até aos 300°.
        76 => {
            let id = nth(vec, 2)?;
            let end = handles_of(ShapeKind::Pie)
                .iter()
                .position(|h| matches!(h.law, Law::Sweep { .. }))?;
            let (c, r) = (FORMAS[2].1, FORMAS[2].2);
            pull(vec, scene, sim, id, end, move |_| {
                (1..=12)
                    .map(|k| {
                        let a = (180.0 + 10.0 * f64::from(k)).to_radians();
                        [c[0] + r * a.cos(), c[1] + r * a.sin()]
                    })
                    .collect()
            })
            .then_some(9)
        }
        _ => None,
    }
}
