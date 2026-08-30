//! ⭐⭐⭐ **A FITA DE GRADIENTE É PREGUIÇOSA — e continua a existir para quem a pede** (W70).
//!
//! O irmão do [`tape_budget`](../tape_budget.rs): lá prova-se que o traçado **não** compila
//! nenhuma, aqui que a extração **compila uma**. ⛔ **São dois binários porque o contador é do
//! processo** — no mesmo binário estes dois testes correm em paralelo e um zeraria a conta do
//! outro. *Duas leis sobre o mesmo contador global não cabem num binário só.*

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::{GRAD_TAPES, Hybrid, Registry};
use std::sync::atomic::Ordering;

/// A mesma peça de perfil do irmão, mais barata (o que se mede aqui não é o relógio).
fn profile_piece(edges: usize) -> FieldDoc {
    let contour: Vec<[f32; 2]> = (0..edges)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (edges as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão")
}

#[test]
fn the_gradient_tape_is_still_there_for_whoever_asks_and_is_built_once() {
    // ⚠️ **A outra metade da lei.** Adiar a montagem só é correcto se ela ainda acontecer quando
    // alguém a pede — e uma vez só, senão a extração paga-a por lote.
    let doc = profile_piece(64);
    let reg = Registry::new();
    let mut shape = Hybrid::new(&doc, &reg);
    GRAD_TAPES.store(0, Ordering::Relaxed);

    let pts = [0.30f32, 0.31, 0.32];
    let zeros = [0.0f32; 3];
    let mut out = Vec::new();
    shape
        .gradients(&pts, &zeros, &zeros, 1e-3, &mut out)
        .expect("gradiente");
    assert_eq!(
        GRAD_TAPES.load(Ordering::Relaxed),
        1,
        "o primeiro pedido de gradiente tem de compilar a fita"
    );
    assert!(
        out.iter().any(|g| g.iter().any(|c| c.abs() > 1e-6)),
        "a fita compilou e o gradiente saiu nulo — {out:?}"
    );

    let mut again = Vec::new();
    shape
        .gradients(&pts, &zeros, &zeros, 1e-3, &mut again)
        .expect("gradiente");
    assert_eq!(
        GRAD_TAPES.load(Ordering::Relaxed),
        1,
        "o segundo pedido recompilou a fita — ela guarda-se, e a extração pede-a por lote"
    );
    assert_eq!(out, again, "a mesma fita devolveu outra resposta");
}
