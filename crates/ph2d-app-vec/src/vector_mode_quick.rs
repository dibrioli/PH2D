//! ⭐ **A cena 12** (`PH2D_OBJECT_MODE_SMOKE=12`, D10 F7 — 08/10): o QUICKSHAPE. No layout *Vetor*,
//! o Lápis pelo botão dele na coluna ① e oito traços de mão SINTÉTICOS: um rectângulo tosco, um
//! círculo tosco, uma linha trémula, um círculo PEQUENO (F7b), um ARCO e o gancho ASSIMÉTRICO da
//! captura do dono (F7c), uma SETA de cabo curvo (F7d), cada um SEGURADO no fim — e um gelado SOLTO
//! sem segurar, o traço livre com quinas vivas e a bola curva (F7b). Quem
//! mede o segurar é a SHELL, a cada quadro, com o relógio do ponteiro (o `clock_ns` do chrome é esse
//! relógio): a cena só carimba as amostras e espera. No fim os oito ficam seleccionados com o
//! Lápis ainda na mão: os nós à vista, o QuickShape aceso, a dica no ⑤.
//!
//! ⭐ **A cena 13** (`=13`, F7e — 10/10): o EDIT SHAPE. Só o rectângulo tosco, seguro (vira o
//! rectângulo perfeito); o botão «Edit Shape» da barra ④ pelo MESMO clique que o rato dá; e o canto
//! de baixo-direita arrastado pelo MESMO press que a shell chama — o rectângulo cresce e continua
//! rectângulo, com os nós dele à vista e a dica do Edit Shape no ⑤.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_edit::pencil_width::PenDynamics;
use std::cell::Cell;

/// Mundo por px da câmera de abertura — MEDIDO na foto da cena (o rectângulo de 1,5 unidades ocupa
/// 146 px a 101 %). ⚠️ Era 0,0045 (copiado do `pencil_smoke`): o «círculo de 15 px» tinha 7.
const PX_TO_WORLD: f64 = 0.0103;
/// Uma amostra a cada 8 ms e a cada ~4 px: o passo de uma mão.
const STEP_NS: u128 = 8_000_000;
const STEP_WORLD: f64 = 4.0 * PX_TO_WORLD;
/// Se a forma não nascer em 3 s (o segurar é 0,5 s), a cena solta na mesma: a foto mostra a falha.
const GIVE_UP_NS: u128 = 3_000_000_000;
const STROKES: usize = 8;
/// Os primeiros `HELD` traços são SEGUROS (QuickShape); o último é solto logo — o traço livre.
const HELD: usize = 7;

/// O arrasto do canto na cena 13 (mundo): para a direita e para baixo, num passo de mão.
const EDIT_DRAG: [f64; 2] = [0.45, -0.3];

thread_local! {
    static STROKE: Cell<usize> = const { Cell::new(0) };
    static PRESSED: Cell<u128> = const { Cell::new(0) };
}

