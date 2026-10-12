//! ⭐ **A cena 19** (`PH2D_OBJECT_MODE_SMOKE=19`, F12 — 12/10): AS SETAS LADO A LADO. No layout
//! *Vetor*, um polígono, uma estrela, uma pizza e uma espiral; o botão *Forma*; e, pelo MESMO press e
//! arrasto da shell (`shape_handles_live::on`), um gesto em cada: a aresta do polígono enverga (o
//! ponto do meio dela), a estrela sobe a 6 pontas (o 1.º ⇕), a pizza abre o furo e arredonda as
//! quinas (o ⊙), e a ponta da espiral dá ¼ de volta À MÃO — ela segue o ângulo do ponteiro. A foto
//! final: a espiral seleccionada com a ponta e os dois ⇕ na borda, e o ⇕ do *Expansion* A MEIO do
//! arrasto, com a etiqueta do número à vista (o 1.º clique larga-o). `PH2D_CENA19_FICA=0..3` deixa
//! seleccionada outra forma (0 = polígono · 1 = estrela · 2 = pizza) — para fotografar as setas dela.

use crate::shape_handles_live::{Gate, on, view};
use crate::state::VecState;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::shape_handles::{HandleDesc, Law, Mods, Spot, handles_of};
use ph2d_vec_scene::{ShapeKind, VecPathId};

/// As quatro formas: o tipo e o centro (m), lado a lado; a meia-largura é [`MEIA`].
pub(crate) const FORMAS: [(ShapeKind, [f64; 2]); 4] = [
    (ShapeKind::Polygon, [-4.8, 0.0]),
    (ShapeKind::Star, [-1.6, 0.0]),
    (ShapeKind::Pie, [1.6, 0.0]),
    (ShapeKind::Spiral, [4.8, 0.0]),
];

/// A meia-largura de cada forma (m) — a fila de ⇕ cabe entre uma forma e a seguinte.
pub(crate) const MEIA: f64 = 0.9;

/// Metros por pixel da vista de omissão — o mesmo das outras cenas.
const PX: f64 = 0.01;

/// Quanto a mão sobe um ⇕ (m): 1,2 degraus de 1/24 da altura da caixa — um inteiro anda UM.
pub(crate) const SOBE: f64 = 1.2 * 2.0 * MEIA / 24.0;

/// A forma que fica seleccionada no fim, com o ⇕ dela a meio do arrasto (a espiral por omissão).
fn fica() -> usize {
    std::env::var("PH2D_CENA19_FICA")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|k: &usize| *k < FORMAS.len())
        .unwrap_or(3)
}

/// A 1.ª alça de `kind` que `pick` aceita.
pub(crate) fn alca(kind: ShapeKind, pick: impl Fn(&HandleDesc) -> bool) -> Option<usize> {
    handles_of(kind).iter().position(pick)
}

/// A alça que fica VIVA no fim: o 1.º ⇕ da forma; a pizza não tem ⇕ — o furo.
pub(crate) fn viva(kind: ShapeKind) -> Option<usize> {
    alca(kind, |h| h.spot == Spot::Rail).or_else(|| alca(kind, |h| h.law == Law::Radial))
}

/// Agarra a alça `handle` da forma `id` (sozinha na seleção) pelo press da shell e leva-a pelo
/// `path(onde ela está)`; `soltar` = o botão sobe no fim. `false` = a alça não estava lá.
fn pull(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    id: VecPathId,
    handle: usize,
    path: impl Fn([f64; 2]) -> Vec<[f64; 2]>,
    soltar: bool,
) -> bool {
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
    !soltar || vec.shape_handles.release()
}

/// A mão anda de `a` até `a + d`, em seis passos.
fn walk(d: [f64; 2]) -> impl Fn([f64; 2]) -> Vec<[f64; 2]> {
    move |a| {
        (1..=6)
            .map(|k| {
                let t = f64::from(k) / 6.0;
                [a[0] + d[0] * t, a[1] + d[1] * t]
            })
            .collect()
    }
}

