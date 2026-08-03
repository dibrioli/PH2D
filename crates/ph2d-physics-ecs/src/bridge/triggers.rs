//! **Trigger state** (W7) — which sensor has which body inside it.
//!
//! Split out of `bridge.rs` for LOC. A sensor collider passes through but the
//! solver reports its overlaps
//! ([`PhysicsWorld::intersecting_collider_pairs`]); this turns those pairs into
//! an entity map the overlay reads. `impl PhysicsBridge` here, so it reaches the
//! private fields the way the other `bridge::*` submodules do.
//!
//! # ⚠️ A chave do mapa é a FORMA; o que ela lista são CORPOS (W-PartSensor)
//!
//! Ser sensor é propriedade de um **collider**, nunca de um corpo — e enquanto
//! um corpo tinha exatamente uma forma as duas frases eram intercambiáveis. A
//! W-Compound tornou a segunda falsa e este módulo não foi reconferido, então o
//! caso mais comum que existe num módulo 2D ficou **morto em silêncio**: o
//! *sensor de pé* de um personagem (tronco sólido + uma peça-sensor embaixo, o
//! `isGrounded` de Box2D e Unity).
//!
//! Medido antes do conserto (`tests/measure_part_sensor.rs`): o chip **chegava**
//! ao solver — o tronco assenta em `1,6990` com o pé sólido e em `1,4990` com o
//! pé sensor, ou seja a peça de fato atravessa — e `triggered_sensors()` ficava
//! **vazio nos dois braços**. O par reportado era `(tronco, chão)`, o teste
//! perguntava se o collider **próprio do tronco** era sensor (não é), e a
//! sobreposição era descartada. E como o overlay é o ÚNICO consumidor deste
//! canal neste build, a consequência visível era exata: a peça-sensor
//! desenhada magenta **apagado para sempre**, sem nunca acender.
//!
//! As duas metades do par são resolvidas por perguntas diferentes, e é isso que
//! o torna correto:
//!
//! - **quem é o sensor** é a ENTIDADE que carrega o `Collider` marcado — o
//!   corpo, se for a forma própria dele; a **peça**, se for uma peça. É o que o
//!   artista marcou e é o que o contorno desenha.
//! - **o que está dentro** é o CORPO, lido do MESMO campo de onde a mão tira a
//!   resposta dela (`PartRef::owner`): quem pergunta *"quem entrou no
//!   gatilho?"* quer um objeto, não uma das formas dele.

use std::collections::BTreeMap;

use ph2d_ecs::Entity;

use super::PhysicsBridge;

/// **Algo ENTROU num sensor** (W-Signal) — a transição que o mapa permanente
/// não carrega.
///
/// Duas entidades, e cada uma responde a uma pergunta diferente, exatamente como
/// no canal de trigger (W-PartSensor): o **`sensor`** é quem o artista MARCOU (a
/// peça, se for peça), e o **`other`** é o CORPO que chegou — quem pergunta
/// *"quem entrou no gatilho?"* quer um objeto, não uma das formas dele.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TriggerEvent {
    /// A entidade que carrega o `Collider` sensor.
    pub sensor: Entity,
    /// O corpo que acabou de entrar.
    pub other: Entity,
}

/// De quem é este collider: a entidade que o AUTOROU, o corpo a que ele
/// pertence, e se ele é sensor.
type ColliderOwner = (Entity, Entity, bool);

