//! **POSAR ARRASTANDO A PONTA** (W-IK) — o gesto de cinemática inversa.
//!
//! Irmão do [`crate::body_grab`], e a diferença entre os dois é o RELÓGIO:
//!
//! | | a MÃO (W-Grab) | a POSE (W-IK) |
//! |---|---|---|
//! | relógio | ANDANDO | **PARADO** |
//! | o que o gesto escreve | velocidade, pelo solver | o **`Transform` AUTORADO** |
//! | o que sobra | nada (é cutucar) | a pose, no documento e no undo |
//!
//! As duas metades são irmãs de propósito, exatamente como o `body_grab` e o
//! `joint_rig_drag`: tocando, a pose é do SOLVER e autorar não faria sentido (o
//! readback sobrescreve no mesmo frame); parado, a pose é do DOCUMENTO e uma
//! mola não faria sentido (nada a integra). O predicado é UM
//! (`JointTool::gesture`) e mora ao lado do que decide o alcance do arrasto,
//! para as duas metades do Alt não poderem discordar.
//!
//! # As condições, e a que NÃO está aqui
//!
//! 1. **O modo de joint em mãos é o IK** — `JointTool::gesture()`, a porta
//!    única (e é ela que faz o **Alt** suprimir o gesto: Alt significa *leve o
//!    rig inteiro*, que é um arrasto).
//! 2. **O relógio está PARADO.** Ver acima.
//! 3. **O corpo é dinâmico e pertence a uma cadeia rígida** — respondido pelo
//!    `ik_plan` da ponte, não por uma cópia da regra aqui.
//! 4. **A trava é honrada** (`Locked`), como em todo gesto de autoria.
//!
//! ⚠️ **O toggle `Physics` do transporte NÃO é condição, e isso é decisão.** As
//! três ferramentas do W-Hand precisam dele porque empurram o SOLVER, e sem
//! passo nada se move. Posar não dá passo nenhum: a árvore é resolvida e o
//! resultado é escrito no `Transform`. Exigi-lo aqui obrigaria o artista a armar
//! a simulação para *não* simular — e o `hold` da ponte reconcilia os joints de
//! qualquer forma, então a árvore existe com o toggle desmarcado.
//!
//! # O que o gesto escreve, e onde a conversão mora
//!
//! O solver devolve pose de **MUNDO** e o `Transform` é **LOCAL** (W5). A
//! conversão é `Transform::inverse_compose` contra o mundo do PAI — a mesma que
//! o `readback` da ponte usa —, e recusar quando ela não é finita não é higiene:
//! um `NaN` num `Transform` envenena o `GlobalTransform` da subárvore inteira.
//!
//! A **escala não é tocada.** A IK resolve ângulos e posições; esticar um elo
//! não é uma coisa que uma junta faça, e escrever a escala aqui apagaria o que o
//! artista autorou por um número que o solver nem produziu.
//!
//! # O undo é o global, e é UM passo
//!
//! Nada aqui grava história: o `post_frame_undo` compara o estado no fim do
//! frame e é suprimido enquanto o botão está preso, então o gesto inteiro —
//! quantos Moves tiver — vira um passo só. É a mesma máquina do dot de âncora do
//! joint (W-JointAnchor), e é por isso que este arquivo não tem uma linha sobre
//! undo.

use ph2d_ecs::{Entity, Transform, parent_world_transform, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;


/// **A porta única do press.** `true` = a pose pegou, e o chamador NÃO deve
/// abrir arrasto de gizmo nem mexer na seleção.
///
/// As condições 1 e 2 chegam como argumento (é o chamador que vê o relógio e a
/// ferramenta), a 3 é da ponte. A divisão é a mesma do `body_grab::take_hold`,
/// e pelo mesmo motivo: assim a decisão inteira é testável sem janela.
pub fn take_pose(
    physics: &mut PhysicsBridge,
    is_ik: bool,
    entity: Entity,
    playing: bool,
) -> bool {
    if playing || !is_ik {
        return false;
    }
    physics.ik_begin(entity)
}

/// **A cadeia segue o cursor.** No-op sem sessão, então roda em todo Move ao lado dos
/// outros `advance_*`. **Devolve `true` se AUTOROU.**
///
/// ⚠️ Devolve «autorou» ao contrário do `advance_body_grab` — e a diferença é
/// exatamente a que o cabeçalho descreve: aquele cutuca o solver e não deixa nada;
/// este AUTORA a cena, então o diff de undo tem de ver o quadro em que a pose mudou.
///
/// ⚠️ Era um `impl App` (W2/L2 Fase B). As `opts` entram por ARGUMENTO porque vivem no
/// `PhysicsState`, que é campo da `App` — e é o chamador que o tem.
pub fn advance_body_pose(
    physics: &mut PhysicsBridge,
    sim: &mut SimWorld,
    world: [f32; 2],
    opts: ph2d_physics_ecs::IkOptions,
) -> bool {
    if !physics.is_posing() {
        return false;
    }
    // O ângulo alvo é o que a ponta JÁ tem: com `Match` ligado, arrastar mantém a
    // atitude em vez de girar a ponta para um número que o mouse não tem. Lido do
    // `Transform` autorado, que é onde a pose vive.
    let tip_angle = physics
        .posing_tip()
        .and_then(|e| sim.world().get::<Transform>(e))
        .map_or(0.0, |t| t.rotation);
    let poses = physics.ik_move(world, tip_angle, opts);
    for (e, translation, rotation) in poses {
        write_world_pose(sim, e, translation, rotation);
    }
    true
}

/// Encerra o gesto. Idempotente — soltar sem ter posado é o caso de quase todo release
/// do app, e é por isso que ele mora no topo do handler junto do irmão da mão.
pub fn release_body_pose(physics: &mut PhysicsBridge) {
    physics.ik_end();
}

pub fn write_world_pose(
    sim: &mut ph2d_ecs::SimWorld,
    e: Entity,
    translation: [f32; 2],
    rotation: f32,
) {
    let parent = parent_world_transform(sim.world(), e);
    let Some(current) = sim.world().get::<Transform>(e).copied() else {
        return;
    };
    let mut world = current;
    world.translation.x = translation[0];
    world.translation.y = translation[1];
    world.rotation = rotation;
    let Some(local) = Transform::inverse_compose(parent, world) else {
        return;
    };
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        // Só o que a IK de fato resolveu. A escala e o skew ficam onde o artista
        // os deixou.
        t.translation = local.translation;
        t.rotation = local.rotation;
    }
}

#[cfg(test)]
#[path = "body_pose_tests.rs"]
mod tests;
