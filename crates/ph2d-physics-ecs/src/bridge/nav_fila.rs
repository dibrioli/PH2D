//! ⭐ (W9) **A FILA DO REPLANEIO na ponte** — a lei é [`ph2d_nav::refresh`]; aqui decide-se, antes da
//! condução do tique, quem procura um caminho novo porque a malha mudou (plano 30 §17).
//!
//! ⚠️ Só se lê estado que entra no anel (o [`ph2d_nav::AgentRuntime`] de cada agente: `owed`,
//! `broken`, `last_nodes`) e a ordem das entidades: um replay serve a MESMA fila no mesmo tique.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::Entity;
use ph2d_nav::refresh::{Owed, path_still_walkable, serve};

use super::{ChaveMalha, Vez};
use crate::bridge::PhysicsBridge;

/// ⭐ **O orçamento de nós da procura por tique** para os caminhos que a fila deve — o recurso é o
/// TEMPO do tique (`~110 ns` por nó na procura uniforme, `~330` na ponderada ⇒ `~2–7 ms`). Medido
/// (`examples/medir_replaneio.rs`, `--release`, load `~3`, `100 × 100 m`, `1 000` caixas, a porta a
/// alternar seis vezes; o pior tique da janela que se segue, mediana):
///
/// | orçamento | pior tique a 10 · 50 · 200 agentes | a fila esvazia (200) | o último PARTIDO (200) |
/// |---|---|---|---|
/// | sem fila (todos no tique) | `8,8 · 24,2 · 85,6 ms` | `1` tique | `1` |
/// | `40 000` | `7,8 · 10,1 · 21,3` | `30` | `5` |
/// | **`20 000`** | **`5,7 · 8,3 · 18,1`** | **`58`** (`~1 s`) | **`9`** |
/// | `10 000` | `5,5 · 8,1 · 17,6` | `97` | `16` |
///
/// (Depois da grelha das paredes do desvio; o CONTROLO sem a porta é `0,4 · 2,0 · 3,8 ms`.) ⇒ abaixo
/// de `20 000` o tique quase não desce e a espera dobra.
pub(super) const ORCAMENTO_DE_NOS_POR_TIQUE: u64 = 20_000;

impl PhysicsBridge {
    /// Marca na fila os agentes cuja malha MUDOU (o caminho que ainda se anda fica; o partido passa à
    /// frente), envelhece quem lá está, e devolve quem a fila serve neste tique — a quem a condução
    /// esquece o caminho antes do passo.
    pub(super) fn fila_do_replaneio(
        &mut self,
        vez: &[Vez],
        mudou: &BTreeSet<ChaveMalha>,
    ) -> BTreeSet<Entity> {
        // Pela ordem das ENTIDADES (a do `vez` é a da consulta ao mundo): o `id` de cada um é a ordem
        // dele aqui.
        let por_entidade: BTreeMap<Entity, &Vez> = vez.iter().map(|v| (v.p.entity, v)).collect();
        let mut fila: Vec<Owed> = Vec::new();
        let mut quem: BTreeMap<u64, Entity> = BTreeMap::new();
        for (i, (&e, v)) in por_entidade.iter().enumerate() {
            let malha = v
                .chave
                .and_then(|k| self.nav.meshes.get(&k))
                .map(ph2d_navmesh::TiledMesh::mesh);
            let Some(rt) = self.nav.agents.get_mut(&e) else {
                continue;
            };
            if v.chave.is_some_and(|k| mudou.contains(&k)) && !rt.path.is_empty() {
                rt.broken |= !malha.is_some_and(|m| path_still_walkable(m, rt, v.pos));
                rt.owed = rt.owed.max(1);
            }
            if rt.owed > 0 {
                fila.push(Owed {
                    id: i as u64,
                    broken: rt.broken,
                    ticks: rt.owed,
                    nodes: rt.last_nodes,
                });
                quem.insert(i as u64, e);
            }
        }
        let n = serve(&mut fila, self.nav.orcamento);
        let mut servir = BTreeSet::new();
        for (k, o) in fila.iter().enumerate() {
            let e = quem[&o.id];
            if k < n {
                servir.insert(e);
            } else if let Some(rt) = self.nav.agents.get_mut(&e) {
                rt.owed = rt.owed.saturating_add(1);
            }
        }
        servir
    }

    /// A sonda e o CONTROLO dos gates: outro orçamento de nós por tique (`u64::MAX` = todos no tique,
    /// o comportamento antes da fila).
    pub fn set_nav_replan_budget(&mut self, nos: u64) {
        self.nav.orcamento = nos;
    }
}
