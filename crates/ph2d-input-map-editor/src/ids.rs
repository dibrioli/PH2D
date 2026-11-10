//! Os ids de controlo da aba (plano §7: na crate que os lê).

use ph2d_a11y::NodeId;
use ph2d_editor_core::registry::hash_node_id;

pub const VIEW_LIST: NodeId = hash_node_id("input_map_tab.view.list");
pub const VIEW_KEYBOARD: NodeId = hash_node_id("input_map_tab.view.keyboard");
pub const VIEW_TRY: NodeId = hash_node_id("input_map_tab.view.try");

pub const NEW_NAME: NodeId = hash_node_id("input_map_list.new_name");
pub const ADD: NodeId = hash_node_id("input_map_list.add");
pub const SEARCH: NodeId = hash_node_id("input_map_list.search");

/// Os alvos por acção: a base de cada espécie, mais `acção × STRIDE + n`.
const STRIDE: u64 = 256;
const CHIP_X: NodeId = hash_node_id("input_map_list.chip_x");
const CHIP_PLUS: NodeId = hash_node_id("input_map_list.chip_plus");
const TRASH: NodeId = hash_node_id("input_map_list.trash");
const DEAD: NodeId = hash_node_id("input_map_list.dead");
const PRESS: NodeId = hash_node_id("input_map_list.press");
const ROW: NodeId = hash_node_id("input_map_list.row");
const CHIP: NodeId = hash_node_id("input_map_list.chip");
const DEAD_HANDLE: NodeId = hash_node_id("input_map_list.dead_handle");
const PRESS_HANDLE: NodeId = hash_node_id("input_map_list.press_handle");

const fn per(base: NodeId, a: ph2d_input::ActionId, n: usize) -> NodeId {
    NodeId(
        base.0
            .wrapping_add((a.0 as u64).wrapping_mul(STRIDE))
            .wrapping_add(n as u64),
    )
}

/// O que um alvo da lista é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListHit {
    RemoveBinding(ph2d_input::ActionId, usize),
    Listen(ph2d_input::ActionId),
    Trash(ph2d_input::ActionId),
    Dead(ph2d_input::ActionId),
    Press(ph2d_input::ActionId),
    Row(ph2d_input::ActionId),
    /// O chip de uma tecla (o × é o `RemoveBinding`).
    Chip(ph2d_input::ActionId, usize),
    /// A alça «D»/«P» que arrasta o número.
    DeadHandle(ph2d_input::ActionId),
    PressHandle(ph2d_input::ActionId),
}

impl ListHit {
    #[must_use]
    pub const fn id(self) -> NodeId {
        match self {
            Self::RemoveBinding(a, n) => per(CHIP_X, a, n),
            Self::Listen(a) => per(CHIP_PLUS, a, 0),
            Self::Trash(a) => per(TRASH, a, 0),
            Self::Dead(a) => per(DEAD, a, 0),
            Self::Press(a) => per(PRESS, a, 0),
            Self::Row(a) => per(ROW, a, 0),
            Self::Chip(a, n) => per(CHIP, a, n),
            Self::DeadHandle(a) => per(DEAD_HANDLE, a, 0),
            Self::PressHandle(a) => per(PRESS_HANDLE, a, 0),
        }
    }
}

pub const SEARCH_CLEAR: NodeId = hash_node_id("input_map_list.search_clear");

/// O nó de acessibilidade do documento da aba (os controlos são filhos dele).
pub const DOCUMENT: NodeId = hash_node_id("input_map_tab.document");

/// O polegar da barra da lista.
pub const LIST_THUMB: NodeId = hash_node_id("input_map_list.thumb");
