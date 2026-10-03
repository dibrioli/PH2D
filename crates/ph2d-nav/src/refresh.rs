//! ⭐ (W9) **A FILA DO REPLANEIO** — quando a malha muda, quem replaneia NESTE tique (plano 30 §17).
//!
//! Medido (`examples/medir_replaneio.rs` da ponte, `100 × 100 m`, `1 000` caixas): sem fila, uma porta
//! que muda põe TODOS os agentes da malha a procurar no mesmo tique — `200` procuras, um tique de
//! `126 ms` contra `36` do mesmo tique sem a porta. Com a fila:
//!
//! - quem tem o caminho que falta AINDA andável na malha nova ([`path_still_walkable`]) continua a
//!   andá-lo e entra na fila (o caminho novo pode ser mais curto — uma porta que ABRIU —, mas o velho
//!   não atravessa nada);
//! - quem o tem PARTIDO entra à frente (continua a andar o que tem enquanto espera — um ou dois
//!   tiques —, e o corpo não atravessa a parede: é a física);
//! - cada tique serve a fila por ordem ([`serve`]) até um ORÇAMENTO em nós da procura (a estimativa
//!   de cada um é a última procura dele) — sempre pelo menos um.
//!
//! ⚠️ **Determinismo:** nada aqui lê um relógio. A ordem é a da fila (partidos, quem espera há mais
//! tiques, a ordem das entidades), a estimativa é uma contagem de nós, e o estado (`owed`, `broken`,
//! `last_nodes`) vive no [`crate::AgentRuntime`], que entra no anel de checkpoints — o replay serve a
//! mesma fila no mesmo tique.

use crate::agent::AgentRuntime;
use crate::cost::segment_cost;
use crate::geom::V2;
use crate::mesh::NavMesh;

/// O caminho que falta (de `pos` até ao fim) ainda se anda na malha `mesh`? Os troços de um ATALHO
/// não se andam (o teleporte salta; a porta de um sentido é um furo para quem não a atravessa).
#[must_use]
pub fn path_still_walkable(mesh: &NavMesh, rt: &AgentRuntime, pos: V2) -> bool {
    if rt.next >= rt.path.len() {
        return false;
    }
    let mut a = pos;
    for i in rt.next..rt.path.len() {
        let b = rt.path[i];
        let atalho = i >= 1 && rt.hop_at(i - 1).is_some();
        if !atalho && segment_cost(mesh, &[], a, b).is_none() {
            return false;
        }
        a = b;
    }
    true
}

/// Um agente na fila: a ordem dele entre as entidades (`id`), se o caminho está PARTIDO, há quantos
/// tiques espera, e quantos nós a última procura dele expandiu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owed {
    pub id: u64,
    pub broken: bool,
    pub ticks: u32,
    pub nodes: u64,
}

/// Ordena a fila (os partidos primeiro, depois quem espera há mais tiques, depois o `id`) e devolve
/// quantos da frente se servem neste tique com `budget` nós — sempre pelo menos um, e nunca se salta
/// à frente de quem não coube (um barato atrás de um caro esperaria para sempre se o caro cedesse).
pub fn serve(fila: &mut [Owed], budget: u64) -> usize {
    fila.sort_by(|a, b| {
        b.broken
            .cmp(&a.broken)
            .then(b.ticks.cmp(&a.ticks))
            .then(a.id.cmp(&b.id))
    });
    let mut gasto = 0u64;
    let mut n = 0;
    for o in fila.iter() {
        if n > 0 && gasto.saturating_add(o.nodes) > budget {
            break;
        }
        gasto = gasto.saturating_add(o.nodes);
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::{Owed, serve};

    fn o(id: u64, broken: bool, ticks: u32, nodes: u64) -> Owed {
        Owed {
            id,
            broken,
            ticks,
            nodes,
        }
    }

    #[test]
    fn a_fila_serve_os_partidos_depois_os_mais_antigos_e_nunca_salta_a_frente() {
        let mut f = [
            o(0, false, 1, 10),
            o(1, false, 3, 10),
            o(2, true, 1, 10),
            o(3, false, 3, 10),
        ];
        assert_eq!(serve(&mut f, 25), 2);
        assert_eq!(f.map(|x| x.id), [2, 1, 3, 0]);
        // Sempre pelo menos um, mesmo acima do orçamento.
        let mut g = [o(5, false, 1, 1_000)];
        assert_eq!(serve(&mut g, 1), 1);
        // Um barato atrás de um caro que não coube espera (a ordem não salta).
        let mut h = [o(0, false, 2, 10), o(1, false, 2, 1_000), o(2, false, 1, 1)];
        assert_eq!(serve(&mut h, 100), 1);
    }
}
