//! ⭐⭐ **PARA ONDE um agente vai neste tique** (plano 30 §3 passo 1, W3 + W6) — por nome, para um
//! ponto, para *a tag mais perto*, ou pela *patrulha*. Ver o cabeçalho de [`super`]. ⚠️ Módulo FILHO
//! de `nav.rs`: lê os campos privados da `NavWorld`.
//!
//! # A tag mais perto
//!
//! Quem pertence à tag (com a subárvore — a porta `ph2d_ecs::tags::tagged`, a da tabela de acções),
//! menos o próprio agente; o mais perto em LINHA RECTA (o *«nearest»* dos motores de referência),
//! empate pela ordem da identidade. ⚠️ A árvore das tags é do documento e chega à ponte por
//! [`PhysicsBridge::set_tag_tree`]; sem ela a tag não alcança ninguém (falha FECHADO).
//!
//! # A patrulha
//!
//! O agente visita os pontos de uma forma desenhada, por ordem: **fechada dá voltas, aberta vai e
//! volta**. Os pontos chegam DERIVADOS ao [`NavRoute`] do agente (a família lê a curva cozida — a
//! geometria não entra no ECS). O ponto em curso avança quando o agente chega a ele à
//! `arrive_distance` — a MESMA régua com que a condução diria «chegou», logo ele nunca trava num
//! ponto do meio e o `On Arrived` não fala durante a ronda. Ao entrar na ronda ele junta-se pelo
//! ponto mais perto. ⚠️ O ponto em curso é estado da corrida e entra no anel.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_nav::V2;

use crate::bridge::PhysicsBridge;
use crate::components::{NavRoute, NavTarget};

/// ⭐ **Onde vai a ronda de um agente** — o ponto em curso e o sentido (numa forma aberta ele volta).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ronda {
    /// O índice do ponto para onde ele vai, no [`NavRoute`].
    pub ponto: u32,
    /// `true` = a voltar (só numa forma aberta).
    pub volta: bool,
}

/// Os candidatos de cada tag neste tique, com a posição — consultados UMA vez por tag.
pub(super) type PorTag = BTreeMap<u64, Vec<(Entity, V2)>>;

impl PhysicsBridge {
    /// ⭐ **A árvore das tags do documento**, para o alvo *a tag mais perto*. A shell entrega-a a
    /// cada quadro; só se copia quando muda.
    pub fn set_tag_tree(&mut self, tree: &ph2d_tags::TagTree) {
        if self.nav.arvore != *tree {
            self.nav.arvore = tree.clone();
        }
    }

    /// **A ronda de um agente, agora** — `None` fora da patrulha ou antes do 1.º tique dela.
    #[must_use]
    pub fn nav_ronda(&self, agente: Entity) -> Option<Ronda> {
        self.nav.rondas.get(&agente).copied()
    }

    /// ⭐ **Quem (se é alguém) e onde** o agente persegue neste tique. Ver o cabeçalho.
    pub(super) fn alvo_de(
        &mut self,
        sim: &SimWorld,
        agente: Entity,
        target: NavTarget,
        pos: V2,
        chegada: f64,
        por_tag: &mut PorTag,
    ) -> (Option<Entity>, Option<V2>) {
        if !matches!(target, NavTarget::Patrol(_)) {
            self.nav.rondas.remove(&agente);
        }
        match target {
            NavTarget::None => (None, None),
            NavTarget::Point(q) => (None, Some([f64::from(q[0]), f64::from(q[1])])),
            NavTarget::Named(id) => {
                let quem = self.entidade_do_alvo(sim, id);
                (quem, quem.and_then(|e| self.posicao_de(sim, e)))
            }
            NavTarget::NearestTagged(tag) => {
                let candidatos = por_tag.entry(tag).or_insert_with(|| {
                    ph2d_ecs::tags::tagged(sim.world(), &self.nav.arvore, ph2d_tags::TagId(tag))
                        .into_iter()
                        .filter_map(|e| self.posicao_de(sim, e).map(|p| (e, p)))
                        .collect()
                });
                let mais_perto = candidatos
                    .iter()
                    .filter(|(e, _)| *e != agente)
                    .map(|&(e, p)| (dist2(p, pos), e, p))
                    .min_by(|a, b| a.0.total_cmp(&b.0));
                (mais_perto.map(|m| m.1), mais_perto.map(|m| m.2))
            }
            NavTarget::Patrol(_) => {
                let Some(rota) = sim.world().get::<NavRoute>(agente) else {
                    self.nav.rondas.remove(&agente);
                    return (None, None);
                };
                let pontos: Vec<V2> = rota
                    .points
                    .iter()
                    .map(|q| [f64::from(q[0]), f64::from(q[1])])
                    .collect();
                if pontos.is_empty() {
                    self.nav.rondas.remove(&agente);
                    return (None, None);
                }
                let ronda = self.nav.rondas.entry(agente).or_insert_with(|| Ronda {
                    ponto: mais_perto_de(&pontos, pos),
                    volta: false,
                });
                avanca(ronda, &pontos, rota.closed, pos, chegada);
                (None, Some(pontos[ronda.ponto as usize]))
            }
        }
    }
}

/// Avança a ronda pelos pontos a que o agente JÁ chegou (no máximo uma volta inteira por tique).
fn avanca(r: &mut Ronda, pontos: &[V2], fechada: bool, pos: V2, chegada: f64) {
    let n = pontos.len() as u32;
    r.ponto = r.ponto.min(n - 1);
    if n < 2 {
        return;
    }
    for _ in 0..n {
        if dist2(pontos[r.ponto as usize], pos) > chegada * chegada {
            return;
        }
        if fechada {
            r.ponto = (r.ponto + 1) % n;
        } else {
            if r.ponto == n - 1 {
                r.volta = true;
            } else if r.ponto == 0 {
                r.volta = false;
            }
            r.ponto = if r.volta { r.ponto - 1 } else { r.ponto + 1 };
        }
    }
}

/// O índice do ponto mais perto (empate: o primeiro).
fn mais_perto_de(pontos: &[V2], pos: V2) -> u32 {
    let mut melhor = (f64::INFINITY, 0u32);
    for (i, &p) in pontos.iter().enumerate() {
        let d = dist2(p, pos);
        if d < melhor.0 {
            melhor = (d, i as u32);
        }
    }
    melhor.1
}

fn dist2(a: V2, b: V2) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

#[cfg(test)]
#[path = "nav_alvo_tests.rs"]
mod tests;
