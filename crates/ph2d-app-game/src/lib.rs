//! A família do JOGO a correr (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`).
//!
//! A shell compõe; a lei vive aqui: [`Sessao`] decide quando o documento se fotografa e quando volta.

mod dicas;
pub mod encaixe;
pub mod object_add;
pub mod personagem;
pub mod sessao;
pub mod smoke;

pub use sessao::{Passo, Sessao, marca_a_copia};

/// O registo que os gates da família usam: os registadores que os objectos do menu Add tocam.
#[cfg(test)]
pub(crate) fn registo_de_teste() -> ph2d_ecs::scene::ComponentRegistry {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}