impl PhysicsBridge {
    /// Rebuild [`triggers`](Self::triggers) from the world's current sensor
    /// overlaps. Each sensor entity gets the entities inside it. Returns early —
    /// before touching the reverse map — when nothing overlaps a sensor, which
    /// is every frame of a scene with no triggers.
    pub(super) fn rebuild_triggers(&mut self) {
        self.triggers.clear();
        let pairs = self.world.intersecting_collider_pairs();
        if pairs.is_empty() {
            // ⚠️ **O diff roda mesmo com o grafo VAZIO**, e a primeira versão não
            // rodava. O early-out existe para não construir os dois mapas numa
            // cena sem triggers — mas pular o diff junto significa que
            // `trigger_events` **guarda o conteúdo do último quadro em que houve
            // algo**, e o publicador o re-emite para sempre.
            //
            // Achado pela sonda `measure_sensor_blind_speed`: uma bala a 60 m/s
            // atravessava o sensor e o canal disparava **58 vezes em 60 ticks**,
            // porque o grafo esvaziava logo depois e nada mais limpava a lista.
            // *Um early-out que pula uma limpeza não é um atalho, é um vazamento.*
            self.diff_trigger_entries();
            return;
        }
        // handle → (forma, corpo, sensor?), construído aqui uma vez (só quando um
        // sensor de fato sobrepõe alguma coisa) em vez de mantido todo frame.
        //
        // As PEÇAS entram por handle direto (a ponte sabe onde pendurou cada
        // uma); a forma PRÓPRIA de um corpo entra pelo caminho inverso — o
        // `collider_body` do wrapper —, porque a ponte não guarda o handle de
        // collider de um corpo e adivinhá-lo pela ORDEM de inserção seria uma
        // suposição que a primeira peça pendurada quebraria.
        let mut by_collider: BTreeMap<(u32, u32), ColliderOwner> = BTreeMap::new();
        for (e, p) in &self.parts {
            by_collider.insert(p.handle.into_raw_parts(), (*e, p.owner, p.rest.is_sensor));
        }
        let mut by_body: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
        for (e, b) in &self.bodies {
            by_body.insert(b.handle.into_raw_parts(), *e);
        }
        let resolve = |c: ph2d_physics::ColliderHandle| -> Option<ColliderOwner> {
            if let Some(&hit) = by_collider.get(&c.into_raw_parts()) {
                return Some(hit);
            }
            let body = self.world.collider_body(c)?;
            let e = *by_body.get(&body.into_raw_parts())?;
            Some((e, e, self.bodies.get(&e)?.rest.is_sensor))
        };
        let mut hits: Vec<(Entity, Entity)> = Vec::new();
        for (c1, c2) in pairs {
            let (Some((shape1, body1, sensor1)), Some((shape2, body2, sensor2))) =
                (resolve(c1), resolve(c2))
            else {
                continue;
            };
            // Pelo menos um lado é sensor (um par sólido nunca se intersecta),
            // mas os dois podem ser — cada sensor lista o CORPO do outro lado.
            if sensor1 {
                hits.push((shape1, body2));
            }
            if sensor2 {
                hits.push((shape2, body1));
            }
        }
        for (sensor, inside) in hits {
            self.triggers.entry(sensor).or_default().push(inside);
        }
        for inside in self.triggers.values_mut() {
            inside.sort_unstable_by_key(|e| e.to_bits());
            inside.dedup();
        }
        self.diff_trigger_entries();
    }

    /// Diff the standing set against the previous one to fill
    /// [`trigger_events`](Self::trigger_events) e
    /// [`trigger_exits`](Self::trigger_exits) — *quem ENTROU* e *quem SAIU* de um
    /// sensor (W-Signal · W-SignalLeave).
    ///
    /// # Os dois extremos, cada um com o SEU nome
    ///
    /// O W-Signal deferiu a saída porque emitir os dois sob o MESMO nome tornaria
    /// o sinal ambíguo para quem escuta. A resposta é a que o deferral prescreveu:
    /// **duas listas**, lidas por **dois componentes** ([`crate::SignalOnHit`] e
    /// [`crate::SignalOnLeave`]), e nunca uma fase enfiada no mesmo evento.
    ///
    /// ⚠️ **Um diff, não dois** — as duas listas saem da MESMA comparação entre o
    /// conjunto de agora e o de antes. Duas varreduras seriam duas respostas a
    /// *quem estava dentro*, e o dia em que discordassem a porta fecharia sem ter
    /// aberto.
    ///
    /// # ⚠️ Este diff é por DISPATCH, e o irmão de contato é por TICK
    ///
    /// A assimetria é medida, não descuido. O canal de contato passou a ser
    /// por-tick (W-TickContacts) porque o **SOLVER separa** um par dentro de um
    /// tick: uma queda de 8 m não gerava evento nenhum. Um sensor não empurra —
    /// ele é atravessado —, então o tempo que algo passa dentro dele é
    /// geometria pura, e a única forma de perdê-lo é ATRAVESSAR o sensor inteiro
    /// dentro de um tick.
    ///
    /// Isso é uma velocidade, e ela foi **MEDIDA** — não deduzida.
    ///
    /// ⚠️ **O número que eu tinha escrito aqui estava errado por uma ordem de
    /// grandeza.** A aritmética diz que um sensor de `2h` metros é atravessado
    /// num tick a `2h / dt`, ou seja **60 m/s** para o sensor de 1 m da sonda; a
    /// varredura (`tests/signal.rs::measure_sensor_blind_speed`) mostra que ele
    /// ainda dispara a **280 m/s** e só fica cego a partir de **320**:
    ///
    /// | m/s | 240 | 280 | **320** | 480 | 960 |
    /// |---|---|---|---|---|---|
    /// | sinais | 1 | 1 | **0** | 0 | 0 |
    ///
    /// O mecanismo é a **fase larga**, que trabalha com AABBs preditas: o par
    /// entra no grafo pelo movimento, não pelas poses de fim de tick. Cinco vezes
    /// a margem que a aritmética previa.
    ///
    /// Acima disso o `Ccd` (W-CCD) é a resposta e já existe; um diff por-tick aqui
    /// custaria uma varredura do grafo de interseção **por tick** (o
    /// `rebuild_triggers` reconstrói dois mapas do zero) para cobrir um regime que
    /// o CCD cobre melhor. Nomeado com o número em vez de escondido — e o número é
    /// o da medição, não o do meu palpite (§0).
    ///
    /// # A descontinuidade RE-BASELIZA em silêncio
    ///
    /// A mesma disciplina do canal de contato, e pelo mesmo motivo: o conjunto
    /// permanente é recomputado do zero, então um diff ingênuo faz de todo
    /// scrub uma tempestade de entradas. `hold` e `rewind` derrubam
    /// `triggers_continuous`, e a próxima passada adota o conjunto sem reportar
    /// nada.
    fn diff_trigger_entries(&mut self) {
        self.trigger_events.clear();
        self.trigger_exits.clear();
        if !self.triggers_continuous {
            self.trigger_since = self.triggers.clone();
            self.triggers_continuous = true;
            return;
        }
        for (sensor, inside) in &self.triggers {
            let before = self.trigger_since.get(sensor);
            for &other in inside {
                if !before.is_some_and(|v| v.contains(&other)) {
                    self.trigger_events.push(TriggerEvent {
                        sensor: *sensor,
                        other,
                    });
                }
            }
        }
        // A metade da SAÍDA percorre a baseline, não o conjunto de agora — um
        // sensor que esvaziou por completo não tem entrada nenhuma em
        // `self.triggers` (o `rebuild` só cria a chave quando há alguém dentro),
        // então varrer o conjunto de agora perderia exatamente o caso que a
        // porta existe para cobrir: o último corpo a sair.
        for (sensor, before) in &self.trigger_since {
            let now = self.triggers.get(sensor);
            for &other in before {
                if !now.is_some_and(|v| v.contains(&other)) {
                    self.trigger_exits.push(TriggerEvent {
                        sensor: *sensor,
                        other,
                    });
                }
            }
        }
        self.trigger_since.clone_from(&self.triggers);
    }

