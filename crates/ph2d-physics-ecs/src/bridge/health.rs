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
    /// ⭐ **Foi curada** (plano 28, W2b) — `amount` é o que a vida GANHOU (depois do tecto).
    ///
    /// ⚠️ **APENDADO** e o primeiro produtor do sinal `On Heal`: até aqui aquele campo da [`Health`]
    /// não tinha quem o acendesse.
    Healed { amount: f64 },
}

/// ⭐⭐ **Um pedido de vida da tabela de acções** (plano 28, W2b) — os verbos `Damage`/`Heal`.
///
/// ⚠️ **Só números finitos `> 0` chegam aqui** — a porta que o constrói (a tabela) recusa o resto
/// como INERTE; a ponte volta a conferir, porque a fita pode vir de outro caminho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PedidoDeVida {
    /// Tira `f64` pontos, pelo pipeline inteiro da lei (invencibilidade · esquiva · armadura ·
    /// escudo), como o `Hit` do oráculo com os dois interruptores ligados.
    Dano(f64),
    /// Devolve `f64` pontos (um morto não é curado — a regra da casa).
    Cura(f64),
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

    /// ⭐⭐ **Um pedido de vida para o PRÓXIMO tique** (plano 28, W2b) — a porta pela qual a tabela
    /// de acções chega à vida.
    ///
    /// ⚠️ **Ele não age agora:** fica à espera do próximo tique VIVO, que o grava na fita por tique
    /// e o aplica. Num relógio parado ele espera — *um golpe da tabela não pode acontecer fora do
    /// tempo da corrida*, senão um scrub não o saberia refazer.
    pub fn pede_vida(&mut self, alvo: Entity, pedido: PedidoDeVida) {
        let valido = match pedido {
            PedidoDeVida::Dano(x) | PedidoDeVida::Cura(x) => x.is_finite() && x > 0.0,
        };
        if valido {
            self.pedidos_de_vida.push((alvo, pedido));
        }
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
    ///
    /// ⭐⭐ **Os pedidos da tabela** (W2b) saem da fila num tique VIVO — que os grava no tique dele,
    /// **sobrescrevendo** o que lá estava — e da FITA num replay.
    ///
    /// ⚠️ **Sobrescreve tique a tique e não corta o futuro**, e é a regra da irmã
    /// ([`super::tape::InputTape::record`]): *o artista que scrubba para trás e toca de novo está a
    /// autorar por cima*. Com duas regras diferentes, a fita do dedo e a da vida descreveriam duas
    /// corridas depois do mesmo gesto.
    pub(super) fn drive_health(&mut self, sim: &SimWorld, publicar: bool, tick: u64) {
        let pedidos: Vec<(Entity, PedidoDeVida)> = if publicar {
            let fila = std::mem::take(&mut self.pedidos_de_vida);
            if fila.is_empty() {
                self.fita_da_vida.remove(&tick);
            } else {
                self.fita_da_vida.insert(tick, fila.clone());
            }
            fila
        } else {
            self.fita_da_vida.get(&tick).cloned().unwrap_or_default()
        };
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
        // ⭐ O 3.º membro de cada toque é a NORMAL do contacto (plano 28, W5), a apontar do 1.º lado
        // para o 2.º — só o solver a tem; um sensor e um mover chegam sem ela.
        let mut toques: Vec<(Lado, Lado, Option<[f32; 2]>)> = Vec::new();
        let by_handle = self.handle_map();
        for (key, amostra) in self.world.tick_contacts() {
            if let (Some(&a), Some(&b)) = (by_handle.get(&key.0), by_handle.get(&key.1)) {
                // ⚠️ A chave é o par com o handle MENOR primeiro, e a normal aponta do `body1` para o
                // `body2` — a mesma ordem (`PeakSample::normal`), logo ela aponta de `a` para `b`.
                toques.push(((a, a), (b, b), Some(amostra.normal)));
            }
        }
        for (forma, corpo, dentro) in self.sobreposicoes_de_sensor() {
            toques.push(((forma, corpo), (dentro, dentro), None));
        }
        for &(mover, bateu) in &self.toques_do_mover {
            if let Some(&b) = by_handle.get(&bateu.into_raw_parts()) {
                toques.push(((mover, mover), (b, b), None));
            }
        }
        // `(fonte, alvo)` — cada lado é perguntado nos DOIS papéis: um contacto é simétrico, e um
        // inimigo que também magoa leva e dá no mesmo toque.
        let mut agora: BTreeMap<Entity, BTreeMap<Entity, Toque>> = BTreeMap::new();
        for (x, y, normal) in toques {
            for (f, a, n) in [(x, y, normal), (y, x, normal.map(|[nx, ny]| [-nx, -ny]))] {
                // A forma que o artista marcou primeiro (a peça-espada), e o corpo dela a seguir.
                let fonte = [f.0, f.1].into_iter().find(|&e| tem_dano(e));
                let alvo = [a.1, a.0].into_iter().find(|&e| tem_vida(e));
                if let (Some(fonte), Some(alvo)) = (fonte, alvo)
                    && fonte != alvo
                {
                    // ⚠️ O PRIMEIRO toque do par dá a direcção — o solver vem à frente (é o único
                    // com normal), e a ordem de tudo aqui é a do `BTreeMap`: determinística.
                    let direccao = n.or_else(|| self.do_centro_ao_centro(f.1, a.1));
                    agora
                        .entry(alvo)
                        .or_default()
                        .entry(fonte)
                        .or_insert(Toque {
                            direccao: direccao.and_then(unitario),
                            corpo: a.1,
                        });
                }
            }
        }

        // ── 2. Cada vida anda o tique e leva os golpes dele ─────────────────────
        let dt_ms = dt * 1000.0;
        let mut vivos: BTreeMap<Entity, HealthState> = BTreeMap::new();
        let mut gastos: BTreeSet<Entity> = BTreeSet::new();
        // Os EMPURRÕES deste tique — aplicados DEPOIS de todas as vidas andarem, para que uma vida
        // não leia um corpo que outra já empurrou (a ordem do laço deixaria de ser irrelevante).
        let mut empurroes: Vec<(Entity, [f32; 2])> = Vec::new();
        for (alvo, h) in alvos {
            let cfg = h.config();
            let mut st = self.health_state.remove(&alvo).unwrap_or_else(|| {
                nascer(&h, &cfg, world.get::<ph2d_ecs::StableId>(alvo).map(|s| s.0))
            });
            st.vida.anda(dt_ms);
            st.vida.pre_quadro(&cfg, Regras::CASA, dt);
            // ⭐⭐ Os pedidos da TABELA (W2b), pela ordem em que foram feitos — depois do início do
            // quadro da lei (a regeneração e as marcas) e antes dos golpes do contacto.
            for &(quem, pedido) in &pedidos {
                if quem != alvo {
                    continue;
                }
                let antes = st.vida;
                match pedido {
                    PedidoDeVida::Dano(quanto) => {
                        let rng = &mut st.rng;
                        st.vida
                            .golpe(&cfg, Regras::CASA, quanto, true, true, &mut || sorteio(rng));
                    }
                    PedidoDeVida::Cura(quanto) => st.vida.cura(&cfg, Regras::CASA, quanto),
                }
                if publicar {
                    // ⚠️ **A fonte é o PRÓPRIO alvo** — um verbo não tem quem bata. É o idioma que
                    // os eventos de player já usam (o `other` deles é ele próprio), e é o que faz
                    // o `From Myself` da tabela continuar a funcionar.
                    for kind in factos(&antes, &st.vida) {
                        self.health_events.push(HealthEvent {
                            target: alvo,
                            source: alvo,
                            kind,
                        });
                    }
                }
            }
            let fontes = agora.remove(&alvo).unwrap_or_default();
            for (&fonte, toque) in &fontes {
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
                // ⚠️ Os factos saem SEMPRE, e só a publicação depende do laço: o empurrão é estado
                // da simulação e o replay tem de o refazer igual.
                let fs = factos(&antes, &st.vida);
                if comecou && let Some(dv) = empurrao(dano, &h, toque.direccao, &fs) {
                    empurroes.push((toque.corpo, dv));
                }
                if publicar {
                    for kind in fs {
                        self.health_events.push(HealthEvent {
                            target: alvo,
                            source: fonte,
                            kind,
                        });
                    }
                }
            }
            st.tocando = fontes.into_keys().collect();
            vivos.insert(alvo, st);
        }
        self.health_state = vivos;
        for (corpo, dv) in empurroes {
            self.empurra(corpo, dv);
        }
        if publicar {
            self.damage_spent.extend(gastos);
        }
    }

    /// **Aplica UM empurrão** a um corpo — pelo CANAL do empurrão do mover de vista de cima (que a
    /// lei dele recupera a uma taxa própria) ou pela velocidade do solver num dinâmico.
    ///
    /// ⚠️ **O mover primeiro**: o corpo dele é CINEMÁTICO (a semente da casa), e o solver recusaria
    /// o empurrão — ele tem de entrar pelo estado que a lei do mover integra. ⛔ Um projéctil e um
    /// cinemático sem mover **não** são empurrados: a pose deles é de quem os conduz.
    fn empurra(&mut self, corpo: Entity, dv: [f32; 2]) {
        if let Some(st) = self.topdown_state.get_mut(&corpo) {
            st.knockback[0] += dv[0];
            st.knockback[1] += dv[1];
            return;
        }
        if let Some(b) = self.bodies.get(&corpo) {
            self.world.push_velocity(b.handle, dv);
        }
    }

    /// A direcção do centro de um corpo ao de outro — a do empurrão quando o toque não traz normal
    /// (um sensor, um mover). `None` se um deles não tem corpo.
    fn do_centro_ao_centro(&self, de: Entity, para: Entity) -> Option<[f32; 2]> {
        let centro = |e: Entity| {
            let h = self.bodies.get(&e)?.handle;
            let p = self.world.body_pose(h)?.translation;
            Some([p.x, p.y])
        };
        let (a, b) = (centro(de)?, centro(para)?);
        Some([b[0] - a[0], b[1] - a[1]])
    }
}

