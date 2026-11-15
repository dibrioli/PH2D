//! ⭐ **O objecto «Ray Sensor» do lado da app** (GEF 07, `docs/GameEngineFuncional/componentes/07_ray_sensor.md`).
//!
//! A lei do raio (o lance, os filtros, a marca do canvas) é da `ph2d-physics-ecs`; aqui mora o que o
//! cartão e o script fazem com ela: [`inspector`] (o instantâneo e o dreno), [`topo`] (o «Ignore up
//! to» e o conta-gotas). Saiu da `ph2d-app-components`, que a reexporta pelos nomes de antes.

pub mod gotas;
pub mod inspector;
pub mod script;
pub mod topo;