    /// Forget what was inside every sensor, **sem reportar nada** — o irmão
    /// exato do `discard_contact_history`, chamado das mesmas duas
    /// descontinuidades.
    ///
    /// ⚠️ **As duas limpezas de lista aqui são DEFESA EM CAMADA, e a medição o
    /// diz** (W-SignalLeave): todo caminho que descarta a história segue para o
    /// `rebuild_triggers` do mesmo dispatch, e o `diff_trigger_entries` limpa as
    /// duas listas no topo. Mutação: apagar `trigger_exits.clear()` **não
    /// sangra** — e o CONTROLE prova que não é buraco de gate desta wave, porque
    /// apagar o `trigger_events.clear()` **pré-existente** também não sangra. As
    /// duas ficam pelo mesmo motivo do precedente do ADR-0145: no regime que
    /// shipa a segunda camada não é observável, e um `discard` que deixasse
    /// eventos para trás seria um bug óbvio no dia em que alguém lesse este
    /// canal fora do dispatch.
    pub(super) fn discard_trigger_history(&mut self) {
        self.trigger_since.clear();
        self.trigger_events.clear();
        self.trigger_exits.clear();
        self.triggers_continuous = false;
    }

    /// **Quem ENTROU num sensor** neste dispatch (W-Signal). Vazio em todo frame
    /// em que nada chegou, que é quase todos.
    #[must_use]
    pub fn trigger_events(&self) -> &[TriggerEvent] {
        &self.trigger_events
    }

    /// **Quem SAIU de um sensor** neste dispatch (W-SignalLeave) — o espelho
    /// exato do irmão, do mesmo diff.
    ///
    /// ⚠️ **Uma descontinuidade re-baseliza em SILÊNCIO nos dois canais**, e aqui
    /// isso importa mais: sem o re-baseline, um scrub que sai da sobreposição
    /// gritaria uma saída que a simulação nunca atravessou — e o consumidor
    /// fecharia uma porta que, no tempo para onde o artista foi, está aberta.
    #[must_use]
    pub fn trigger_exits(&self) -> &[TriggerEvent] {
        &self.trigger_exits
    }

    /// Is `entity` a **sensor** with at least one body inside it right now? The
    /// overlay reads this to light a triggered sensor up.
    ///
    /// ⚠️ This used to claim the Inspector read it for an "N inside" readout. It never
    /// did — §11 has no readout row, and grepping for a consumer found none (caught
    /// while building the contact channel next door, which faced the same question and
    /// gave the same answer: the visible half is the OVERLAY). A comment that names a
    /// consumer which does not exist is worse than none, because it reads as coverage
    /// ([[feedback_stale_comment_and_dead_code_lie]]).
    pub fn is_triggered(&self, entity: Entity) -> bool {
        self.triggers.get(&entity).is_some_and(|v| !v.is_empty())
    }

    /// The entities currently inside sensor `entity` (empty slice if it is not a
    /// triggered sensor). Sorted for a stable readout.
    ///
    /// Queryable state with no consumer in this build — deliberate, and the same
    /// shape the contact list has: *what a game DOES with an overlap* is the next
    /// layer, and this is the door it will come through.
    pub fn bodies_inside(&self, entity: Entity) -> &[Entity] {
        self.triggers.get(&entity).map_or(&[], Vec::as_slice)
    }

    /// The sensor entities that have at least one body inside them right now —
    /// what the overlay lights up. Sorted (the map is). Empty without sensors.
    pub fn triggered_sensors(&self) -> Vec<Entity> {
        self.triggers
            .iter()
            .filter(|(_, inside)| !inside.is_empty())
            .map(|(e, _)| *e)
            .collect()
    }
}
