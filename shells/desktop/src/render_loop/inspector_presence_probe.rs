//! **A sonda de PRESENÇA das seções do Inspector** — oito perguntas de uma palavra, para o
//! [`crate::inspector_presence_tests`] (ADR-0166 / F3).
//!
//! ⚠️ **Ela existe porque os oito builders vivem em módulos privados do `render_loop` e cada um tem
//! a sua lista de argumentos** (a §11 sozinha pede nove). Sem esta camada, a lei da F3 teria de ser
//! escrita como oito testes soltos, cada um a soletrar os defaults do vizinho — e o que se perde aí
//! não é linhas: é a *lei*, que só existe enquanto for **uma** varredura sobre **uma** tabela.
//!
//! ⛔ **Nenhum destes wrappers decide coisa nenhuma** — cada um chama o builder de produção com os
//! fatos que só a shell tem nos seus valores NEUTROS, e devolve `is_some()`. Uma sonda que
//! respondesse por conta própria mediria a sonda.

use ph2d_ecs::{SimWorld, World};

pub(crate) fn ordering(world: &World, bits: u64) -> bool {
    super::inspector_ordering::build_ordering_info(world, bits, &[], 1).is_some()
}

/// ⭐ **O `Z Index` que a §7 MOSTRA** — `None` = `—` (vem da árvore).
///
/// ⚠️ Ela existe pela mesma razão das irmãs: o builder vive num módulo privado do `render_loop`, e
/// a lei da §7 precisa de afirmar **o que o campo diz**, não só que a seção existe. *Uma seção
/// presente com um zero fabricado passaria a metade 1 e mentiria ao artista.*
pub(crate) fn ordering_z_index(world: &World, bits: u64) -> Option<Option<i32>> {
    super::inspector_ordering::build_ordering_info(world, bits, &[], 1).map(|i| i.z_index)
}

pub(crate) fn sampling(world: &World, bits: u64) -> bool {
    super::inspector_ordering::build_sampling_info(world, bits, &[], 1).is_some()
}

pub(crate) fn blend(world: &World, bits: u64) -> bool {
    super::inspector_ordering::build_blend_info(world, bits, &[], 1).is_some()
}

pub(crate) fn slice(world: &World, bits: u64) -> bool {
    super::inspector_slice::build_slice_info(world, bits, &[], 1).is_some()
}

pub(crate) fn visibility_section(world: &World, bits: u64) -> bool {
    super::inspector_visibility::build_visibility_section_info(world, bits, &[], 1).is_some()
}

pub(crate) fn anchors(world: &World, bits: u64) -> bool {
    super::inspector_anchor::build_anchor_info(world, bits, &[], 1, 100.0).is_some()
}

pub(crate) fn anim(world: &World, bits: u64) -> bool {
    super::inspector_anim::build_anim_info(world, bits, 1).is_some()
}

pub(crate) fn physics(world: &World, bits: u64) -> bool {
    super::inspector_physics::build_physics_info(world, bits, 0, 0, 0, false, 0, (0.0, 5.0), 0)
        .is_some()
}

pub(crate) fn player(sim: &SimWorld, bits: u64) -> bool {
    super::inspector_player::build_player_info(
        sim,
        bits,
        0.0,
        0.0,
        None,
        ph2d_physics_ecs::PlayerLiveness::SPRING,
    )
    .is_some()
}
