//! **O MUNDO da 1.ª cópia do DISPOSITIVO para a sonda `PH2D_PAN_DIAG`** (doc 123 §8, Bug #14).
//!
//! Na rota da placa o mundo das cópias não volta à CPU; a sonda lia só contagens. A 1.ª cópia
//! viaja pela **leitura dos cartões que já existe** (`motion.gpu_tap`, `readout::take_tap`): ela
//! amostra TODO nó encenado, a saída incluída, e a amostra `0` é a linha `0` (o gather do
//! `ph2d_gpu_cook::tap` faz `e = i·n/amostras`). ⇒ zero download novo, zero tomada nova.
//!
//! ⛔ **Nunca uma tomada da BOMBA** (`pump.set_taps`): na rota da placa ela recoze a cadeia na CPU
//! (`cook_taps_only`, doc 120 §8.5 — `2,4 ms` a `32 768`) e a sonda mediria a CPU, não a placa.
//!
//! ⚠️ A leitura dos cartões vem `1`–`2` quadros atrás do cozimento (doc 120 §8.6); a linha diz.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;

/// **O mundo da 1.ª cópia** — a linha `0` da 1.ª saída (pela ordem de `sinks`) com linhas.
///
/// O MESMO número que a rota da CPU dá à sua 1.ª instância: o lowering faz
/// `world_pos = P[i]` (omissão `[0, 0]`, `ph2d_eval_motion::lower`) — sem transformação de objecto
/// por cima; as saídas baixam-se pela ordem delas, uma a seguir à outra.
#[must_use]
pub fn mundo_da_primeira_copia(
    lida: &BTreeMap<NodeId, Stream>,
    sinks: &[NodeId],
) -> Option<[f32; 2]> {
    let s = sinks
        .iter()
        .filter_map(|n| lida.get(n))
        .find(|s| s.count() > 0)?;
    Some(match s.get("P") {
        Some(Column::Vec2(p)) => p.first().copied().unwrap_or([0.0, 0.0]),
        _ => [0.0, 0.0],
    })
}

/// A porta do produto: a leitura dos cartões deste quadro, `None` sem leitura (CPU, 1.º quadro).
#[must_use]
pub fn mundo_lido(motion: &MotionState) -> Option<[f32; 2]> {
    mundo_da_primeira_copia(motion.gpu_tap.as_ref()?, &motion.sinks)
}

#[cfg(test)]
#[path = "pan_diag_tap_tests.rs"]
mod tests;
