//! Os NÚMEROS de fábrica de um nó recém-solto (ciclo 18, doc 127) — cortado do `lib.rs` pelo tecto
//! de LOC; o irmão do `register_text_defaults`.

use crate::NodeRegistry;
use ph2d_nodegraph::node::NodeTypeId;

impl NodeRegistry {
    /// **O NÚMERO que um nó recém-SOLTO já carrega** — o irmão numérico de
    /// [`Self::register_text_defaults`], aplicado pelo editor no MESMO gesto.
    ///
    /// ⭐ Existe para mudar o que um nó NOVO faz sem mudar o que um documento GRAVADO faz (ciclo
    /// 18: o `motion.tint` novo mistura em OKLab, e um gravado sem o param continua no default do
    /// manifesto — RGB linear, ao bit). ⚠️ O default do MANIFESTO é o que um documento antigo lê;
    /// este é o que o artista recebe ao largar o nó.
    ///
    /// Aditivo; last write wins. Ausente ⇒ nada é semeado.
    pub fn register_param_defaults(
        &mut self,
        id: NodeTypeId,
        pairs: &'static [(&'static str, f32)],
    ) {
        self.param_defaults.insert(id, pairs);
    }

    /// Os números de fábrica de um nó recém-solto ([`Self::register_param_defaults`]).
    #[must_use]
    pub fn param_defaults(&self, id: NodeTypeId) -> &'static [(&'static str, f32)] {
        self.param_defaults.get(&id).copied().unwrap_or(&[])
    }
}
