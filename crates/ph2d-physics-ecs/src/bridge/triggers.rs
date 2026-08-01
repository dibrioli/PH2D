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
