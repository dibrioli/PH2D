//! **O que o STREAM DE ESTADO carrega** — os leitores de coluna deste corpo.
//!
//! Cada nome (`P` · `sb_vel` · `sim_t` · `accel` · `inv_mass` · `falloff`) é uma
//! convenção de string do módulo, soletrada LOCALMENTE por quem a lê em vez de
//! acoplar as crates (a lei do `motion.verlet_rope`/`motion.boids`). Ficam aqui
//! juntos porque são a mesma pergunta — *o que o tique anterior me deixou* — e
//! porque o `lib.rs` bateu no tecto de LOC (HR-18).

use super::{Column, FALLOFF_COL, INV_MASS_COL, Stream};

pub(crate) fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

pub(crate) fn vec2_col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

pub(crate) fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// The transient `accel` the state carries, at exactly `n` — **absent is zeros**.
///
/// A `force.*` wired into this body's `state` chain (`soft_body.out --pre-->
/// force.vortex --> soft_body.state`) accumulates world-units/s² here, and this
/// one read hands the gelatin the whole force family: gravity with a DIRECTION,
/// wind, curl, an attractor, drag (doc 89 §2.1). It enters the PREDICTION, which
/// is where an acceleration belongs in a position-based step — the shape match
/// then answers it, which is exactly how a soft body should respond to being
/// blown on.
///
/// ⚠️ **Consumed, never emitted** (the stream carries `P`/`sb_vel`/`sim_t`), so
/// every tick starts from zero acceleration; and zeros are the IDENTITY, so a
/// body no force reaches is byte-identical to the one that shipped.
/// A massa inversa por partícula, alargada a `n` e tornada segura — o espelho
/// exacto do leitor do `motion.collide`: **ausente lê como livre (`1`)**, e um
/// peso negativo ou não-finito lê como **pinado (`0`)** em vez de inverter o puxão.
///
/// ⚠️ **Consumida, nunca emitida**, a mesma disciplina do `accel` — o
/// `motion.pin_constraint` MULTIPLICA no que já está no stream, então um corpo que
/// a reemitisse faria um pino parcial decair a cada tique.
pub(crate) fn inv_mass_col(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// **O PESO DE UMA PARTÍCULA NO CORPO** — o `wᵢ = mᵢ` de Müller 2005, que este
/// arquivo declara ter fixado em `1` desde que nasceu (*"masses are uniform here
/// — exact for this even grid, so the paper's mass-weighted centroid/`A_pq`
/// reduce to the plain sums"*). O `falloff` é esse peso, e a folha 03 pedia-o
/// como *goal/peso por partícula* (o **Goal** por vertex group do Blender
/// Softbody · a espinha MOPs *todo modificador é modulado por `mops_falloff`*).
///
/// ⚠️ **Devolve `None` quando a coluna está AUSENTE, e não um vetor de uns** — e
/// a distinção é byte-identidade, não gosto: com pesos o centroide de repouso
/// deixa de ser zero e passa a ser subtraído, e medido (a sonda
/// `is_the_rest_centroid_exactly_zero`) o centroide de uma malha real vale
/// `−1,192e-7`, não `0`. `None` deixa a lei correr com `c₀ = [0,0]` literal, que
/// é a identidade em IEEE-754 e é exactamente o corpo que sempre shipou.
///
/// ⚠️ **Consumida, nunca emitida**, a disciplina do `accel` e do `inv_mass`: o
/// stream emitido carrega `P`/`sb_vel`/`sim_t`, então um corpo que a reemitisse
/// faria um `field.*` no laço compor o peso consigo mesmo a cada tique — o
/// *produto sobre a lista* que esta casa já curou quatro vezes.
///
/// Fora de faixa é clampado: um documento editado à mão não pode dar peso
/// NEGATIVO a uma partícula e virar o ajuste do avesso.
pub(crate) fn falloff_col(s: &Stream, n: usize) -> Option<Vec<f32>> {
    match s.get(FALLOFF_COL) {
        Some(Column::Scalar(v)) if v.len() == n => {
            Some(v.iter().map(|f| f.clamp(0.0, 1.0)).collect())
        }
        _ => None,
    }
}

pub(crate) fn accel_col(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("accel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}
