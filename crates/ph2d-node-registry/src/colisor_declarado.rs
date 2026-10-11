//! ⭐ **QUEM DECLARA e QUEM LÊ o colisor** — as perguntas que o diagnóstico e a cerca do dispositivo
//! fazem a um botão `Collide` (doc 125 §2). Quem ARMA o passe do fim já não é pergunta: desde o
//! doc 115 §33 ele corre sozinho sobre toda declaração que chega a um sink.
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
    /// until they do: the flag is what the diagnosis (`ColisorMudo`) reads to tell
    /// a simulation that sees the declared shape from one that does not. (The
    /// device fence no longer needs it: since doc 115 §33 the sink's pass reads
    /// every declaration, so a declaration alone keeps the device away.)
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
    /// (`≥ 0,5`) — o `Collide` da `source.shape` e o do `source.object`. Aditivo; o último registo
    /// vence.
    pub fn register_collider_switch(&mut self, id: NodeTypeId, param: &'static str) {
        self.collider_switches.insert(id, param);
    }

    /// O interruptor com que `id` declara o colisor, se o tiver.
    #[must_use]
    pub fn collider_switch(&self, id: NodeTypeId) -> Option<&'static str> {
        self.collider_switches.get(&id).copied()
    }
}
