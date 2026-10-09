//! ⭐ **A cena 12** (`PH2D_OBJECT_MODE_SMOKE=12`, D10 F7 — 08/10): o QUICKSHAPE. No layout *Vetor*,
//! o Lápis pelo botão dele na coluna ① e três traços de mão SINTÉTICOS — um rectângulo tosco, um
//! círculo tosco, uma linha trémula —, cada um SEGURADO no fim. Quem mede o segurar é a SHELL, a cada
//! quadro, com o relógio do ponteiro (o `clock_ns` do chrome é esse relógio): a cena só carimba as
//! amostras e espera, e cada traço vira a forma perfeita pela mesma porta da mão. No fim as três
//! ficam seleccionadas com o Lápis ainda na mão: os nós à vista, o QuickShape aceso, a dica no ⑤.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_edit::pencil_width::PenDynamics;
use std::cell::Cell;

/// Mundo por px da câmera de abertura (4 unidades em ~900 px — a do `pencil_smoke`).
const PX_TO_WORLD: f64 = 0.0045;
/// Uma amostra a cada 8 ms e a cada ~4 px: o passo de uma mão.
const STEP_NS: u128 = 8_000_000;
const STEP_WORLD: f64 = 4.0 * PX_TO_WORLD;
/// Se a forma não nascer em 3 s (o segurar é 0,5 s), a cena solta na mesma: a foto mostra a falha.
const GIVE_UP_NS: u128 = 3_000_000_000;
const STROKES: usize = 3;

thread_local! {
    static STROKE: Cell<usize> = const { Cell::new(0) };
    static PRESSED: Cell<u128> = const { Cell::new(0) };
}

/// Um polígono amostrado a passo de mão, com um tremor determinístico de `amp` (mundo).
fn hand(poly: &[[f64; 2]], amp: f64) -> Vec<[f64; 2]> {
    let mut out = vec![poly[0]];
    for w in poly.windows(2) {
        let n = ((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]) / STEP_WORLD).ceil().max(1.0) as usize;
        out.extend((1..=n).map(|i| {
            let u = i as f64 / n as f64;
            [w[0][0] + (w[1][0] - w[0][0]) * u, w[0][1] + (w[1][1] - w[0][1]) * u]
        }));
    }
    out.iter()
        .enumerate()
        .map(|(i, p)| {
            let f = i as f64;
            [p[0] + amp * (f * 0.7).sin(), p[1] + amp * (f * 1.3).cos()]
        })
        .collect()
}

/// Os três traços (mundo, y para cima).
fn stroke(k: usize) -> Vec<[f64; 2]> {
    match k {
        // Um rectângulo tosco: os cantos fora do sítio e o fim a 20 px do início.
        0 => hand(&[[-2.0, 0.2], [-0.5, 0.26], [-0.56, 1.15], [-1.96, 1.1], [-2.0, 0.29]], 0.006),
        // Um círculo tosco: o raio respira ±4 % e o fim pára antes de fechar.
        1 => {
            let ring: Vec<[f64; 2]> = (0..=96)
                .map(|i| {
                    let a = std::f64::consts::TAU * 0.97 * f64::from(i) / 96.0;
                    let r = 0.5 * (1.0 + 0.04 * (3.0 * a).sin());
                    [0.9 + r * a.cos(), 0.65 + r * a.sin()]
                })
                .collect();
            hand(&ring, 0.004)
        }
        // Uma linha trémula, inclinada.
        _ => hand(&[[-1.8, -0.85], [1.6, -0.55]], 0.012),
    }
}

/// Um passo da cena 12; devolve o próximo estágio, se mudou. `50` = o Lápis pelo botão · `51` =
/// aceso · `52` = um traço desenhado e SEGURO · `53` = à espera da forma, depois solta · `54` =
/// as três seleccionadas, com o Lápis ainda na mão · `9` = feito.
pub fn step(stage: u8, vec: &mut VecState, scene: &mut ph2d_vec_scene::VecScene, hero: &mut HeroScreen) -> Option<u8> {
    let lit = |hero: &HeroScreen, id| hero.store.tool_bars().is_some_and(|b| b.column.iter().flatten().any(|x| x.on && x.id == id));
    let now = hero.store.clock_ns();
    match stage {
        50 => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_PENCIL));
            Some(51)
        }
        51 if lit(hero, ph2d_tool_vector::ids::VECTOR_MODE_PENCIL) => Some(52),
        52 => {
            // A mão acabou AGORA: a última amostra leva o relógio de agora, as outras o passado.
            let pts = stroke(STROKE.with(Cell::get));
            let n = pts.len() as u128;
            let at = |i: usize| PenDynamics { pressure: 1.0, t_ns: now - (n - 1 - i as u128) * STEP_NS };
            vec.pencil.on_press(scene, pts[0], PX_TO_WORLD, at(0));
            for (i, &p) in pts.iter().enumerate().skip(1) {
                vec.pencil.on_drag(scene, p, at(i));
            }
            PRESSED.with(|t| t.set(now));
            Some(53)
        }
        53 if vec.pencil.is_quick() || now.saturating_sub(PRESSED.with(Cell::get)) > GIVE_UP_NS => {
            if !vec.pencil.is_quick() {
                eprintln!("[smoke] quickshape: o traço {} NÃO virou forma em 3 s", STROKE.with(Cell::get));
            }
            if vec.pencil.on_release(scene) {
                vec.pen.select(vec.pencil.selected());
            }
            let k = STROKE.with(|s| {
                s.set(s.get() + 1);
                s.get()
            });
            Some(if k < STROKES { 52 } else { 54 })
        }
        // O Lápis FICA na mão (a fila mostra o QuickShape aceso e o rodapé ⑤ a dica); as três
        // seleccionadas mostram os nós.
        54 => {
            let all: Vec<ph2d_vec_scene::VecPathId> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&all);
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_three_synthetic_strokes_are_the_shapes_the_scene_promises() {
        use ph2d_vec_edit::quickshape::{QuickShape, recognise};
        let got: Vec<_> = (0..super::STROKES).map(|k| recognise(&super::stroke(k), super::PX_TO_WORLD, false)).collect();
        assert!(matches!(&got[0], Some(QuickShape::Polygon { corners, closed: true }) if corners.len() == 4), "{:?}", got[0]);
        assert!(matches!(&got[1], Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry), "{:?}", got[1]);
        assert!(matches!(&got[2], Some(QuickShape::Line { .. })), "{:?}", got[2]);
    }
}
