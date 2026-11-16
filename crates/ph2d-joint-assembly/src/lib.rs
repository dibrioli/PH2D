//! **A montagem de juntas** (ADR-0201) — o objeto que agrupa as juntas de um gabarito, e os
//! gestos que a montam. A `ph2d-app-physics` re-exporta cada módulo pelo mesmo nome. As peças de fábrica
//! e as fileiras (e os testes que montam máquinas com elas) moram na `ph2d-joint-factory`, acima desta.

pub mod joint_assembly;
pub mod joint_assembly_card;
pub mod joint_create;
pub mod joint_drop;
pub mod joint_info;
pub mod joint_world;
