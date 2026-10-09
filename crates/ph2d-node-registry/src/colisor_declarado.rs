//! ⭐ **QUEM DECLARA, QUEM LÊ E QUEM ARMA o colisor** — as três perguntas que o diagnóstico faz a
//! um botão `Collide` (doc 125 §2): *a declaração deste nó chega a alguém que a leia?*
//!
//! Os campos ficam no `lib.rs` (são o estado do registry); os acessores moram aqui.

use crate::NodeRegistry;
use ph2d_nodegraph::node::NodeTypeId;

impl NodeRegistry {
    /// Register a node type as a **reader of the DECLARED collider** (doc 115 W4):
    /// it calls `ph2d_contact::colisores` on the stream that reaches it, so the
    /// shape an object declares changes what it does. Additive; idempotent.
    ///
    /// ⚠️ **Whoever writes the next such node registers it here**, and the census
    /// in `ph2d-app-motion` (`todo_leitor_do_colisor_declarado_se_regista`) fails
    /// until they do: the flag is what a fence reads, and a reader that forgets it
    /// cooks on the device with a disc kernel while the CPU honours a box.
    pub fn register_declared_collider_reader(&mut self, id: NodeTypeId) {
        self.declared_collider_readers.insert(id);
    }

    /// Does `id` read the collider the stream DECLARES? Absent ⇒ no, the
    /// byte-identical default.
    #[must_use]
    pub fn reads_declared_collider(&self, id: NodeTypeId) -> bool {
        self.declared_collider_readers.contains(&id)
    }

    /// Este tipo DECLARA o colisor das suas peças enquanto o param `param` estiver ligado
    /// (`≥ 0,5`) — a `source.shape` e o seu `Collide`. Aditivo; o último registo vence.
    pub fn register_collider_switch(&mut self, id: NodeTypeId, param: &'static str) {
        self.collider_switches.insert(id, param);
    }

    /// O interruptor com que `id` declara o colisor, se o tiver.
    #[must_use]
    pub fn collider_switch(&self, id: NodeTypeId) -> Option<&'static str> {
        self.collider_switches.get(&id).copied()
    }

    /// Este sink ARMA o passe do fim do cozimento (`ph2d_contact::passe`) enquanto o param
    /// `param` estiver ligado — o `Collide` do `motion.output`. Aditivo; o último registo vence.
    pub fn register_contact_pass_switch(&mut self, id: NodeTypeId, param: &'static str) {
        self.contact_pass_switches.insert(id, param);
    }

    /// O interruptor com que `id` arma o passe do fim, se o tiver.
    #[must_use]
    pub fn contact_pass_switch(&self, id: NodeTypeId) -> Option<&'static str> {
        self.contact_pass_switches.get(&id).copied()
    }
}
