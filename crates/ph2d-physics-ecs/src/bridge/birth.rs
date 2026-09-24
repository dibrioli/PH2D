//! **Como uma ponte NASCE, e como ela RENASCE** — o [`PhysicsBridge::new`] e o
//! [`PhysicsBridge::rebuild`].
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e não arrumação** (tecto de LOC do ficheiro-mãe, suplente #21
//! W5): o [`super`] declara **o que uma ponte TEM** — setenta e tal campos, cada um com a prosa que
//! diz porque existe —, e este responde *como se constrói uma* e *o que se deita fora ao trocar de
//! documento. Um campo novo continua a custar uma linha em cada um dos dois, que é o preço honesto:
//! **nascer é uma decisão por campo**, e um `..Default::default()` faria essa decisão em silêncio.
//!
//! Módulo FILHO, logo os campos privados do ficheiro-mãe continuam ao alcance.

use super::*;

impl Default for PhysicsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsBridge {
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            player_probes: Vec::new(),
            player_views: BTreeMap::new(),
            player_events: Vec::new(),
            player_probe_anchors: Vec::new(),
            bodies: BTreeMap::new(),
            last_stepped: 0,
            query: None,
            joints: BTreeMap::new(),
            ik: None,
            fk: None,
            joint_query: None,
            part_query: None,
            surface_query: None,
            no_cling_query: None,
            surfaces: surfaces::Surfaces::default(),
            parts: std::collections::BTreeMap::new(),
            part_seen: Vec::new(),
            wheel_query: None,
            seen: Vec::new(),
            names: BTreeMap::new(),
            joints_seen: Vec::new(),
            joints_to_spawn: Vec::new(),
            joints_to_remove: Vec::new(),
            joints_to_retune: Vec::new(),
            joints_to_seed: Vec::new(),
            wheels_to_seed: Vec::new(),
            pulleys_to_install: Vec::new(),
            pulley_wheels_to_install: Vec::new(),
            rope_wheels: Vec::new(),
            wheel_entities: Vec::new(),
            wheel_wraps: Vec::new(),
            wheel_spin: Vec::new(),
            wheel_spin_by_entity: std::collections::BTreeMap::new(),
            route_scratch: Vec::new(),
            pulley_records: Vec::new(),
            joints_to_sync: Vec::new(),
            kin_start: Vec::new(),
            chain: Vec::new(),
            readback_order: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
            to_settle: Vec::new(),
            settings: PhysicsSettings::default(),
            ring: PhysicsCheckpointRing::new(),
            steps_taken: 0,
            triggers: BTreeMap::new(),
            ray_hits: BTreeMap::new(),
            ray_marks: Vec::new(),
            ray_enters: Vec::new(),
            ray_exits: Vec::new(),
            trigger_since: BTreeMap::new(),
            trigger_events: Vec::new(),
            trigger_exits: Vec::new(),
            triggers_continuous: true,
            contacts: Vec::new(),
            contact_since: BTreeMap::new(),
            contact_events: Vec::new(),
            joint_breaks: Vec::new(),
            joint_peaks: BTreeMap::new(),
            pulley_peaks: BTreeMap::new(),
            flashes: Vec::new(),
            contacts_continuous: true,
            player_input: BTreeMap::new(),
            player_launch: BTreeMap::new(),
            state_ring: BTreeMap::new(),
            player_state: BTreeMap::new(),
            topdown_state: BTreeMap::new(),
            projectile_state: BTreeMap::new(),
            projectile_done: Vec::new(),
            health_state: BTreeMap::new(),
            health_events: Vec::new(),
            damage_spent: Vec::new(),
            pedidos_de_vida: Vec::new(),
            fita_da_vida: BTreeMap::new(),
            toques_do_mover: Vec::new(),
            player_drop: BTreeMap::new(),
        }
    }

    /// Throw away the derived world. Call on project load / undo restore —
    /// entity bits are recycled there, so the handle map dangles. The world
    /// is rebuilt from components on the next [`dispatch`](Self::dispatch)
    /// (runtime-truth: the components are the truth, the world is derived).
    pub fn rebuild(&mut self) {
        self.world = PhysicsWorld::new();
        // A fresh world starts from the ENGINE defaults, not this document's
        // settings — re-push them or a project load quietly reverts every knob.
        self.settings.apply_to(&mut self.world);
        self.bodies.clear();
        self.joints.clear();
        self.last_stepped = 0;
        self.query = None; // re-bind to the (possibly fresh) world
        self.joint_query = None;
        self.wheel_query = None;
        self.ring.clear(); // cached states belong to the document being left
        // ⭐ E as VIDAS (plano 28, W2): elas são estado da corrida do documento que se deixa, e o
        // `last_stepped = 0` acima recomeça-a do repouso — onde toda vida está no `start`.
        self.health_state.clear();
        // ⚠️ E os PEDIDOS de vida e a fita deles (W2b): chaveados por `Entity`, e os bits são
        // reciclados aqui — um pedido de ontem cairia noutra vida.
        self.pedidos_de_vida.clear();
        self.fita_da_vida.clear();
        // Entity bits are recycled here, so a held input would start driving
        // SOMEONE ELSE — the same trap that made joint anchors travel by NAME.
        self.clear_player_input();
    }
}
