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
//! - cada tique reparte um ORÇAMENTO de trabalho da procura pela fila, por ordem ([`reparte`]; em nós
//!   uniformes, [`crate::Stats::work`], W14; a estimativa de cada um é a última procura dele). ⭐ (W15)
//!   O último a caber leva o que sobra e a procura dele PÁRA a meio ([`crate::Plano`]) — continua no
//!   tique seguinte, à frente: nenhuma procura passa o orçamento sozinha.
//!
//! ⚠️ **Determinismo:** nada aqui lê um relógio. A ordem é a da fila (partidos, quem espera há mais
//! tiques, a ordem das entidades), a estimativa é uma contagem, e o estado (`owed`, `broken`,
//! `last_work`) vive no [`crate::AgentRuntime`], que entra no anel de checkpoints — o replay serve a
//! mesma fila no mesmo tique.

use crate::agent::AgentRuntime;
use crate::cost::segment_cost;
use crate::geom::{EPS, V2};
use crate::mesh::NavMesh;

/// O caminho que falta (de `pos` até ao fim) ainda se anda na malha `mesh`? Os troços de um ATALHO
/// não se andam (o teleporte salta; a porta de um sentido é um furo para quem não a atravessa).
///
/// ⭐ `onde`: os rectângulos onde a malha MUDOU (`None` = em toda a parte). Um troço cuja caixa não
/// toca nenhum deles atravessa a MESMA geometria de antes — andava, anda — e não se percorre. Medido
/// (`medir_replaneio`, 200 agentes, caminhos de `~100 m`): percorrer todos custava `12,4 ms` em cada
/// tique em que a malha mudava.
#[must_use]
pub fn path_still_walkable(
    mesh: &NavMesh,
    rt: &AgentRuntime,
    pos: V2,
    onde: Option<&[(V2, V2)]>,
) -> bool {
    if rt.next >= rt.path.len() {
        return false;
    }
    let toca = |a: V2, b: V2| {
        onde.is_none_or(|rs| {
            rs.iter().any(|&(lo, hi)| {
                a[0].min(b[0]) <= hi[0] + EPS
                    && a[0].max(b[0]) >= lo[0] - EPS
                    && a[1].min(b[1]) <= hi[1] + EPS
                    && a[1].max(b[1]) >= lo[1] - EPS
            })
        })
    };
    let mut a = pos;
    for i in rt.next..rt.path.len() {
        let b = rt.path[i];
        let atalho = i >= 1 && rt.hop_at(i - 1).is_some();
        if !atalho && toca(a, b) && segment_cost(mesh, &[], a, b).is_none() {
            return false;
        }
        a = b;
    }
    true
}

/// Um agente na fila: a ordem dele entre as entidades (`id`), se o caminho está PARTIDO, há quantos
/// tiques espera, e o trabalho da última procura dele.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owed {
    pub id: u64,
    pub broken: bool,
    pub ticks: u32,
    pub work: u64,
}

/// Ordena a fila (os partidos primeiro, depois quem espera há mais tiques, depois o `id`) e reparte
/// `budget` de trabalho pela frente dela: a cada um a estimativa dele (pelo menos `1`), e ao último o que
/// sobra. A reserva de cada um, pela ordem da fila (`0` = não é a vez dele). Nunca se salta à frente de
/// quem não coube (um barato atrás de um caro esperaria para sempre se o caro cedesse).
pub fn reparte(fila: &mut [Owed], budget: u64) -> Vec<u64> {
    fila.sort_by(|a, b| {
        b.broken
            .cmp(&a.broken)
            .then(b.ticks.cmp(&a.ticks))
            .then(a.id.cmp(&b.id))
    });
    let mut resta = budget;
    fila.iter()
        .map(|o| {
            let r = o.work.max(1).min(resta);
            resta -= r;
            r
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Owed, path_still_walkable, reparte};
    use crate::{AgentRuntime, NavMesh};

    /// Dois quadrados SEM ligação (um vão em `1 < x < 2`) e um caminho que os atravessa: partido —
    /// salvo quando a mudança foi longe dele, e então nem se percorre (é o que poupa o tique).
    #[test]
    fn so_os_trocos_que_tocam_a_mudanca_se_percorrem() {
        let m = NavMesh::from_polygons(
            vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [2.0, 0.0],
                [3.0, 0.0],
                [3.0, 1.0],
                [2.0, 1.0],
            ],
            vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7]],
        )
        .expect("dois quadrados");
        let mut rt = AgentRuntime::default();
        rt.path = vec![[0.5, 0.5], [2.5, 0.5]];
        rt.next = 1;
        let pos = [0.5, 0.5];
        assert!(
            !path_still_walkable(&m, &rt, pos, None),
            "sem zona: percorre e acha o vão"
        );
        let perto = [([1.2, 0.0], [1.8, 1.0])];
        assert!(
            !path_still_walkable(&m, &rt, pos, Some(&perto)),
            "a zona toca o troço"
        );
        let longe = [([10.0, 10.0], [11.0, 11.0])];
        assert!(
            path_still_walkable(&m, &rt, pos, Some(&longe)),
            "a zona longe: o troço não se percorre"
        );
    }

    fn o(id: u64, broken: bool, ticks: u32, work: u64) -> Owed {
        Owed {
            id,
            broken,
            ticks,
            work,
        }
    }

    #[test]
    fn a_fila_reparte_pelos_partidos_depois_os_mais_antigos_e_nunca_salta_a_frente() {
        let mut f = [
            o(0, false, 1, 10),
            o(1, false, 3, 10),
            o(2, true, 1, 10),
            o(3, false, 3, 10),
        ];
        assert_eq!(reparte(&mut f, 25), [10, 10, 5, 0]);
        assert_eq!(f.map(|x| x.id), [2, 1, 3, 0]);
        // Um caro acima do orçamento leva-o todo (a procura dele pára a meio).
        let mut g = [o(5, false, 1, 1_000)];
        assert_eq!(reparte(&mut g, 7), [7]);
        // Um barato atrás de um caro que não coube espera (a ordem não salta).
        let mut h = [o(0, false, 2, 10), o(1, false, 2, 1_000), o(2, false, 1, 1)];
        assert_eq!(reparte(&mut h, 100), [10, 90, 0]);
        // Sem estimativa (ainda não procurou) vale `1`.
        let mut z = [o(0, false, 1, 0), o(1, false, 1, 0)];
        assert_eq!(reparte(&mut z, 5), [1, 1]);
    }
}
