//! **As peças de fábrica de um gabarito** (ADR-0201, plano `docs/JointEditor/01_plano.md` §W4): o
//! «Make the parts» ([`joint_factory`]), o número de peças de uma fileira já montada ([`joint_rows`]) e
//! o desenho do gabarito em metros ([`stage`]), que põe cada forma onde a fábrica põe a peça.
//! Saiu da `ph2d-joint-assembly` (tecto de linhas por crate); fica acima dela.

pub mod joint_factory;
pub mod joint_rows;
pub mod stage;
