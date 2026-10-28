//! ⭐ **As PARTÍCULAS que a pilha de constraints prende** — o ponto de simulação
//! (`docs/BlenderConstraints/07_W6_pontas.md` §5).
//!
//! A constraint pede um objecto Motion (`ParticulasPedidas`); o Motion arma a TOMADA dos sinks dele
//! — a mesma porta dos gizmos, que cavalga as duas rotas (CPU: a corrente separada do sink; placa:
//! a leitura sem espera, `motion_bridge_gpu_tomadas.rs`) — e devolve, do último cozimento,
//! `id → P` no espaço do objecto (`Particulas`). A pilha leva-as ao mundo com o afim do objecto.
//!
//! | lei | porquê |
//! |---|---|
//! | o 1.º sink do objecto (pela ordem de `sinks`) com coluna `id` | dois emissores no mesmo objecto repetem ids; trocar de sink quando um id morre seria um salto |
//! | `id` é o ordinal de emissão `< ID_WRAP = 2²⁴` no `f32` da corrente | exacto: todo inteiro `< 2²⁴` cabe no `f32` |
//! | partícula ausente ⇒ sem entrada | a constraint fica sem alvo e o dono na pose dele |
//! | um quadro atrás | a pilha corre antes do cozimento do Motion (o Motion lê o mundo: um objecto preso a uma partícula que segue um objecto é um ciclo); na placa, mais um (a leitura sem espera) |

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::World;
use ph2d_ecs::motion_generator::{Particulas, ParticulasPedidas};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

use crate::motion_state::MotionState;

/// Os sinks cujas tomadas a pilha pede: os dos objectos pedidos.
#[must_use]
pub fn taps_for(motion: &MotionState) -> Vec<NodeId> {
    if motion.objeto.pinos.is_empty() {
        return Vec::new();
    }
    motion
        .sinks
        .iter()
        .copied()
        .filter(|&s| {
            motion
                .doc
                .owner_of(s)
                .is_some_and(|o| motion.objeto.pinos.contains(&o))
        })
        .collect()
}

/// `id → P` do 1.º sink (pela ordem dada) cuja corrente traz a coluna `id`.
#[must_use]
pub fn do_objeto<'a>(
    correntes: impl IntoIterator<Item = &'a Stream>,
) -> Option<BTreeMap<u32, [f64; 2]>> {
    let s = correntes
        .into_iter()
        .find(|s| matches!(s.get("id"), Some(Column::Scalar(_))))?;
    let Some(Column::Scalar(ids)) = s.get("id") else {
        return None;
    };
    let p = match s.get("P") {
        Some(Column::Vec2(p)) => p.as_slice(),
        _ => &[],
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o id é um inteiro ≥ 0 e < 2²⁴, exacto no f32"
    )]
    let por_id = ids.iter().enumerate().map(|(i, &id)| {
        let q = p.get(i).copied().unwrap_or([0.0, 0.0]);
        (id as u32, [f64::from(q[0]), f64::from(q[1])])
    });
    Some(por_id.collect())
}

/// ⭐ **A porta do quadro** — a shell chama-a ANTES da pilha: lê o pedido (arma as tomadas do
/// próximo cozimento) e publica as partículas do último.
pub fn publica(motion: &mut MotionState, world: &mut World) {
    let pedidas: BTreeSet<u64> = world
        .get_resource::<ParticulasPedidas>()
        .map(|p| p.0.iter().map(|s| s.0).collect())
        .unwrap_or_default();
    if pedidas != motion.objeto.pinos {
        // A tomada nova tem de sair mesmo com a cena parada: o tique repete-se pelo scrub.
        if pedidas.difference(&motion.objeto.pinos).next().is_some() {
            motion.pump.repete_o_tique();
        }
        motion.objeto.pinos = pedidas;
    }
    if motion.objeto.pinos.is_empty() {
        world.remove_resource::<Particulas>();
        return;
    }
    let tomadas = motion.pump.tap_streams();
    let mut por_objeto = BTreeMap::new();
    for &o in &motion.objeto.pinos {
        let correntes = motion
            .sinks
            .iter()
            .filter(|&&s| motion.doc.owner_of(s) == Some(o))
            .filter_map(|s| tomadas.iter().find(|(n, _)| n == s).map(|(_, c)| c));
        if let Some(m) = do_objeto(correntes) {
            por_objeto.insert(ph2d_ecs::StableId(o), m);
        }
    }
    world.insert_resource(Particulas(por_objeto));
}

#[cfg(test)]
#[path = "motion_particulas_tests.rs"]
mod tests;
