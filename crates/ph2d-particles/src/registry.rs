//! **O registo mínimo** — os dez nós que o compilador usa, e nenhum outro.

use ph2d_node_registry::{NodeRegistry, RegistryError};

/// O registador de um nó — a assinatura que as dez crates-folha exportam.
type Register = fn(&mut NodeRegistry) -> Result<(), RegistryError>;

/// Um `NodeRegistry` com exactamente os nós que [`crate::compile`] monta.
///
/// # Panics
/// Um dos dez recusa registar-se — um defeito de build, não de dados.
#[must_use]
pub fn node_registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    let regs: [Register; 10] = [
        ph2d_node_motion_emitter::register,
        ph2d_node_motion_integrate::register,
        ph2d_node_force_wind::register,
        ph2d_node_force_drag::register,
        ph2d_node_motion_tint::register,
        ph2d_node_motion_drive::register,
        ph2d_node_value_attribute::register,
        ph2d_node_value_map_range::register,
        ph2d_node_value_table::register,
        ph2d_node_motion_output::register,
    ];
    for r in regs {
        r(&mut reg).expect("um nó do emissor recusou registar-se");
    }
    reg
}
