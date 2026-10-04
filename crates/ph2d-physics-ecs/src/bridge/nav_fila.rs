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
/// | sem fila (todos no tique) | `9,9 · 29,1 · 85,0 ms` | `1` tique | `1` |
/// | `40 000` | `8,6 · 9,1 · 10,4` | `29` | `0` |
/// | **`20 000`** | **`5,7 · 6,5 · 10,1`** | **`56`** (`~1 s`) | **`4`** |
/// | `10 000` | `5,7 · 8,0 · 9,5` | `97` | `6` |
///
/// (Com a grelha das paredes do desvio e o caminho só percorrido onde a malha mudou; load `5–8`.)
/// ⇒ abaixo de `20 000` o tique quase não desce e a espera dobra.
pub(super) const ORCAMENTO_DE_NOS_POR_TIQUE: u64 = 20_000;

impl PhysicsBridge {
    /// Marca na fila os agentes cuja malha MUDOU (o caminho que ainda se anda fica; o partido passa à
    /// frente), envelhece quem lá está, e devolve quem a fila serve neste tique — a quem a condução
    /// esquece o caminho antes do passo — e os nós que essas procuras devem gastar (a estimativa de
    /// cada um: a última procura dele).
    pub(super) fn fila_do_replaneio(
        &mut self,
        vez: &[Vez],
        mudou: &BTreeSet<ChaveMalha>,
    ) -> (BTreeSet<Entity>, u64) {
        // Pela ordem das ENTIDADES (a do `vez` é a da consulta ao mundo): o `id` de cada um é a ordem
        // dele aqui.
        let por_entidade: BTreeMap<Entity, &Vez> = vez.iter().map(|v| (v.p.entity, v)).collect();
        let mut fila: Vec<Owed> = Vec::new();
        let mut quem: BTreeMap<u64, Entity> = BTreeMap::new();
        for (i, (&e, v)) in por_entidade.iter().enumerate() {
            let tiled = v.chave.and_then(|k| self.nav.meshes.get(&k));
            let malha = tiled.map(ph2d_navmesh::TiledMesh::mesh);
            // Só os troços que tocam os mosaicos refeitos se percorrem (a lei, `refresh`).
            let onde = tiled.and_then(ph2d_navmesh::TiledMesh::changed_area);
            let Some(rt) = self.nav.agents.get_mut(&e) else {
                continue;
            };
            if v.chave.is_some_and(|k| mudou.contains(&k)) && !rt.path.is_empty() {
                rt.broken |=
                    !malha.is_some_and(|m| path_still_walkable(m, rt, v.pos, onde.as_deref()));
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
        let mut gasto = 0u64;
        for (k, o) in fila.iter().enumerate() {
            let e = quem[&o.id];
            if k < n {
                servir.insert(e);
                gasto = gasto.saturating_add(o.nodes);
            } else if let Some(rt) = self.nav.agents.get_mut(&e) {
                rt.owed = rt.owed.saturating_add(1);
            }
        }
        (servir, gasto)
    }

    /// A sonda e o CONTROLO dos gates: outro orçamento de nós por tique (`u64::MAX` = todos no tique,
    /// o comportamento antes da fila).
    pub fn set_nav_replan_budget(&mut self, nos: u64) {
        self.nav.orcamento = nos;
    }
}
