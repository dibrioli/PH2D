//! **A PONTE da VIDA** (plano 28, W2) — a lei pura [`ph2d_health`] corrida contra o mundo, UMA vez
//! por tique, DEPOIS do passo.
//!
//! Desenho e medição que o decidiu: `docs/Components/28_plano_vida_e_dano.md` §8.
//!
//! # ⭐⭐⭐ Um golpe tem TRÊS fontes, e a vida guarda a SUA memória
//!
//! Um par `(quem bate, quem leva)` está a TOCAR neste tique se aparecer em qualquer um de:
//!
//! 1. o **contacto sólido** do solver (`tick_contacts`, a união dos sub-passos);
//! 2. a **sobreposição de um sensor** depois do passo ([`PhysicsBridge::sobreposicoes_de_sensor`]);
//! 3. ⭐ **o que o próprio MOVER bateu** neste tique (o campo `toques_do_mover` da ponte) — a
//!    metade que a sonda `mede_o_golpe_que_chega` obrigou: **uma bala nunca encosta no alvo** (o
//!    mover pára rente a ele), logo o solver não tem toque nenhum para reportar.
//!
//! ⛔ **A vida NÃO lê o `contact_events` nem o `trigger_events`:** esses são canais de ECRÃ e
//! re-baseiam em silêncio depois de um scrub. A vida tem de reproduzir num replay **exactamente** os
//! golpes da corrida, logo o *«quem tocava quem no tique anterior»* dela vai no [`HealthState`], que
//! vai no anel. De graça: no tique 0 essa memória está vazia, e quem **nasce** sobreposto leva o
//! golpe no 1.º tique.
//!
//! # ⚠️ A ordem dentro do tique
//!
//! `os relógios andam → o início do quadro da lei (regeneração, marcas, escudo) → os golpes deste
//! tique, fonte a fonte pela ordem das entidades → a memória do toque é substituída`
//!
//! # ⛔ O que esta ponte NÃO faz
//!
//! Apagar. Ela **anuncia** quem morreu e quem bateu e deve sair (o molde do `projectile_done`); o
//! dreno único da shell remove, e só quem nasceu numa corrida.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_health::{Regras, Vida};
use ph2d_physics::RigidBodyHandle;

use super::PhysicsBridge;
use crate::components::{Damage, Health, OnHit};

/// **O ESTADO VIVO de uma vida** — o que o anel de checkpoints guarda por entidade.
///
/// ⚠️ **Três metades, e é o TIPO que as mantém juntas:** a vida da lei, o gerador da esquiva e a
/// memória do toque. Guardar só a primeira faria um scrub devolver a vida certa com a sequência de
/// sorteios de outra corrida, ou com um `Began` a mais (a memória vazia lê todo toque em curso como
/// novo).
#[derive(Clone, Debug, PartialEq)]
pub struct HealthState {
    vida: Vida,
    /// O estado do splitmix64 da esquiva (um sorteio por golpe que passa a invencibilidade).
    rng: u64,
    /// Quem estava a tocar esta vida no tique ANTERIOR — um `Began` é *«agora e não antes»*.
    tocando: BTreeSet<Entity>,
}

impl HealthState {
    /// A vida da lei, para quem lê (o Inspector, a barra).
    #[must_use]
    pub fn vida(&self) -> &Vida {
        &self.vida
    }
}

/// **O que aconteceu a uma vida neste dispatch.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HealthEventKind {
    /// Levou dano na VIDA — `amount` é o que a vida perdeu (depois da armadura e do escudo).
    Damaged { amount: f64 },
    /// O golpe só tocou o ESCUDO.
    Shielded { amount: f64 },
    /// Esquivou (o golpe foi sorteado e falhou).
    Dodged,
    /// Morreu neste golpe (uma vez só: um morto não morre outra vez).
    Died,
}

/// **Um facto de vida** — quem o sofreu, quem o causou, e o quê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HealthEvent {
    /// Quem tem a [`Health`].
    pub target: Entity,
    /// Quem tem o [`Damage`].
    pub source: Entity,
    pub kind: HealthEventKind,
}

/// Um lado de um toque: a entidade que autorou a forma, e o corpo dela.
type Lado = (Entity, Entity);

