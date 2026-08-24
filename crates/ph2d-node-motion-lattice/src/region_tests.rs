//! Os gates da **FORMA** desta colmeia (doc 89, folha 01) — a mesma lei do
//! `motion.grid`: a rede não se dobra, então a forma **recorta**.

use super::*;
use ph2d_motion_region::{Region, SHAPE_CIRCLE, SHAPE_RING};

const SIDE: usize = 15;
const SPACING: f32 = 0.4;

fn built() -> Vec<[f32; 2]> {
    lattice(SIDE, SIDE, SPACING, 1, &[])
}

fn region(shape: f32, inner: f32) -> Region {
    Region::of(
        shape,
        (SIDE as f32 - 1.0) * SPACING + SPACING * 0.5,
        (SIDE as f32 - 1.0) * SPACING * ROW_PITCH,
        inner,
    )
}

/// ⭐ **O RETÂNGULO NÃO PERDE UM PONTO NEM MOVE UM BIT.**
#[test]
fn the_default_shape_keeps_the_whole_honeycomb_bit_for_bit() {
    let raw = built();
    let cut = carve(built(), &region(0.0, 0.9));
    assert_eq!(cut.len(), SIDE * SIDE);
    for (i, (p, q)) in cut.iter().zip(&raw).enumerate() {
        assert_eq!(p.map(f32::to_bits), q.map(f32::to_bits), "ponto {i}");
    }
}

/// ⭐ **O CÍRCULO RECORTA** — e o que sobra está dentro dele.
#[test]
fn the_circle_carves_the_honeycomb() {
    let circle = region(SHAPE_CIRCLE as f32, 0.0);
    let cut = carve(built(), &circle);
    assert!(
        cut.len() < SIDE * SIDE && cut.len() > SIDE * SIDE / 2,
        "cortou sem apagar: {} de {}",
        cut.len(),
        SIDE * SIDE
    );
    for p in &cut {
        assert!(circle.contains(*p), "sobrou um de fora: {p:?}");
    }
}

/// ⭐⭐ **A COLMEIA CONTINUA SENDO UMA COLMEIA depois do corte** — a propriedade que
/// define este nó (*todo vizinho mais próximo à distância `spacing` exacta*) é sobre a
/// REDE, e recortar não pode inventar um espaçamento novo.
///
/// ⚠️ **É o gate que separa «recortar» de «re-arranjar»**: um nó que empacotasse os
/// pontos dentro do círculo teria de os mover, e aí a distância mínima mudaria. Aqui
/// ela é a mesma, ao bit.
#[test]
fn carving_never_invents_a_new_spacing() {
    let closest = |pts: &[[f32; 2]]| {
        let mut m = f32::MAX;
        for (i, p) in pts.iter().enumerate() {
            for q in &pts[i + 1..] {
                m = m.min((p[0] - q[0]).hypot(p[1] - q[1]));
            }
        }
        m
    };
    let whole = closest(&built());
    for (shape, inner) in [(SHAPE_CIRCLE as f32, 0.0), (SHAPE_RING as f32, 0.5)] {
        let cut = carve(built(), &region(shape, inner));
        assert!(cut.len() > 10, "shape={shape}: {} pontos", cut.len());
        let after = closest(&cut);
        assert!(
            after >= whole - 1e-6,
            "shape={shape}: o corte apertou o espacamento {whole:.6} -> {after:.6}"
        );
    }
}

/// A casca do C4D é este anel de buraco grande, e a banda encolhe monotonamente.
#[test]
fn the_shell_is_a_ring_with_a_big_hole() {
    let n = |inner: f32| carve(built(), &region(SHAPE_RING as f32, inner)).len();
    let (a, b, c) = (n(0.0), n(0.5), n(0.85));
    assert!(a > b && b > c && c > 0, "{a} > {b} > {c} > 0");
}

/// Os params novos são declarados, com hint, e reduzem ao nó de hoje.
#[test]
fn every_new_param_is_declared_and_defaults_to_today() {
    assert_eq!(MANIFEST.param_default(ph2d_motion_region::SHAPE), Some(0.0));
    for h in [ph2d_motion_region::SHAPE, ph2d_motion_region::INNER] {
        assert!(
            PARAM_HINTS.iter().any(|x| x.param == h),
            "{h} sem hint de painel"
        );
    }
}
