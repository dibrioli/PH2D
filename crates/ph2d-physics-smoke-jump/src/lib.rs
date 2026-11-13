//! **As cenas de smoke do pulo** (110, 111) e o gate do pulo encostado na parede (Bug #14).
//!
//! Saíram da `ph2d-physics-smoke-scenes` (tecto de LOC) com o mesmo [`SceneCtx`]; a
//! `ph2d-app-physics` re-exporta os módulos, e nenhum chamador muda de caminho.
//!
//! [`SceneCtx`]: ph2d_physics_smoke_scenes::SceneCtx

pub mod physics_smoke_ledge;
pub mod physics_smoke_multi_jump;

#[cfg(test)]
mod wall_jump_tests;