impl PhysicsBridge {
    /// **Os factos de vida deste dispatch** (dano, escudo, esquiva, morte), pela ordem em que a lei
    /// os produziu. Vazio num replay: um scrub não é uma tempestade de golpes.
    #[must_use]
    pub fn health_events(&self) -> &[HealthEvent] {
        &self.health_events
    }

    /// **Quem BATEU e deve sair da cena** ([`OnHit::Vanish`]) neste dispatch.
    ///
    /// ⚠️ Quem lê isto **não pode apagar toda a gente** — uma bala posta à mão é documento. A porta
    /// é a `ph2d_ecs::is_transient`, no dreno da shell.
    #[must_use]
    pub fn damage_spent(&self) -> &[Entity] {
        &self.damage_spent
    }

    /// **A vida de uma entidade, agora** — `None` se ela não tem [`Health`] ou ainda não correu um
    /// tique (a barra e o Inspector leem a config enquanto isso).
    #[must_use]
    pub fn health_of(&self, entity: Entity) -> Option<&HealthState> {
        self.health_state.get(&entity)
    }

    /// Esquece os factos anunciados — o canal é do DISPATCH (o irmão do
    /// `discard_projectile_deaths`, e pela mesma razão: uma morte no 1.º tique de uma moldura que
    /// deve três não pode ser apagada pelo 2.º).
    pub(super) fn discard_health_events(&mut self) {
        self.health_events.clear();
        self.damage_spent.clear();
    }

    /// **Um tique das vidas.** Ver o cabeçalho do módulo.
    ///
    /// `publicar = false` no laço de REPLAY: o estado anda (é isso que faz um scrub devolver a vida
    /// exacta do tique), os factos não saem.
    pub(super) fn drive_health(&mut self, sim: &SimWorld, publicar: bool) {
        let world = sim.world();
        // Quem tem vida — pela ordem determinística do `BTreeMap` de corpos.
        let alvos: Vec<(Entity, Health)> = self
            .bodies
            .keys()
            .filter_map(|&e| world.get::<Health>(e).map(|h| (e, h.clone())))
            .collect();
        if alvos.is_empty() {
            // ⚠️ Uma vida que perdeu o componente não deixa estado órfão no anel.
            self.health_state.clear();
            return;
        }
        let dt = f64::from(self.world.dt());
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }

        // ── 1. Quem está a TOCAR quem, neste tique (as três fontes) ─────────────
        let tem_dano = |e: Entity| world.get::<Damage>(e).is_some();
        let tem_vida = |e: Entity| world.get::<Health>(e).is_some();
        let mut toques: Vec<(Lado, Lado)> = Vec::new();
        let by_handle = self.handle_map();
        for key in self.world.tick_contacts().keys() {
            if let (Some(&a), Some(&b)) = (by_handle.get(&key.0), by_handle.get(&key.1)) {
                toques.push(((a, a), (b, b)));
            }
        }
        for (forma, corpo, dentro) in self.sobreposicoes_de_sensor() {
            toques.push(((forma, corpo), (dentro, dentro)));
        }
        for &(mover, bateu) in &self.toques_do_mover {
            if let Some(&b) = by_handle.get(&bateu.into_raw_parts()) {
                toques.push(((mover, mover), (b, b)));
            }
        }
        // `(fonte, alvo)` — cada lado é perguntado nos DOIS papéis: um contacto é simétrico, e um
        // inimigo que também magoa leva e dá no mesmo toque.
        let mut agora: BTreeMap<Entity, BTreeSet<Entity>> = BTreeMap::new();
        for (x, y) in toques {
            for (f, a) in [(x, y), (y, x)] {
                // A forma que o artista marcou primeiro (a peça-espada), e o corpo dela a seguir.
                let fonte = [f.0, f.1].into_iter().find(|&e| tem_dano(e));
                let alvo = [a.1, a.0].into_iter().find(|&e| tem_vida(e));
                if let (Some(fonte), Some(alvo)) = (fonte, alvo)
                    && fonte != alvo
                {
                    agora.entry(alvo).or_default().insert(fonte);
                }
            }
        }

