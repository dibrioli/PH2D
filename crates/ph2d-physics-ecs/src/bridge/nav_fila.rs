//! ⭐ (W9) **A FILA DO REPLANEIO na ponte** — a lei é [`ph2d_nav::refresh`]; aqui decide-se, antes da
//! condução do tique, quem procura um caminho novo porque a malha mudou (plano 30 §17).
//!
//! ⚠️ Só se lê estado que entra no anel (o [`ph2d_nav::AgentRuntime`] de cada agente: `owed`,
//! `broken`, `last_work`) e a ordem das entidades: um replay serve a MESMA fila no mesmo tique.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::Entity;
use ph2d_nav::refresh::{Owed, path_still_walkable, reparte};
use ph2d_nav::{AgentRuntime, Planeado, Plano, Query};
use rayon::prelude::*;

use super::{ChaveMalha, NavWorld, Pedido};
use crate::bridge::PhysicsBridge;

/// ⭐ **O orçamento de trabalho da procura por tique** para os caminhos que a fila deve, em nós
/// uniformes ([`ph2d_nav::Stats::work`], W14 — um nó da ponderada pesa o que custa) — o recurso é o
/// TEMPO do tique (`~110 ns` por nó uniforme ⇒ `~2 ms`). Medido
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
/// ⇒ abaixo de `20 000` o tique quase não desce e a espera dobra. (W14, load `~2`) Uma unidade de trabalho
/// vale `85–123 ns` em todas as procuras (`~114`) ⇒ `20 000` é `~2,3 ms` de procura por tique; com muita lama
/// o tique continua acima — UMA procura passa o orçamento sozinha, e os replaneios fora da fila não contam
/// (plano 30 §22.8). ⭐ (W15) Os dois deixaram de ser verdade: TODA procura paga daqui, e a que não cabe
/// pára a meio e continua no tique seguinte (§23).
pub(super) const ORCAMENTO_DE_TRABALHO_POR_TIQUE: u64 = 20_000;

/// ⭐ (W15) **As procuras A MEIO que avançam juntas num tique**, cada uma com uma fatia do orçamento, em
/// paralelo (ADR-0180, plano 30 §23.4) — o recurso é os NÚCLEOS: com eles, o relógio do tique paga uma
/// fatia e não todas. ⚠️ É um número FIXO e não o dos núcleos da máquina: quem avança e quanto decide-se
/// antes de as correr, logo o resultado é o mesmo com qualquer número de threads (na web, uma: o tique
/// paga-as em série). Medido (`medir_replaneio`, `150` lamas, `200` agentes, o mínimo de 7 intercaladas):
///
/// | em paralelo | sem caminho no fim | pior tique depois da porta (50 · 200 agentes) | crítico (máx) |
/// |---|---|---|---|
/// | `0` (série) | `189` | `8,5 · 13,5 ms` | `76 000` |
/// | `8` | `0` | `11,1 · 14,4` | `80 000` |
/// | **`16`** | **`0`** | **`7,5 · 12,9`** | **`40 000`** |
pub(super) const PROCURAS_EM_PARALELO: usize = 16;

/// Um agente que a ponte conduz neste tique: o pedido, a velocidade do mover, onde está, o raio e a
/// malha que pede (`None` fora de toda região). (W18: mudou-se para aqui com o tecto de LOC do `nav.rs`.)
pub(super) struct Vez {
    pub(super) p: Pedido,
    /// O alvo que vale neste tique: o de uma ordem `Start` com nome, ou o autorado.
    pub(super) target: crate::components::NavTarget,
    pub(super) speed: f64,
    pub(super) pos: ph2d_nav::V2,
    pub(super) raio: f32,
    pub(super) chave: Option<ChaveMalha>,
}

/// ⭐ **As alavancas da sonda e dos gates** — escolhidas em EXECUÇÃO, para que as versões se meçam no
/// MESMO processo (a régua do dono de 05/10). Por omissão, o produto.
pub(super) struct Sonda {
    /// (W15) `false` = toda procura inteira, no tique em que é pedida — [`PhysicsBridge::set_nav_slices`].
    pub(super) fatias: bool,
    /// (W15) Quantas procuras a meio avançam em paralelo num tique — [`PROCURAS_EM_PARALELO`].
    pub(super) paralelas: usize,
    /// (plano 30 §25, B) `false` = o caminho não contorna os corpos que andam — [`PhysicsBridge::set_nav_detour`].
    pub(super) contorno: bool,
    /// (plano 30 §25, C2) O alvo à vista dispensa a procura — [`ph2d_nav::agent::a_vista`].
    pub(super) a_vista: bool,
}

impl Default for Sonda {
    fn default() -> Self {
        Self {
            fatias: true,
            paralelas: PROCURAS_EM_PARALELO,
            contorno: true,
            a_vista: true,
        }
    }
}

