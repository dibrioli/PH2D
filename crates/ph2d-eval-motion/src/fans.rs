//! **Os LEQUES DE TEMPO desta marcha** ([ADR-0163](../../../docs/architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md))
//! — a porta 0 de um nó cozida em N instantes em vez de uma vez.
//!
//! ⚠️ **Eles são ESTADO, e os escopos de tempo são ARGUMENTO. A assimetria é
//! deliberada e tem duas razões.**
//!
//! A primeira é que um leque precisa da **duração de um tique**, que não sai do
//! grafo: ela é do shell, que é quem faz o playhead andar, enquanto um escopo é
//! função pura do documento.
//!
//! A segunda é **isolamento** (CLAUDE.md §0.2, *ao criar foundational novo
//! projete-o para isolamento*): pendurar mais um argumento no
//! `advance_or_scrub_scoped` mexeria em **29 sítios de chamada** e faria desta
//! linha um ímã de conflito para todas as outras. Um ponto de extensão
//! append-only não move assinatura nenhuma.
//!
//! ⚠️ **Quem esquece de os pousar num quadro cozinha com os do quadro anterior**,
//! que é o modo de falha de todo estado por-quadro. Há gate de FONTE a exigir que
//! o shell chame o `set_time_fans` ao lado do `time_scopes`.
//!
//! Vazio por omissão ⇒ toda marcha que não o põe cozinha exactamente como antes,
//! ao bit.

use super::MotionCookPump;

impl MotionCookPump {
    /// Pousa os leques desta marcha — uma vez por quadro.
    pub fn set_time_fans(&mut self, fans: ph2d_nodegraph::cook::TimeFans) {
        self.fans = fans;
    }

    /// Os leques em vigor (o censo, e o gate que confere que a marcha os usou).
    #[must_use]
    pub fn time_fans(&self) -> &ph2d_nodegraph::cook::TimeFans {
        &self.fans
    }
}
