//! **AS ÂNCORAS da seleção** — a projeção que o painel lê (plano UI/UX W3).
//!
//! Irmão do [`crate::state_layout`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecAnchors`) e isto é o que a shell publica por frame.
//!
//! # Por que o chip aceso viaja como `NodeId`
//!
//! ⚠️ Pelo mesmo motivo do layout: este painel **não alcança o `ph2d-ecs`**, e espelhar aqui os
//! pares de âncora criaria um segundo vocabulário para o mesmo facto. O que ele precisa saber é
//! *qual chip está aceso* — um `NodeId`, que já vive na `ph2d-editor-core`, que os DOIS lados leem.
//!
//! ⚠️ **E cada eixo é `Option`**: o componente admite pares que a UI não oferece (um `0,25` é
//! exprimível), e nesse caso **nenhum chip acende**, que é a verdade. Acender o mais próximo faria
//! a fileira dizer uma regra que o documento não tem.

use std::cell::Cell;

use ph2d_a11y::NodeId;

/// A regra de ancoragem do filho selecionado — um chip aceso por eixo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorState {
    /// O chip HORIZONTAL aceso (`VECTOR_ANCHOR_H_*`), ou `None` para um par sem nome.
    pub h: Option<NodeId>,
    /// O chip VERTICAL aceso (`VECTOR_ANCHOR_V_*`), ou `None`.
    pub v: Option<NodeId>,
}

thread_local! {
    static ANCHORS: Cell<Option<AnchorState>> = const { Cell::new(None) };
}

/// Publica a regra do filho selecionado (shell → painel). `None` = a seleção não é ancorável.
pub fn set_anchor_state(state: Option<AnchorState>) {
    ANCHORS.with(|c| c.set(state));
}

/// A regra do filho selecionado — `None` = não oferecer a seção.
#[must_use]
pub(crate) fn anchor_state() -> Option<AnchorState> {
    ANCHORS.with(Cell::get)
}
