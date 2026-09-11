//! **POSAR PELA JUNTA** (W-FK) — o gesto de cinemática DIRETA.
//!
//! Irmão do [`crate::body_pose`] em tudo que é fiação (o mesmo relógio parado, o
//! mesmo `Transform` autorado, o mesmo passo de undo, a mesma porta de escrita)
//! e o oposto dele em INTENÇÃO:
//!
//! | | a POSE (W-IK) | a JUNTA (W-FK) |
//! |---|---|---|
//! | o artista arrasta | a **ponta** da cadeia | **um elo** qualquer |
//! | o que ele pede | *ponha a mão ali* | *dobre o cotovelo assim* |
//! | quem se move | tudo entre a raiz e a ponta | o elo e os **descendentes** |
//! | como | um solver amortecido | geometria exata |
//!
//! Nenhuma substitui a outra, e é por isso que todo pacote de animação carrega
//! as duas: a IK acerta um ALVO e a FK autora um ÂNGULO. O artista alterna entre
//! elas dentro do mesmo plano.
//!
//! # As condições, e a que NÃO está aqui
//!
//! 1. **O modo de joint em mãos é o FK** — `JointTool::gesture()`, a porta única
//!    (e é ela que faz o **Alt** suprimir o gesto: Alt significa *leve o rig
//!    inteiro*, que é um arrasto e não uma pose).
//! 2. **O relógio está PARADO.** O resultado é `Transform` autorado, e com o
//!    relógio andando o `readback` o sobrescreveria no mesmo frame.
//! 3. **Há uma junta com grau de liberdade acima do corpo pego** — respondido
//!    pelo `fk_begin` da ponte, não por uma cópia da regra aqui.
//! 4. **A trava é honrada** (`Locked`), como em todo gesto de autoria.
//!
//! ⚠️ O toggle `Physics` do transporte **não** é condição, exatamente como na
//! IK: este gesto não dá passo nenhum no solver, e exigi-lo obrigaria o artista
//! a armar a simulação para *não* simular.
//!
//! # A escrita é a MESMA porta da IK
//!
//! [`crate::body_pose::write_world_pose`] converte mundo → local contra o pai,
//! preserva escala e skew e recusa o não-finito. Uma segunda conversão aqui
//! divergiria no primeiro corpo parenteado — o defeito que o W5 levou quatro
//! waves para encontrar porque toda fixture usava corpo-raiz.

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