        // ── 2. Cada vida anda o tique e leva os golpes dele ─────────────────────
        let dt_ms = dt * 1000.0;
        let mut vivos: BTreeMap<Entity, HealthState> = BTreeMap::new();
        let mut gastos: BTreeSet<Entity> = BTreeSet::new();
        for (alvo, h) in alvos {
            let cfg = h.config();
            let mut st = self.health_state.remove(&alvo).unwrap_or_else(|| {
                nascer(&h, &cfg, world.get::<ph2d_ecs::StableId>(alvo).map(|s| s.0))
            });
            st.vida.anda(dt_ms);
            st.vida.pre_quadro(&cfg, Regras::CASA, dt);
            let fontes = agora.remove(&alvo).unwrap_or_default();
            for &fonte in &fontes {
                let Some(dano) = world.get::<Damage>(fonte) else {
                    continue;
                };
                if !dano.fere(&h) {
                    continue;
                }
                let comecou = !st.tocando.contains(&fonte);
                let quanto = if dano.per_second {
                    f64::from(dano.amount) * dt
                } else if comecou {
                    f64::from(dano.amount)
                } else {
                    continue;
                };
                if comecou && dano.on_hit == OnHit::Vanish {
                    gastos.insert(fonte);
                }
                let antes = st.vida;
                let rng = &mut st.rng;
                st.vida.golpe(
                    &cfg,
                    Regras::CASA,
                    quanto,
                    !dano.ignores_shield,
                    !dano.ignores_armor,
                    &mut || sorteio(rng),
                );
                if publicar {
                    for kind in factos(&antes, &st.vida) {
                        self.health_events.push(HealthEvent {
                            target: alvo,
                            source: fonte,
                            kind,
                        });
                    }
                }
            }
            st.tocando = fontes;
            vivos.insert(alvo, st);
        }
        self.health_state = vivos;
        if publicar {
            self.damage_spent.extend(gastos);
        }
    }
}

/// **Nascer** — a vida do tique 0: o valor inicial, o escudo inicial, e o gerador semeado.
///
/// ⚠️ A semente mistura a do componente com a IDENTIDADE durável (`StableId`), que sobrevive a um
/// undo: dois inimigos iguais não esquivam em uníssono, e o mesmo inimigo esquiva igual em toda
/// corrida.
fn nascer(h: &Health, cfg: &ph2d_health::Config, id: Option<u64>) -> HealthState {
    let mut vida = Vida::nasce(f64::from(h.start), cfg);
    // ⛔ O `nasce` usa as regras do ALVO (é a criação dele), e o alvo aceita um inicial negativo;
    // a casa não, e esta é a única porta de escrita.
    vida.define(cfg, Regras::CASA, f64::from(h.start));
    if h.shield_start > 0.0 {
        vida.activa_escudo(cfg, f64::from(h.shield_start), true);
    }
    HealthState {
        vida,
        rng: h.seed ^ id.unwrap_or(0),
        tocando: BTreeSet::new(),
    }
}

/// Um número em `[0, 1)` do splitmix64 da casa — os 53 bits de cima, a mantissa exacta de um `f64`.
fn sorteio(rng: &mut u64) -> f64 {
    #[allow(clippy::cast_precision_loss)] // 53 bits cabem exactamente num `f64`
    let x = (ph2d_shake::baralha(rng) >> 11) as f64;
    x * (1.0 / 9_007_199_254_740_992.0)
}

/// Os factos que um golpe produziu, lidos da diferença entre o antes e o depois.
///
/// ⚠️ **Da diferença e não das marcas da lei**, porque as marcas duram o QUADRO: dois golpes no
/// mesmo tique (o 2.º ignorado pela invencibilidade) leriam a marca do 1.º outra vez.
fn factos(antes: &Vida, depois: &Vida) -> Vec<HealthEventKind> {
    let mut out = Vec::new();
    if !antes.acabou_de_esquivar && depois.acabou_de_esquivar {
        out.push(HealthEventKind::Dodged);
    }
    if depois.escudo < antes.escudo {
        out.push(HealthEventKind::Shielded {
            amount: antes.escudo - depois.escudo,
        });
    }
    if depois.pontos < antes.pontos {
        out.push(HealthEventKind::Damaged {
            amount: antes.pontos - depois.pontos,
        });
    }
    if !antes.morta() && depois.morta() {
        out.push(HealthEventKind::Died);
    }
    out
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;

/// A vida não guarda handles — o tipo vem aqui só para o campo da ponte ter nome.
pub(super) type ToqueDoMover = (Entity, RigidBodyHandle);
