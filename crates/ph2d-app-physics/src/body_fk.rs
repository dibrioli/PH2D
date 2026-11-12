//! **POSAR PELA JUNTA** (W-FK) — o gesto de cinemática DIRETA: o artista arrasta um elo e o elo
//! e os descendentes rodam por geometria exata (a IK do [`crate::body_pose`] acerta um ALVO; a FK
//! autora um ÂNGULO). Mesma fiação da pose: relógio parado, `Transform` autorado, um passo de undo.
//!
//! Condições: o modo FK em mãos (`JointTool::gesture()`; Alt suprime), relógio PARADO, uma junta
//! com grau de liberdade acima do corpo (`fk_begin` da ponte), e a trava (`Locked`) honrada. O
//! toggle `Physics` não é condição. A escrita é a da IK: [`crate::body_pose::write_world_pose`].

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

/// **A porta única do press.** `true` = a FK pegou, e o chamador NÃO deve abrir
/// arrasto de gizmo nem mexer na seleção.
///
/// Mesma divisão do `body_pose::take_pose` e do `body_grab::take_hold`: as
/// condições que dependem do relógio e da ferramenta chegam como argumento (é o
/// chamador que as vê), a estrutural é da ponte. É assim que a decisão inteira
/// fica testável sem janela.
pub fn take_fk(
    physics: &mut PhysicsBridge,
    sim: &SimWorld,
    is_fk: bool,
    entity: Entity,
    cursor: [f32; 2],
    playing: bool,
) -> bool {
    if playing || !is_fk {
        return false;
    }
    physics.fk_begin(sim, entity, cursor)
}

/// **O elo segue o cursor em torno da própria junta.** No-op sem sessão, então roda
/// em todo Move ao lado dos outros `advance_*`.
///
/// **Devolve `true` se AUTOROU** — e é o chamador que marca o quadro (na shell,
/// `any_input_this_frame`; pela porta do host, `note_authored_change`). O gesto sabe
/// *que* autorou; **quando** um quadro conta para o diff de undo é da shell.
///
/// ⚠️⚠️ **Era um `impl App` e passou a receber o que de facto usa** (W2/L2 Fase B).
/// Os três argumentos são todos de **crate de módulo** — [`PhysicsBridge`] é da
/// `ph2d-physics-ecs`, [`SimWorld`] da `ph2d-ecs`, e o ponto é um par de `f32`. ⭐ *Não
/// foi preciso um sexto método no `AppHost`:* o que parecia «precisar da `App`» era
/// precisar de **três tipos que a shell por acaso segurava**, e a conversão
/// `ecrã → mundo` fica onde a câmera está, que é a shell.
pub fn advance_body_fk(physics: &mut PhysicsBridge, sim: &mut SimWorld, world: [f32; 2]) -> bool {
    if !physics.is_posing_fk() {
        return false;
    }
    let poses = physics.fk_move(world);
    for (e, translation, rotation) in poses {
        crate::body_pose::write_world_pose(sim, e, translation, rotation);
    }
    true
}

/// Encerra o gesto. Idempotente — soltar sem ter posado é o caso de quase todo
/// release do app, e é por isso que ele mora no topo do handler junto dos irmãos.
pub fn release_body_fk(physics: &mut PhysicsBridge) {
    physics.fk_end();
}

#[cfg(test)]
#[path = "body_fk_tests.rs"]
mod tests;
