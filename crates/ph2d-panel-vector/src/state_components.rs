//! **O COMPONENTE da seleção** — a projeção que o painel lê (plano UI/UX W5).
//!
//! Irmão do [`crate::state_anchors`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecComponentMain` / `VecInstance`) e isto é o que a shell publica por frame. O
//! painel não alcança o mundo — se alcançasse, a resposta que decide QUE botão pintar divergiria
//! da que HONRA o clique.
//!
//! ⚠️ **Os quatro campos são a resposta a *"que verbos fazem sentido agora?"*, e nada mais.** Uma
//! contagem de instâncias, ou o nome do mestre, seriam factos que o painel mostraria e que ninguém
//! usa para decidir — e cada um deles é uma cópia que fica velha.

use std::cell::Cell;

/// O que a seleção É, do ponto de vista dos componentes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComponentState {
    /// A seleção é um MESTRE (oferece *Place Instance*).
    pub is_main: bool,
    /// A seleção é uma INSTÂNCIA (oferece *Detach*).
    pub is_instance: bool,
    /// A instância selecionada tem overrides (oferece *Reset Overrides*).
    ///
    /// ⚠️ Separado de `is_instance` de propósito: um *Reset* sobre uma instância limpa é um clique
    /// que não faz nada, e o artista não tem como saber disso antes de o dar.
    pub has_overrides: bool,
    /// O mestre desta instância **não resolve** — o readout de órfã.
    pub main_missing: bool,
}

thread_local! {
    static COMPONENT: Cell<Option<ComponentState>> = const { Cell::new(None) };
}

/// Publica o estado da seleção (shell → painel). `None` = não oferecer a seção.
pub fn set_component_state(state: Option<ComponentState>) {
    COMPONENT.with(|c| c.set(state));
}

/// O estado da seleção — `None` = não oferecer a seção.
#[must_use]
pub(crate) fn component_state() -> Option<ComponentState> {
    COMPONENT.with(Cell::get)
}
