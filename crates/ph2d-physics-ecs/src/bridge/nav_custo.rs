//! ⭐⭐ (W7) **O QUE CUSTA, O QUE FERE, E OS ATALHOS** na ponte da navegação (plano 30 §4, W7). ⚠️
//! Módulo FILHO de `nav.rs`: lê os campos privados da `NavWorld`.
//!
//! # Três fontes, e a pergunta de cada agente: «que malha é a minha, e quanto custa cada área?»
//!
//! - **`NavCostArea` finita** → uma ÁREA da malha, igual para todos; o custo vai na CONSULTA
//!   (`ph2d_nav::Query`), logo mexer nele não refaz malha nenhuma.
//! - **`NavCostArea` proibida** → um FURO para todos.
//! - **Um `Damage` parado que MAGOA este agente** (`Damage::magoa`: a equipa, e o tipo que o
//!   `Health` dele sente) → um FURO só para ELE — a decisão do dono (plano 30 §11.1). Sem `Health`
//!   nada o magoa; `NavAgent::avoid_harm` desligado, também não.
//!
//! ⇒ a chave da malha ganha a ASSINATURA do conjunto de zonas que o agente evita: dois inimigos com
//! as mesmas resistências partilham a malha, o imune ao fogo tem outra.
//!
//! ⚠️ A forma é o colisor PRINCIPAL do corpo (sensor ou não), pela pose de um obstáculo: estático, ou
//! cinemático PARADO — uma zona que anda não recorta (a lei do W6).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_nav::Link;
use ph2d_navmesh::{Area, Shape};

use super::malha::forma;
use crate::bridge::PhysicsBridge;
use crate::components::{Damage, Health, NavCostArea, NavLink};

/// O que custa e o que fere NESTE tique.
pub(super) struct Custos {
    /// As áreas finitas, as mais CARAS primeiro (onde se sobrepõem manda a mais cara), ids `1..`.
    pub(super) areas: Vec<Area>,
    /// O custo de cada id (`tabela[0]` é o chão, `1`).
    pub(super) tabela: Vec<f64>,
    /// As proibidas: furos para todos.
    pub(super) proibidas: Vec<Shape>,
    /// As zonas que podem magoar: a entidade e a forma.
    pub(super) ferem: Vec<(Entity, Shape)>,
}

impl PhysicsBridge {
    /// As três fontes deste tique, pela ordem das entidades.
    pub(super) fn custos_deste_tique(&self, sim: &SimWorld) -> Custos {
        let world = sim.world();
        let movers = super::malha::quem_anda(sim);
        let mut finitas: Vec<(f32, Entity, Shape)> = Vec::new();
        let mut proibidas = Vec::new();
        let mut ferem = Vec::new();
        for (&e, b) in &self.bodies {
            let area = world.get::<NavCostArea>(e);
            let dano = world.get::<Damage>(e).filter(|_| !movers.contains(&e));
            if area.is_none() && dano.is_none() {
                continue;
            }
            let Some((d, rot)) = self.pose_de_obstaculo(b.kind, b.handle, &b.rest) else {
                continue;
            };
            let f = forma(&d, rot);
            if let Some(a) = area {
                if a.forbidden {
                    proibidas.push(f.clone());
                } else {
                    finitas.push((a.cost, e, f.clone()));
                }
            }
            if dano.is_some() {
                ferem.push((e, f));
            }
        }
        // A mais cara primeiro (manda onde se sobrepõem); o empate pela ordem das entidades.
        finitas.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut tabela = vec![1.0];
        let areas = finitas
            .into_iter()
            .enumerate()
            .map(|(i, (custo, _, shape))| {
                // ⚠️ O custo tem de ser > 0 para ser uma distância (a lei soma-o × comprimento).
                tabela.push(f64::from(custo).max(f64::from(f32::MIN_POSITIVE)));
                Area {
                    shape,
                    id: (i + 1) as u16,
                    dentro: custo < 1.0,
                }
            })
            .collect();
        Custos {
            areas,
            tabela,
            proibidas,
            ferem,
        }
    }

    /// As zonas (índices em `custos.ferem`) que MAGOAM este agente — vazio se ele não as evita.
    pub(super) fn zonas_que_evita(
        &self,
        sim: &SimWorld,
        agente: Entity,
        avoid_harm: bool,
        custos: &Custos,
    ) -> Vec<usize> {
        let world = sim.world();
        let Some(vida) = world.get::<Health>(agente).filter(|_| avoid_harm) else {
            return Vec::new();
        };
        custos
            .ferem
            .iter()
            .enumerate()
            .filter(|(_, (e, _))| {
                *e != agente && world.get::<Damage>(*e).is_some_and(|d| d.magoa(vida))
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Os atalhos deste tique (a entrada é a entidade; a saída, quem tem o nome), e quem é cada
    /// `id` (o índice da entidade — estável na sessão; um `rebuild` esquece os caminhos).
    pub(super) fn atalhos_deste_tique(&self, sim: &SimWorld) -> (Vec<Link>, BTreeMap<u32, Entity>) {
        let world = sim.world();
        let mut out = Vec::new();
        let mut quem = BTreeMap::new();
        let Some(mut q) = world.try_query::<(Entity, &NavLink)>() else {
            return (out, quem);
        };
        let mut todos: Vec<(Entity, NavLink)> =
            q.iter(world).map(|(e, l)| (e, l.clone())).collect();
        todos.sort_by_key(|(e, _)| *e);
        for (e, l) in todos {
            let Some(saida) = self.entidade_do_alvo(sim, l.to) else {
                continue;
            };
            let (Some(de), Some(para)) = (self.posicao_de(sim, e), self.posicao_de(sim, saida))
            else {
                continue;
            };
            out.push(Link {
                id: e.index_u32(),
                from: de,
                to: para,
                two_way: l.two_way,
                teleport: l.teleport,
                cost: f64::from(l.cost.max(0.0)),
            });
            quem.insert(e.index_u32(), e);
        }
        (out, quem)
    }
}

/// A assinatura de um conjunto de zonas evitadas (`0` = nenhuma): FNV-1a sobre os bits das entidades.
pub(super) fn assinatura(zonas: &[Entity]) -> u64 {
    if zonas.is_empty() {
        return 0;
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for e in zonas {
        for b in e.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h.max(1)
}