/// A mão anda de `a` para FORA (ou para dentro, `len < 0`) ao longo do raio desde `c`.
fn radial(c: [f64; 2], len: f64) -> impl Fn([f64; 2]) -> Vec<[f64; 2]> {
    move |a| {
        let (dx, dy) = (a[0] - c[0], a[1] - c[1]);
        let n = dx.hypot(dy).max(1e-12);
        walk([dx / n * len, dy / n * len])(a)
    }
}

/// Um passo da cena 19; devolve o próximo estágio, se mudou. `110..=113` = uma forma por estágio ·
/// `114` = o modo Forma pelo botão · `115` = a aresta do polígono · `116` = a estrela a 6 pontas ·
/// `117` = o furo da pizza · `118` = as quinas dela · `119` = a ponta da espiral, ¼ de volta à mão ·
/// `120` = o ⇕ que fica a meio do arrasto · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    let nth = |vec: &VecState, k: usize| vec.entities.keys().nth(k).copied();
    match stage {
        s @ 110..=113 if feitas == usize::from(s - 110) => {
            let (kind, at) = FORMAS[usize::from(s - 110)];
            crate::vector_mode::draw_shape(vec, scene, kind, at, MEIA);
            Some(s + 1)
        }
        114 if feitas == FORMAS.len() => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(115)
        }
        // O ponto do meio da 1.ª aresta, 0,15 m para fora: a aresta enverga.
        115 if vec.draw_config.mode == ph2d_tool_vector::DrawMode::Shape => {
            let (kind, c) = FORMAS[0];
            let k = alca(kind, |h| matches!(h.spot, Spot::EdgeMid(_)))?;
            pull(vec, scene, sim, nth(vec, 0)?, k, radial(c, 0.15), true).then_some(116)
        }
        // O 1.º ⇕ da estrela (as pontas) sobe um degrau: 6 pontas.
        116 => {
            let k = viva(FORMAS[1].0)?;
            pull(vec, scene, sim, nth(vec, 1)?, k, walk([0.0, SOBE]), true).then_some(117)
        }
        // O furo da pizza, 0,3 m para fora pelo raio.
        117 => {
            let (kind, c) = FORMAS[2];
            let k = alca(kind, |h| h.law == Law::Radial)?;
            pull(vec, scene, sim, nth(vec, 2)?, k, radial(c, 0.3), true).then_some(118)
        }
        // O ⊙ das quinas, 0,15 m para o centro pela borda recta do início (às 3h): arredondam.
        118 => {
            let k = alca(FORMAS[2].0, |h| matches!(h.spot, Spot::Rounding { .. }))?;
            pull(vec, scene, sim, nth(vec, 2)?, k, walk([-0.15, 0.0]), true).then_some(119)
        }
        // A ponta da espiral dá ¼ de volta à volta do centro, de 10° em 10° — ela segue a mão.
        119 => {
            let (kind, c) = FORMAS[3];
            let k = alca(kind, |h| h.law == Law::Winding)?;
            let quarto = move |a: [f64; 2]| -> Vec<[f64; 2]> {
                let (r, a0) = (
                    (a[0] - c[0]).hypot(a[1] - c[1]),
                    (a[1] - c[1]).atan2(a[0] - c[0]),
                );
                (1..=9)
                    .map(|s| {
                        let t = a0 + (10.0 * f64::from(s)).to_radians();
                        [c[0] + r * t.cos(), c[1] + r * t.sin()]
                    })
                    .collect()
            };
            pull(vec, scene, sim, nth(vec, 3)?, k, quarto, true).then_some(120)
        }
        // O ⇕ da forma que fica: agarrado e subido, SEM soltar — a etiqueta fica à vista.
        120 => {
            let f = fica();
            let (kind, c) = FORMAS[f];
            let k = viva(kind)?;
            let path: Box<dyn Fn([f64; 2]) -> Vec<[f64; 2]>> =
                if handles_of(kind)[k].law == Law::Radial {
                    Box::new(radial(c, 0.15))
                } else {
                    Box::new(walk([0.0, SOBE]))
                };
            pull(vec, scene, sim, nth(vec, f)?, k, path, false).then_some(9)
        }
        _ => None,
    }
}
