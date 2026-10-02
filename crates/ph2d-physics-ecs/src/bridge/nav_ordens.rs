//! ⭐⭐⭐ **As ORDENS de navegação** (plano 30, W6) — os verbos `Start Navigation` / `Stop Navigation`
//! a chegar aos agentes. Ver o cabeçalho de [`super`]. ⚠️ Módulo FILHO de `nav.rs`.
//!
//! # ⚠️ O idioma da VIDA, e porque
//!
//! A tabela de acções corre por QUADRO; a navegação anda por TIQUE. ⇒ o verbo ANUNCIA, a shell
//! entrega o pedido aqui ([`PhysicsBridge::pede_navegacao`]), e a ponte aplica-o no fim do próximo
//! tique VIVO **e grava-o na fita por tique** — num replay lê-o da fita. É o caminho do
//! `pede_vida`, e pela mesma razão: um pedido aplicado por quadro seria re-aplicado ou esquecido
//! num scrub.
//!
//! # ⛔ A ordem é CORRIDA, nunca documento
//!
//! Ela vive na ponte (no anel de checkpoints, como a memória do agente) e nunca toca no
//! `NavAgent`: o que a corrida escreve a corrida desfaz (plano 30 §10.7) — um recomeço ou um
//! `Ctrl+Z` devolvem o agente ao que o artista autorou, sem um passo de undo por ordem.

use std::collections::BTreeMap;

use ph2d_ecs::Entity;

use crate::bridge::PhysicsBridge;

/// ⭐⭐ **Um pedido de navegação da tabela de acções** (plano 30, W6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PedidoDeNavegacao {
    /// Anda — atrás de quem tem este `stable_name_id`, ou do alvo autorado com `0` (a convenção de
    /// «ninguém» desta casa).
    Anda(u64),
    /// Pára onde está.
    Para,
}

/// **A ordem em vigor num agente** — o que os verbos lhe disseram nesta corrida.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrdemDeNavegacao {
    /// `None` = nenhum verbo falou (vale o *Active* autorado); `Some(true)` = um `Start`;
    /// `Some(false)` = um `Stop`.
    pub ligado: Option<bool>,
    /// O alvo que um `Start` com nome lhe deu (`0` = o autorado).
    pub alvo: u64,
}

/// O estado das ordens na ponte: a fila de pedidos à espera do tique, a fita e as ordens em vigor.
#[derive(Default)]
pub(in crate::bridge) struct Ordens {
    pub(super) pedidos: Vec<(Entity, PedidoDeNavegacao)>,
    pub(super) fita: BTreeMap<u64, Vec<(Entity, PedidoDeNavegacao)>>,
    /// ⚠️ **Entra no anel** pelo [`super::super::tape::ControllerMemory`].
    pub(in crate::bridge) em_vigor: BTreeMap<Entity, OrdemDeNavegacao>,
}

impl PhysicsBridge {
    /// ⭐⭐ **Um pedido de navegação para o PRÓXIMO tique** — a porta pela qual a tabela de acções
    /// chega aos agentes. ⚠️ Não age agora: num relógio parado ele espera (ver o cabeçalho).
    pub fn pede_navegacao(&mut self, alvo: Entity, pedido: PedidoDeNavegacao) {
        self.nav.ordens.pedidos.push((alvo, pedido));
    }

    /// **A ordem em vigor num agente** — `None` se nenhum verbo lhe falou nesta corrida.
    #[must_use]
    pub fn nav_ordem(&self, agente: Entity) -> Option<OrdemDeNavegacao> {
        self.nav.ordens.em_vigor.get(&agente).copied()
    }

    /// **As ordens deste tique**, depois do passo: da fila num tique VIVO (que as grava no tique
    /// dele, sobrescrevendo — a regra da fita da vida), da fita num replay.
    pub(in crate::bridge) fn aplica_ordens_de_navegacao(&mut self, publicar: bool, tick: u64) {
        let o = &mut self.nav.ordens;
        let pedidos = if publicar {
            let fila = std::mem::take(&mut o.pedidos);
            if fila.is_empty() {
                o.fita.remove(&tick);
            } else {
                o.fita.insert(tick, fila.clone());
            }
            fila
        } else {
            o.fita.get(&tick).cloned().unwrap_or_default()
        };
        for (e, p) in pedidos {
            let ordem = o.em_vigor.entry(e).or_default();
            match p {
                PedidoDeNavegacao::Anda(alvo) => {
                    ordem.ligado = Some(true);
                    ordem.alvo = alvo;
                }
                PedidoDeNavegacao::Para => ordem.ligado = Some(false),
            }
        }
    }
}
