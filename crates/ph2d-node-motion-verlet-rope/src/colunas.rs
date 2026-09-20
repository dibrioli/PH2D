//! **COMO SE LÊ O ESTADO DA CORDA** — os leitores de coluna do `motion.verlet_rope`.
//!
//! ⚠️ **Irmão de [`super`] por RESPONSABILIDADE:** ali vive a LEI (o passo de Verlet, as
//! restrições, os pinos e o que a corda publica); aqui o que ela lê do stream que lhe chega — o
//! estado do tique anterior, a massa inversa, a aceleração e a coordenada do ancoradouro. As duas
//! mudam por razões diferentes: uma quando a física muda, a outra quando uma coluna muda de nome.
//!
//! ⛔ Cada um deles é TOLERANTE por construção — coluna ausente ou com o tamanho errado devolve o
//! neutro —, e essa tolerância é a razão de eles viverem juntos: um leitor novo que se esqueça
//! dela fica visível ao lado dos irmãos.

use super::{Column, INV_MASS_COL, Stream};

/// The value coordinate for element 0 (the anchor): **unconnected (empty) → 0.0**;
/// otherwise the first element (broadcast).
pub(super) fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

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
/// ## What this one column buys
///
/// A `force.*` node wired into this rope's `state` chain (`rope.out --pre-->
/// force.wind --> rope.state`) accumulates world-units/s² here, and reading it
/// hands the rope the WHOLE force family at once: gravity with a DIRECTION,
/// wind, curl, an attractor, a vortex, drag. None of that is a kernel this crate
/// has to grow — it is one column, read once (doc 89 §2.1).
///
/// ⚠️ **Consumed, never emitted.** The emitted stream carries `P`/`rope_prev`/
/// `sim_t` and nothing else, so every tick starts from zero acceleration — the
/// same discipline `motion.integrate` states in its own docs. A rope that
/// forwarded `accel` would integrate last tick's wind forever.
///
/// ⚠️ **Zeros are the IDENTITY, so a rope no force reaches is byte-identical to
/// the one that shipped**: `x + 0.0 * dt²` is `x`. That is a property of the
/// arithmetic, not a fast path to keep in step with a slow one.
/// A massa inversa por ponto, alargada a `n` e tornada segura — o espelho exacto
/// do leitor do `motion.collide`: **ausente lê como livre (`1`)**, e um peso
/// negativo ou não-finito de um documento editado à mão lê como **pinado (`0`)**
/// em vez de INVERTER a correção.
///
/// ## O que esta coluna compra, e por que é a MESMA porta do `accel`
///
/// Um `motion.pin_constraint` na cadeia de estado desta corda
/// (`rope.out --pre--> pin --> rope.state`) prega um ÍNDICE ARBITRÁRIO — a
/// capacidade que a folha 03 pedia (linha 51) e que o doc do pino declarava
/// inalcançável (*"um pino a montante não tem fio por onde os alcançar"*). O fio
/// é a cadeia de estado, e ela já era o fio pelo qual o `accel` entra.
///
/// ⚠️ **Consumida, nunca emitida** — a mesma disciplina do `accel`, e aqui a razão
/// é MEDÍVEL: o `motion.pin_constraint` MULTIPLICA no que já está no stream, então
/// uma corda que reemitisse `inv_mass` faria um pino parcial de `0,5` decair
/// `0,5 → 0,25 → 0,125` a cada tique — o *produto sobre a lista* que este módulo
/// já pagou noutro lugar. Emitida uma vez por tique pelo pino, lida uma vez.
pub(super) fn inv_mass_col(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// A fração da correção que cabe a cada ponta de uma restrição — a fórmula PBD
/// `w_i / (w_i + w_j)`, a MESMA que o `motion.collide` usa no seu empurrão.
///
/// ⚠️ **Ela REDUZ LITERALMENTE à tabela de quatro braços que shipava** quando os
/// pesos são `{0, 1}`: `0/1 = 0`, `1/1 = 1` e `1/2 = 0,5` são todos EXACTOS em
/// IEEE-754, e o par degenerado `(0, 0)` cai no guard. É por isso que uma corda
/// que nenhum pino alcança é byte-idêntica — por ARITMÉTICA, não por promessa.
pub(super) fn share(wa: f32, wb: f32) -> (f32, f32) {
    let sum = wa + wb;
    if sum <= 0.0 {
        return (0.0, 0.0);
    }
    (wa / sum, wb / sum)
}

pub(super) fn accel_col(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("accel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}
