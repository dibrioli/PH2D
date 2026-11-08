//! **Os ids da janela SLICES.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_pixel_slices::campos::Campo;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

pub const SLICES_DRAG: NodeId = hash_node_id("pixel_slices.drag");
pub const SEC_LIST: NodeId = hash_node_id("pixel_slices.sec.list");
pub const SEC_PROPS: NodeId = hash_node_id("pixel_slices.sec.props");

pub const ALL: NodeId = hash_node_id("pixel_slices.all");
pub const NONE: NodeId = hash_node_id("pixel_slices.none");
pub const TRANSFORM: NodeId = hash_node_id("pixel_slices.transform");
pub const NINE: NodeId = hash_node_id("pixel_slices.nine");
pub const PIVOT: NodeId = hash_node_id("pixel_slices.pivot");
pub const DELETE: NodeId = hash_node_id("pixel_slices.delete");
pub const DUPLICATE: NodeId = hash_node_id("pixel_slices.duplicate");
pub const COLOR: NodeId = hash_node_id("pixel_slices.color");

pub const SLICE_NAME: NodeId = hash_node_id("pixel_slices.name");
pub const USER_DATA: NodeId = hash_node_id("pixel_slices.user_data");
pub const BOUNDS_X: NodeId = hash_node_id("pixel_slices.bounds.x");
pub const BOUNDS_Y: NodeId = hash_node_id("pixel_slices.bounds.y");
pub const BOUNDS_W: NodeId = hash_node_id("pixel_slices.bounds.w");
pub const BOUNDS_H: NodeId = hash_node_id("pixel_slices.bounds.h");
pub const CENTER_X: NodeId = hash_node_id("pixel_slices.center.x");
pub const CENTER_Y: NodeId = hash_node_id("pixel_slices.center.y");
pub const CENTER_W: NodeId = hash_node_id("pixel_slices.center.w");
pub const CENTER_H: NodeId = hash_node_id("pixel_slices.center.h");
pub const PIVOT_X: NodeId = hash_node_id("pixel_slices.pivot.x");
pub const PIVOT_Y: NodeId = hash_node_id("pixel_slices.pivot.y");

/// O menu do botão direito sobre uma slice selecionada, e as linhas dele.
pub const MENU: NodeId = hash_node_id("pixel_slices.menu");
pub const MENU_PROPERTIES: NodeId = hash_node_id("pixel_slices.menu.properties");
pub const MENU_DUPLICATE: NodeId = hash_node_id("pixel_slices.menu.duplicate");
pub const MENU_DELETE: NodeId = hash_node_id("pixel_slices.menu.delete");
pub const MENU_REMOVE_KEY: NodeId = hash_node_id("pixel_slices.menu.remove_key");

/// As caixas apagadas, sem seleção: ninguém as escuta.
pub const NINE_OFF: NodeId = hash_node_id("pixel_slices.nine.off");
pub const PIVOT_OFF: NodeId = hash_node_id("pixel_slices.pivot.off");

pub const BOTOES: [NodeId; 11] = [
    ALL,
    NONE,
    TRANSFORM,
    NINE,
    PIVOT,
    DELETE,
    DUPLICATE,
    MENU_PROPERTIES,
    MENU_DUPLICATE,
    MENU_DELETE,
    MENU_REMOVE_KEY,
];

/// Cada campo de número com o que ele edita.
pub const CAMPOS: [(NodeId, Campo); 10] = [
    (BOUNDS_X, Campo::X),
    (BOUNDS_Y, Campo::Y),
    (BOUNDS_W, Campo::W),
    (BOUNDS_H, Campo::H),
    (CENTER_X, Campo::CentroX),
    (CENTER_Y, Campo::CentroY),
    (CENTER_W, Campo::CentroW),
    (CENTER_H, Campo::CentroH),
    (PIVOT_X, Campo::PivotX),
    (PIVOT_Y, Campo::PivotY),
];

/// Os ids do cartão User Data da slice.
#[must_use]
pub fn user_data() -> ph2d_editor_core::panel::cards::user_data::UserDataIds {
    ph2d_editor_core::panel::cards::user_data::UserDataIds::new("pixel_slices")
}

/// A linha `i` da lista, na ordem por nome.
#[must_use]
pub fn linha(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_slices.row.{i}"))
}
