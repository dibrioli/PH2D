//! **Os ids da janela LINT.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_pixel_lint::Regra;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

pub const PANEL: NodeId = ph2d_editor_core::ids::PIXEL_LINT_PANEL;
pub const LINT_DRAG: NodeId = hash_node_id("pixel_lint.drag");
pub const LINT_EDGE: NodeId = hash_node_id("pixel_lint.edge");
pub const SEC_RULES: NodeId = hash_node_id("pixel_lint.sec.rules");
pub const SEC_ISSUES: NodeId = hash_node_id("pixel_lint.sec.issues");
/// O campo Color limit.
pub const COLOR_LIMIT: NodeId = hash_node_id("pixel_lint.color_limit");

/// O nome estável de uma regra (o sufixo dos ids e da chave i18n).
#[must_use]
pub fn nome(r: Regra) -> &'static str {
    match r {
        Regra::Orfao => "orphan",
        Regra::CantoDuplo => "doubles",
        Regra::Degrau => "jaggies",
        Regra::Faixa => "banding",
        Regra::Almofada => "pillow",
        Regra::CoresAMais => "too_many_colors",
    }
}

/// A caixa que liga/desliga a regra `r`.
#[must_use]
pub fn regra(r: Regra) -> NodeId {
    hash_node_id_runtime(&format!("pixel_lint.rule.{}", nome(r)))
}

/// A linha do grupo da regra `r` na lista de avisos.
#[must_use]
pub fn grupo(r: Regra) -> NodeId {
    hash_node_id_runtime(&format!("pixel_lint.group.{}", nome(r)))
}

/// A linha do aviso `i` do grupo `r`.
#[must_use]
pub fn aviso(r: Regra, i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_lint.issue.{}.{i}", nome(r)))
}
