//! **O que um nó declara que não dispensa** — ver o doc do `mod requirements` no `lib.rs`.

use crate::{NodeRegistry, RequiredTextParam};
use ph2d_nodegraph::node::NodeTypeId;

impl NodeRegistry {
    /// Register a node type's REQUIRED input ports by name (ADR-0155). Additive; last
    /// write wins. A node whose job needs a stream on a specific input (`duplicator` →
    /// `["shape", "points"]`) declares it, and the setup diagnoser flags a required port
    /// with no edge — the port-level twin of a `Coupling::Requires` (which is column-level).
    pub fn register_required_inputs(&mut self, id: NodeTypeId, ports: &'static [&'static str]) {
        self.required_inputs.insert(id, ports);
    }

    /// The required input port names for `id`, if any. Absent ⇒ no input is required.
    pub fn required_inputs(&self, id: NodeTypeId) -> Option<&'static [&'static str]> {
        self.required_inputs.get(&id).copied()
    }

    /// Regista os params de TEXTO sem os quais o nó fica inerte ([`RequiredTextParam`]).
    pub fn register_required_text_params(
        &mut self,
        id: NodeTypeId,
        params: &'static [RequiredTextParam],
    ) {
        self.required_text_params.insert(id, params);
    }

    /// Os params de TEXTO exigidos por `id`. Ausente ⇒ nenhum é exigido.
    #[must_use]
    pub fn required_text_params(&self, id: NodeTypeId) -> Option<&'static [RequiredTextParam]> {
        self.required_text_params.get(&id).copied()
    }
}
