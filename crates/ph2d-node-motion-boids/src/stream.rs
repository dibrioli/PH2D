//! **OS LEITORES DE COLUNA** do bando — como um `Stream` vira os vetores que o passo
//! de simulação consome, e as duas funções de vetor que ele usa.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`). O corte é
//! por RESPONSABILIDADE: nada aqui sabe o que é um bando — é a fronteira entre o
//! substrato de colunas e a lei de Reynolds.

use ph2d_nodegraph::attr::{Column, Stream};

use super::EPS;

pub(super) fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

pub(super) fn vec2_col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The transient `accel` the state carries, at exactly `n` — **absent is zeros**.
///
/// A `force.*` wired into this flock's `state` chain (`boids.out --pre-->
/// force.curl --> boids.state`) accumulates world-units/s² here, and this one
/// read hands the flock the whole force family: curl (which IS Reynolds'
/// *wander*), wind, an attractor, a vortex, drag (doc 89 §2.1). It joins the
/// three flocking urges as a fourth term, so `max_speed` still bounds it.
///
/// ⚠️ **Consumed, never emitted** (the stream carries `P`/`vel`/`sim_t`), so
/// every tick starts from zero acceleration; and zeros are the IDENTITY, so a
/// flock no force reaches is byte-identical to the one that shipped.
/// A massa INVERSA por agente (`motion.pin_constraint`): `1` = livre, `0` =
/// pinado. **Ausente lê como livre**, e um peso negativo ou não-finito lê como
/// pinado — o espelho exacto do leitor do `motion.collide`. Convenção de string
/// soletrada LOCALMENTE (como `P` / `accel`), sem acoplar as crates.
///
/// ⚠️ **Consumida, nunca emitida** — a disciplina do `accel`, e pelo mesmo motivo
/// medível: o pino MULTIPLICA no que já está no stream.
pub(super) fn inv_mass_col(s: &Stream, n: usize) -> Vec<f32> {
    match s.get("inv_mass") {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

pub(super) fn accel_col(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("accel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// The value coordinate for the (single) target: **unconnected → 0.0** (origin).
pub(super) fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// `(unit, len)` of `v`; `([0,0], 0)` for a ~zero vector (no NaN).
pub(super) fn norm(v: [f32; 2]) -> ([f32; 2], f32) {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < EPS {
        ([0.0, 0.0], 0.0)
    } else {
        ([v[0] / len, v[1] / len], len)
    }
}