/// FNV-1a de 64 bits, por palavra — a assinatura só precisa de distinguir.
struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    fn u64(&mut self, x: u64) {
        for b in x.to_le_bytes() {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
        }
    }
}

impl PhysicsBridge {
    /// Marca na fila os agentes cuja malha MUDOU (o caminho que ainda se anda fica; o partido passa à
    /// frente), envelhece quem lá está, e devolve a RESERVA de trabalho de quem a fila serve neste tique
    /// (a estimativa de cada um: a última procura dele) — a condução refaz-lhe o caminho com ela.
    pub(super) fn fila_do_replaneio(
        &mut self,
        vez: &[Vez],
        mudou: &BTreeSet<ChaveMalha>,
        orcamento: u64,
    ) -> BTreeMap<Entity, u64> {
        // Pela ordem das ENTIDADES (a do `vez` é a da consulta ao mundo): o `id` de cada um é a ordem
        // dele aqui.
        let por_entidade: BTreeMap<Entity, &Vez> = vez.iter().map(|v| (v.p.entity, v)).collect();
        let mut fila: Vec<Owed> = Vec::new();
        let mut quem: BTreeMap<u64, Entity> = BTreeMap::new();
        for (i, (&e, v)) in por_entidade.iter().enumerate() {
            let tiled = v.chave.and_then(|k| self.nav.meshes.get(&k));
            let malha = tiled.map(ph2d_navmesh::TiledMesh::mesh);
            // Só os troços que tocam os mosaicos refeitos se percorrem (a lei, `refresh`) — salvo quando a
            // malha de onde a actualização partiu é de OUTRA corrida (um scrub): então todos.
            let onde = tiled
                .filter(|_| v.chave.is_none_or(|k| !self.nav.sem_zona.contains(&k)))
                .and_then(ph2d_navmesh::TiledMesh::changed_area);
            let Some(rt) = self.nav.agents.get_mut(&e) else {
                continue;
            };
            if v.chave.is_some_and(|k| mudou.contains(&k)) && !rt.path.is_empty() {
                rt.broken |=
                    !malha.is_some_and(|m| path_still_walkable(m, rt, v.pos, onde.as_deref()));
                rt.owed = rt.owed.max(1);
            }
            // (Quem tem uma procura a meio avança no passo em paralelo, [`Self::procuras_a_meio`].)
            if rt.owed > 0 && rt.a_meio.is_none() {
                fila.push(Owed {
                    id: i as u64,
                    broken: rt.broken,
                    ticks: rt.owed,
                    work: rt.last_work,
                });
                quem.insert(i as u64, e);
            }
        }
        let reservas = reparte(&mut fila, orcamento);
        let mut servir = BTreeMap::new();
        for (o, r) in fila.iter().zip(reservas) {
            let e = quem[&o.id];
            if r > 0 {
                servir.insert(e, r);
            } else if let Some(rt) = self.nav.agents.get_mut(&e) {
                rt.owed = rt.owed.saturating_add(1);
            }
        }
        servir
    }

    /// ⭐⭐ (W15) **As procuras A MEIO avançam em PARALELO**, antes da condução: as primeiras
    /// [`PROCURAS_EM_PARALELO`] da fila (a mais ADIANTADA primeiro — acabá-las é o que não deixa
    /// acumular procuras a meio, nem revezá-las sem que nenhuma acabe —, depois os partidos, quem espera
    /// há mais tiques, e a ordem das entidades), cada uma com uma fatia do orçamento inteiro; as outras
    /// envelhecem. Devolve a
    /// resposta de cada uma que acabou (a condução instala-a), o trabalho gasto e a maior fatia (o que o
    /// relógio paga com núcleos que cheguem). ⚠️ Uma procura de outras entradas fica para a condução,
    /// que a recomeça.
    pub(super) fn procuras_a_meio(
        &mut self,
        vez: &[Vez],
        entradas: &BTreeMap<ChaveMalha, u64>,
        q: &Query<'_>,
    ) -> (BTreeMap<Entity, Planeado>, u64, u64) {
        let mut prontos = BTreeMap::new();
        if !self.nav.sonda.fatias {
            return (prontos, 0, 0);
        }
        let mut fila: Vec<((u64, bool, u32), Entity, ChaveMalha)> = vez
            .iter()
            .filter_map(|v| {
                let k = v.chave?;
                let rt = self.nav.agents.get(&v.p.entity)?;
                let a = rt.a_meio?;
                (entradas.get(&k) == Some(&a.entradas)).then_some((
                    (a.trabalho, rt.broken, rt.owed),
                    v.p.entity,
                    k,
                ))
            })
            .collect();
        fila.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let NavWorld {
            meshes,
            agents,
            planos,
            sonda,
            orcamento,
            ..
        } = &mut self.nav;
        let corre = sonda.paralelas.min(fila.len());
        for &(_, e, _) in &fila[corre..] {
            if let Some(rt) = agents.get_mut(&e) {
                rt.owed = rt.owed.saturating_add(1);
            }
        }
        let mut lote: Vec<(Entity, AgentRuntime, Option<Plano>, ChaveMalha)> = fila[..corre]
            .iter()
            .filter_map(|&(_, e, k)| Some((e, agents.remove(&e)?, planos.remove(&e), k)))
            .collect();
        let orc = *orcamento;
        let meshes = &*meshes;
        let feitas: Vec<(Option<Planeado>, u64)> = lote
            .par_iter_mut()
            .map(|(_, rt, plano, k)| {
                let Some(malha) = meshes.get(k) else {
                    return (None, 0);
                };
                let (r, w) = ph2d_nav::agent::advance_mid(rt, malha.mesh(), q, plano, orc);
                (r.map(|(r, _)| r), w)
            })
            .collect();
        let (mut gasto, mut maior) = (0u64, 0u64);
        for ((e, rt, plano, _), (r, w)) in lote.into_iter().zip(feitas) {
            gasto = gasto.saturating_add(w);
            maior = maior.max(w);
            agents.insert(e, rt);
            if let Some(p) = plano {
                planos.insert(e, p);
            }
            if let Some(r) = r {
                prontos.insert(e, r);
            }
        }
        (prontos, gasto, maior)
    }