/// Um polígono amostrado a passo de mão, com um tremor determinístico de `amp` (mundo).
fn hand(poly: &[[f64; 2]], amp: f64) -> Vec<[f64; 2]> {
    let mut out = vec![poly[0]];
    for w in poly.windows(2) {
        let n = ((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]) / STEP_WORLD)
            .ceil()
            .max(1.0) as usize;
        out.extend((1..=n).map(|i| {
            let u = i as f64 / n as f64;
            [
                w[0][0] + (w[1][0] - w[0][0]) * u,
                w[0][1] + (w[1][1] - w[0][1]) * u,
            ]
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

/// Os oito traços (mundo, y para cima).
fn stroke(k: usize) -> Vec<[f64; 2]> {
    match k {
        // Um rectângulo tosco: os cantos fora do sítio e o fim a 20 px do início.
        0 => hand(
            &[
                [-2.0, 0.2],
                [-0.5, 0.26],
                [-0.56, 1.15],
                [-1.96, 1.1],
                [-2.0, 0.29],
            ],
            0.006,
        ),
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
        2 => hand(&[[-1.8, -0.85], [1.6, -0.55]], 0.012),
        // Um círculo PEQUENO — raio 15 px, amostras a ~2 px como a mão devagar (F7b, o dono 09/10:
        // «formas pequenas devem funcionar»).
        3 => {
            let r = 15.0 * PX_TO_WORLD;
            (0..=48)
                .map(|i| {
                    let a = std::f64::consts::TAU * 0.97 * f64::from(i) / 48.0;
                    let wobble = 0.4 * PX_TO_WORLD * (f64::from(i) * 1.7).sin();
                    [2.0 + (r + wobble) * a.cos(), 0.65 + (r + wobble) * a.sin()]
                })
                .collect()
        }
        // Um ARCO (F7c, o dono 10/10: «QuickShape com arcos»): um sorriso de 130° por baixo da
        // linha, o raio a respirar ±3 % — segurado, vira um arco de círculo perfeito.
        4 => {
            let ring: Vec<[f64; 2]> = (0..=40)
                .map(|i| {
                    let a = (205.0 + 130.0 * f64::from(i) / 40.0).to_radians();
                    let r = 0.7 * (1.0 + 0.03 * (4.0 * a).sin());
                    [-0.4 + r * a.cos(), -0.75 + r * a.sin()]
                })
                .collect();
            hand(&ring, 0.004)
        }
        // O gancho ASSIMÉTRICO da captura do dono (F7c, 10/10: «arcos assimétricos como na foto não
        // funcionam»), a meio tamanho: abre largo e fecha apertado — segurado, vira um biarco.
        5 => {
            let k = 0.5 * PX_TO_WORLD;
            let hook: Vec<[f64; 2]> = ph2d_vec_edit::quickshape::OWNERS_HOOK_PX
                .iter()
                .map(|p| [0.3 + (p[0] - 390.0) * k, 2.2 - (p[1] - 193.0) * k])
                .collect();
            hand(&hook, 0.002)
        }
        // Uma SETA de cabo CURVO (F7d, o dono 10/10: «uma linha ou um arco com um bico no fim e
        // SEGURAR»): um arco de 140° por cima do rectângulo e, na ponta, o V aberto — farpa,
        // ponta, farpa — de ~20 px a 30°. Segurada, vira o arco perfeito com a ponta do traço.
        6 => {
            let mut arrow: Vec<[f64; 2]> = (0..=40)
                .map(|i| {
                    let a = (160.0 - 140.0 * f64::from(i) / 40.0_f64).to_radians();
                    [-1.3 + 0.75 * a.cos(), 1.55 + 0.75 * a.sin()]
                })
                .collect();
            let (tip, prev) = (arrow[40], arrow[39]);
            let back = (prev[1] - tip[1]).atan2(prev[0] - tip[0]);
            let barb = |turn: f64| {
                let a = back + turn.to_radians();
                [tip[0] + 0.2 * a.cos(), tip[1] + 0.2 * a.sin()]
            };
            arrow.extend([barb(30.0), tip, barb(-30.0)]);
            hand(&arrow, 0.002)
        }
        // Um GELADO SOLTO sem segurar (F7b): a aba, a ponta do cone, a outra aba e o começo da
        // bola são quatro quinas vivas; a bola continua uma curva suave — o lápis com quinas.
        _ => {
            let mut cone = vec![
                [1.95, -0.7],
                [2.2, -0.7],
                [2.6, -1.6],
                [3.0, -0.7],
                [3.25, -0.7],
            ];
            cone.extend((1..=32).map(|i| {
                let a = std::f64::consts::PI * f64::from(i) / 32.0;
                [2.6 + 0.65 * a.cos(), -0.7 + 0.65 * a.sin()]
            }));
            hand(&cone, 0.002)
        }
    }
}

/// Um passo da cena 12 (ou da 13, `edit`); devolve o próximo estágio, se mudou. `50` = o Lápis
/// pelo botão · `51` = aceso · `52` = um traço desenhado e SEGURO · `53` = à espera da forma,
/// depois solta · `54` = os oito seleccionados, com o Lápis ainda na mão · `55` = o botão Edit
/// Shape · `56` = os nós à vista, o canto arrastado · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
    edit: bool,
) -> Option<u8> {
    let lit = |hero: &HeroScreen, id| {
        hero.store
            .tool_bars()
            .is_some_and(|b| b.column.iter().flatten().any(|x| x.on && x.id == id))
    };
    let now = hero.store.clock_ns();
    match stage {
        50 => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_PENCIL,
            ));
            Some(51)
        }
        51 if lit(hero, ph2d_tool_vector::ids::VECTOR_MODE_PENCIL) => Some(52),
        52 => {
            // A mão acabou AGORA: a última amostra leva o relógio de agora, as outras o passado.
            let pts = stroke(STROKE.with(Cell::get));
            let n = pts.len() as u128;
            let at = |i: usize| PenDynamics {
                pressure: 1.0,
                t_ns: now - (n - 1 - i as u128) * STEP_NS,
            };
            vec.pencil.on_press(scene, pts[0], PX_TO_WORLD, at(0));
            for (i, &p) in pts.iter().enumerate().skip(1) {
                vec.pencil.on_drag(scene, p, at(i));
            }
            PRESSED.with(|t| t.set(now));
            Some(53)
        }
        53 if STROKE.with(Cell::get) >= HELD
            || vec.pencil.is_quick()
            || now.saturating_sub(PRESSED.with(Cell::get)) > GIVE_UP_NS =>
        {
            if STROKE.with(Cell::get) < HELD && !vec.pencil.is_quick() {
                eprintln!(
                    "[smoke] quickshape: o traço {} NÃO virou forma em 3 s",
                    STROKE.with(Cell::get)
                );
            }
            if vec.pencil.on_release(scene) {
                vec.pen.select(vec.pencil.selected());
            }
            let k = STROKE.with(|s| {
                s.set(s.get() + 1);
                s.get()
            });
            Some(if edit {
                55
            } else if k < STROKES {
                52
            } else {
                54
            })
        }
        // A cena 13: o botão da ④ — o clique é o do rato (o painel guarda-o; o quadro do lápis
        // abre os nós no quadro seguinte).
        55 => {
            hero.apply_event(WidgetEvent::Click(
                ph2d_panel_vector::ids::VECTOR_PENCIL_EDIT_SHAPE,
            ));
            Some(56)
        }
        56 if vec.pencil.shape_edit_state() == ph2d_vec_edit::pencil::ShapeEditState::Open => {
            let dyn_at = |t_ns| PenDynamics {
                pressure: 1.0,
                t_ns,
            };
            // O canto de baixo-direita (y para cima: o de menor y entre os de maior x).
            let nodes = vec.pencil.shape_edit_nodes().0;
            let corner = nodes
                .iter()
                .copied()
                .max_by(|a, b| (a[0] - a[1]).total_cmp(&(b[0] - b[1])))?;
            crate::pencil_input::press(vec, scene, corner, PX_TO_WORLD, dyn_at(now), false);
            for i in 1..=8 {
                let f = f64::from(i) / 8.0;
                let p = [corner[0] + EDIT_DRAG[0] * f, corner[1] + EDIT_DRAG[1] * f];
                vec.pencil.on_drag(scene, p, dyn_at(now));
            }
            if vec.pencil.on_release(scene) {
                vec.pen.select(vec.pencil.selected());
            }
            Some(9)
        }
        // O Lápis FICA na mão (a fila mostra o QuickShape aceso e o rodapé ⑤ a dica); os oito
        // seleccionados mostram os nós.
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
    fn the_synthetic_strokes_are_the_shapes_the_scene_promises() {
        use ph2d_vec_edit::quickshape::{QuickShape, recognise};
        let got: Vec<_> = (0..super::HELD)
            .map(|k| recognise(&super::stroke(k), super::PX_TO_WORLD, false))
            .collect();
        assert!(
            matches!(&got[0], Some(QuickShape::Polygon { corners, closed: true }) if corners.len() == 4),
            "{:?}",
            got[0]
        );
        assert!(
            matches!(&got[1], Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry),
            "{:?}",
            got[1]
        );
        assert!(
            matches!(&got[2], Some(QuickShape::Line { .. })),
            "{:?}",
            got[2]
        );
        assert!(
            matches!(&got[3], Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry),
            "o círculo pequeno: {:?}",
            got[3]
        );
        assert!(
            matches!(&got[4], Some(QuickShape::Arc(ph2d_vec_edit::quickshape::ArcShape { sweep, .. }))
                if (sweep.to_degrees() - 130.0).abs() < 10.0),
            "o arco: {:?}",
            got[4]
        );
        assert!(
            matches!(&got[5], Some(QuickShape::Biarc(_))),
            "o gancho: {:?}",
            got[5]
        );
        assert!(
            matches!(&got[6], Some(QuickShape::Arrow(a))
                if a.heads[0].is_none() && a.heads[1].is_some()
                    && matches!(*a.shaft, QuickShape::Arc(_) | QuickShape::Biarc(_))),
            "a seta de cabo curvo: {:?}",
            got[6]
        );
    }

    #[test]
    fn the_free_stroke_keeps_its_corners_and_its_curve() {
        use ph2d_vec_scene::VertexKind;
        let pts = super::stroke(super::HELD);
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut pencil = ph2d_vec_edit::Pencil::default();
        let d = |i: usize| ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: i as u128 * 8_000_000,
        };
        let id = pencil.on_press(&mut scene, pts[0], super::PX_TO_WORLD, d(0));
        for (i, &p) in pts.iter().enumerate().skip(1) {
            pencil.on_drag(&mut scene, p, d(i));
        }
        assert!(pencil.on_release(&mut scene));
        let v = &scene.path(id).expect("o traço").verts;
        let corners = v[1..v.len() - 1]
            .iter()
            .filter(|x| x.kind == VertexKind::Corner)
            .count();
        let smooth = v.iter().filter(|x| x.kind == VertexKind::Smooth).count();
        assert_eq!(
            corners, 4,
            "as duas abas, a ponta e o começo da bola: {v:?}"
        );
        assert!(smooth >= 2, "a bola continua curva: {v:?}");
    }
}
