//! O campo da força no cartão CONSTRAINTS, derivado da barra de cada linha.

use super::inspector_constraint::INSP_CONSTRAINT_FORCA;
use ph2d_a11y::NodeId;
use ph2d_editor_core::constraint_edits::MAX_LINHAS;
use ph2d_tool_registry::hash_node_id;

/// A barra continua a avisar o painel, e o Auto Key lê os dois ids como o mesmo canal.
pub const INSP_CONSTRAINT_FORCA_CHIP: [NodeId; MAX_LINHAS] = {
    let sal = hash_node_id("insp_constraint_forca_chip").0;
    let mut t = INSP_CONSTRAINT_FORCA;
    let mut k = 0;
    while k < MAX_LINHAS {
        t[k] = NodeId(t[k].0 ^ sal);
        k += 1;
    }
    t
};
