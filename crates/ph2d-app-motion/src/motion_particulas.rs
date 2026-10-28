//! ⭐ **As PARTÍCULAS que a pilha de constraints prende** — o ponto de simulação
//! (`docs/BlenderConstraints/07_W6_pontas.md` §5 e §3).
//!
//! A constraint pede um objecto Motion e os `id` dele (`ParticulasPedidas`); o Motion
//! arma a TOMADA dos sinks dele — a mesma porta dos gizmos, que cavalga as duas rotas (CPU: a
//! corrente separada do sink; placa: a leitura sem espera, `motion_bridge_gpu_tomadas.rs`) — e
//! devolve, do último cozimento, `id → P` no espaço do objecto (`Particulas`), só dos ids pedidos.
//!
//! | lei | porquê |
//! |---|---|
//! | o 1.º sink do objecto (pela ordem de `sinks`) com coluna `id`; uma tomada que não chegou ANTES dele deixa a resposta desconhecida (sem alvo no quadro) | dois emissores no mesmo objecto repetem ids; seguir o sink que chegou e saltar quando o outro chega seria um salto |
//! | `id` sem `P` Vec2 ⇒ sem alvo | nunca a origem `[0, 0]` |
//! | `id` é o ordinal de emissão `< ID_WRAP = 2²⁴` no `f32` da corrente | exacto: todo inteiro `< 2²⁴` cabe no `f32` |
//! | partícula ausente ⇒ sem entrada | a constraint fica sem alvo e o dono na pose dele |
//! | publica-se 2× por quadro (antes da pilha e depois do cozimento); sem escrita nas tomadas, nada | a pilha relê o dono preso depois do cozimento (`depois_do_motion`): zero atraso na CPU; na placa um quadro (a leitura sem espera) |
//! | parada, uma tomada nova repete o tique; a tocar, o cozimento seguinte já a coze | repetir a tocar re-simulava o tique (um soluço) |

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::World;
use ph2d_ecs::motion_generator::{Particulas, ParticulasPedidas};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

use crate::motion_state::MotionState;

/// O estado da leitura entre quadros.
#[derive(Clone, Debug, Default)]
pub struct Leitura {
    /// A cena toca? (a shell escreve-o antes de [`publica`].)
    pub tocando: bool,
    /// O que a última publicação leu: a geração das tomadas e o pedido.
    vista: Option<(u64, ParticulasPedidas)>,
    /// Os sinks que só a pilha pede (nenhum gizmo nem sinal): a placa devolve-os em [`COLUNAS_DO_PINO`].
    pub(crate) so_colunas: Vec<NodeId>,
}

/// As colunas que o ponto de simulação lê de um sink.
pub(crate) const COLUNAS_DO_PINO: &[&str] = &["P", "id"];

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

/// `id → P` do 1.º sink (pela ordem dada) cuja corrente traz a coluna `id`, só dos `ids`. `None` na lista = a tomada desse sink não chegou: antes do que responde, a resposta é
/// desconhecida.
#[must_use]
pub fn do_objeto<'a>(
    correntes: impl IntoIterator<Item = Option<&'a Stream>>,
    ids: &BTreeSet<u32>,
) -> Option<BTreeMap<u32, [f64; 2]>> {
    for c in correntes {
        let s = c?;
        let Some(Column::Scalar(col)) = s.get("id") else {
            continue;
        };
        let Some(Column::Vec2(p)) = s.get("P") else {
            return None;
        };
        let mut m = BTreeMap::new();
        for (&id, q) in col.iter().zip(p) {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "o id é um inteiro ≥ 0 e < 2²⁴, exacto no f32"
            )]
            let id = id as u32;
            if ids.contains(&id) {
                m.insert(id, [f64::from(q[0]), f64::from(q[1])]);
                // Os ids são únicos na corrente: achados todos, o resto não se lê.
                if ids.len() == m.len() {
                    break;
                }
            }
        }
        return Some(m);
    }
    None
}

/// ⭐ **A porta do quadro** — a shell chama-a ANTES da pilha (arma as tomadas do próximo cozimento
/// e publica as do último) e DEPOIS do cozimento (as deste quadro). Devolve se republicou: sem
/// escrita nas tomadas nem pedido novo, não lê nada.
pub fn publica(motion: &mut MotionState, world: &mut World) -> bool {
    let pedido = world
        .get_resource::<ParticulasPedidas>()
        .cloned()
        .unwrap_or_default();
    let pedidas: BTreeSet<u64> = pedido.0.keys().map(|s| s.0).collect();
    if pedidas != motion.objeto.pinos {
        // Parada, a tomada nova tem de sair sem o relógio: o tique repete-se pelo scrub. A tocar, o
        // cozimento deste quadro já a coze.
        if !motion.particulas.tocando && pedidas.difference(&motion.objeto.pinos).next().is_some() {
            motion.pump.repete_o_tique();
        }
        motion.objeto.pinos = pedidas;
    }
    if motion.objeto.pinos.is_empty() {
        motion.particulas.vista = None;
        return world.remove_resource::<Particulas>().is_some();
    }
    let chave = (motion.pump.tomadas_geracao(), pedido);
    if world.contains_resource::<Particulas>() && motion.particulas.vista.as_ref() == Some(&chave) {
        return false;
    }
    let tomadas = motion.pump.tap_streams();
    let mut por_objeto = BTreeMap::new();
    for &o in &motion.objeto.pinos {
        let s = ph2d_ecs::StableId(o);
        let correntes = motion
            .sinks
            .iter()
            .filter(|&&n| motion.doc.owner_of(n) == Some(o))
            .map(|n| tomadas.iter().find(|(t, _)| t == n).map(|(_, c)| c));
        let Some(quer) = chave.1.0.get(&s) else {
            continue;
        };
        if let Some(m) = do_objeto(correntes, quer) {
            por_objeto.insert(s, m);
        }
    }
    motion.particulas.vista = Some(chave);
    world.insert_resource(Particulas(por_objeto));
    true
}

#[cfg(test)]
#[path = "motion_particulas_tests.rs"]
mod tests;