/// Um toque de uma fonte numa vida, neste tique: a direcção do empurrão e o CORPO que o leva.
#[derive(Clone, Copy, Debug)]
struct Toque {
    direccao: Option<[f32; 2]>,
    corpo: Entity,
}

/// Normaliza, ou `None` para um vector nulo ou não finito — ⛔ nunca `normalize_or_zero`: um corpo
/// exactamente em cima de outro não tem para onde ser empurrado, e inventar um eixo seria mentir (a
/// lição do estouro).
fn unitario(v: [f32; 2]) -> Option<[f32; 2]> {
    let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (n.is_finite() && n > f32::EPSILON).then(|| [v[0] / n, v[1] / n])
}

/// ⭐ **O empurrão de UM golpe** — a lei, pura (plano 28, W5). `None` = não empurra.
///
/// ⚠️ **Só um golpe que ENTROU empurra** (na vida, no escudo, ou a morte): uma esquiva e um golpe
/// na invencibilidade não produzem facto de dano, logo não empurram — senão o herói invencível
/// seria arrastado por um inimigo que o não fere.
///
/// O vector é `direcção · knockback + (0, lift)`, tudo escalado pelo `knockback_taken` da VIDA.
/// Sem direcção (corpos sobrepostos) sobra o `lift`, que não precisa de uma.
#[must_use]
pub(crate) fn empurrao(
    dano: &Damage,
    vida: &Health,
    direccao: Option<[f32; 2]>,
    fs: &[HealthEventKind],
) -> Option<[f32; 2]> {
    let entrou = fs.iter().any(|k| {
        matches!(
            k,
            HealthEventKind::Damaged { .. }
                | HealthEventKind::Shielded { .. }
                | HealthEventKind::Died
        )
    });
    if !entrou {
        return None;
    }
    let aceita = if vida.knockback_taken.is_finite() {
        vida.knockback_taken.max(0.0)
    } else {
        0.0
    };
    let d = direccao.unwrap_or([0.0, 0.0]);
    let dv = [
        d[0] * dano.knockback * aceita,
        (d[1] * dano.knockback + dano.knockback_lift) * aceita,
    ];
    let vale = dv[0].is_finite() && dv[1].is_finite() && (dv[0] != 0.0 || dv[1] != 0.0);
    vale.then_some(dv)
}

