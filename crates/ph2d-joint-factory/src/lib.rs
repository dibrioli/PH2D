//! **As peças de fábrica de um gabarito** (ADR-0201, plano `docs/JointEditor/01_plano.md` §W4): o
//! «Make the parts» ([`joint_factory`]) e o número de peças de uma fileira já montada ([`joint_rows`]).
//! Saiu da `ph2d-joint-assembly` (tecto de linhas por crate); fica acima dela.

pub mod joint_factory;
pub mod joint_rows;