    /// ⭐ (W15) **A assinatura das entradas da procura**, por malha: a chave, o conteúdo dela (a
    /// assinatura deste tique), os custos e os atalhos — uma procura a meio de outras entradas recomeça
    /// ([`ph2d_nav::agent::Vez::entradas`]). Só lê estado que o replay refaz igual. ⭐ (W18) E a dos
    /// custos e dos atalhos SÓ ([`ph2d_nav::agent::Vez::custos`]): mudou, quem anda replaneia.
    pub(super) fn entradas_das_procuras(
        &self,
        custos: &[f64],
        links: &[ph2d_nav::Link],
    ) -> (BTreeMap<ChaveMalha, u64>, u64) {
        let mut h = Fnv::default();
        custos.iter().for_each(|c| h.u64(c.to_bits()));
        for l in links {
            h.u64(u64::from(l.id));
            [l.from[0], l.from[1], l.to[0], l.to[1], l.cost]
                .iter()
                .for_each(|x| h.u64(x.to_bits()));
            h.u64(u64::from(l.two_way) | u64::from(l.teleport) << 1);
        }
        let por_malha = self
            .nav
            .sinais
            .iter()
            .map(|(&(regiao, raio, evita), &s)| {
                let mut g = Fnv(h.0);
                [regiao.to_bits(), u64::from(raio), evita, s]
                    .iter()
                    .for_each(|&x| g.u64(x));
                ((regiao, raio, evita), g.0)
            })
            .collect();
        (por_malha, h.0)
    }

    /// (W15) O CONTROLO da sonda e dos gates: `false` desliga a vez e as fatias — toda procura corre
    /// inteira no tique em que é pedida (só a fila da malha que muda espera). Por omissão, ligadas.
    pub fn set_nav_slices(&mut self, on: bool) {
        self.nav.sonda.fatias = on;
    }

    /// (plano 30 §25, C2) A sonda e o CONTROLO: `false` = o alvo à vista também espera pela procura.
    pub fn set_nav_sight(&mut self, on: bool) {
        self.nav.sonda.a_vista = on;
    }

    /// (plano 30 §25, B) A sonda e o CONTROLO: `false` = o caminho atravessa os corpos que andam (só o
    /// desvio os vê, como antes).
    pub fn set_nav_detour(&mut self, on: bool) {
        self.nav.sonda.contorno = on;
    }

    /// (W15) A sonda: quantas procuras a meio avançam em paralelo num tique (`0` = nenhuma: só a
    /// condução as avança, em série).
    pub fn set_nav_parallel(&mut self, n: usize) {
        self.nav.sonda.paralelas = n;
    }

    /// (W15) O trabalho de procura ([`ph2d_nav::Stats::work`]) gasto no último tique, todo.
    #[must_use]
    pub fn nav_search_work(&self) -> u64 {
        self.nav.gasto
    }

    /// (W15) O do caminho CRÍTICO do último tique: a maior fatia das procuras em paralelo mais o que a
    /// condução gastou em série — o que o relógio paga com núcleos que cheguem (o orçamento, mais um
    /// `pop`).
    #[must_use]
    pub fn nav_search_critical_work(&self) -> u64 {
        self.nav.critico
    }

    /// A sonda e o CONTROLO dos gates: outro orçamento de trabalho por tique (`u64::MAX` = todos no tique,
    /// o comportamento antes da fila).
    pub fn set_nav_replan_budget(&mut self, nos: u64) {
        self.nav.orcamento = nos;
    }
}