impl PhysicsBridge {
    /// **Publica a vida AGORA no mundo** ([`crate::HealthNow`]) — no fim de todo `dispatch`, pelas
    /// quatro saídas dele (tique, replay, pausa, quadro sem tique).
    ///
    /// ⚠️ **Só escreve quando MUDA** (o `get_mut` é comparado antes): a maioria dos quadros não mexe
    /// numa vida, e um `insert` por quadro seria trabalho sem leitor. E quem perdeu o estado (antes
    /// do 1.º tique, ou sem `Health`) perde também o readout — *um número de outra corrida lido
    /// como o de agora é pior do que nenhum*.
    pub(super) fn publica_vidas(&self, sim: &mut SimWorld) {
        let w = sim.world_mut();
        let mut velhos: Vec<Entity> = Vec::new();
        if let Some(mut q) = w.try_query::<(Entity, &crate::HealthNow)>() {
            velhos.extend(
                q.iter(w)
                    .filter(|(e, _)| !self.health_state.contains_key(e))
                    .map(|(e, _)| e),
            );
        }
        // ⚠️ O PISCAR (plano 28, W5) tem a MESMA metade de higiene: quem perdeu a vida perde a
        // marca, senão um objecto cuja `Health` saiu a meio de uma metade escondida ficava INVISÍVEL
        // para sempre — e ninguém a voltaria a tirar, porque o laço de baixo só visita quem tem vida.
        let mut apagados: Vec<Entity> = Vec::new();
        if let Some(mut q) = w.try_query::<(Entity, &ph2d_ecs::BlinkOff)>() {
            apagados.extend(
                q.iter(w)
                    .filter(|(e, _)| !self.health_state.contains_key(e))
                    .map(|(e, _)| e),
            );
        }
        for e in velhos {
            w.entity_mut(e).remove::<crate::HealthNow>();
        }
        for e in apagados {
            w.entity_mut(e).remove::<ph2d_ecs::BlinkOff>();
        }
        for (&e, st) in &self.health_state {
            let agora = crate::HealthNow {
                pontos: st.vida.pontos,
                escudo: st.vida.escudo,
                morta: st.vida.morta(),
            };
            let Ok(mut em) = w.get_entity_mut(e) else {
                continue;
            };
            match em.get_mut::<crate::HealthNow>() {
                Some(h) if *h == agora => {}
                Some(mut h) => *h = agora,
                None => {
                    em.insert(agora);
                }
            }
            // ⭐ O piscar é função do relógio da LEI (o `desde_golpe_s` que a vida já guarda), logo
            // o tique que se vê num scrub é o tique que a vida tem — nenhum relógio novo.
            let escondido = em.get::<Health>().is_some_and(|h| {
                !ph2d_health::pisca_visivel(
                    st.vida.desde_golpe_s(),
                    st.vida.invencivel(&h.config()),
                    f64::from(h.blink_s),
                )
            });
            match (escondido, em.contains::<ph2d_ecs::BlinkOff>()) {
                (true, false) => {
                    em.insert(ph2d_ecs::BlinkOff);
                }
                (false, true) => {
                    em.remove::<ph2d_ecs::BlinkOff>();
                }
                _ => {}
            }
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
    if depois.pontos > antes.pontos {
        out.push(HealthEventKind::Healed {
            amount: depois.pontos - antes.pontos,
        });
    }
    out
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;

/// A vida não guarda handles — o tipo vem aqui só para o campo da ponte ter nome.
pub(super) type ToqueDoMover = (Entity, RigidBodyHandle);
