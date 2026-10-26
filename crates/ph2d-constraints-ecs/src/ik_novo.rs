//! ⭐ **Nascer uma IK** — o gesto *Add IK* / *Look At*: o alvo na ponta do osso e o item IK no fim da
//! pilha dele. Mora aqui (e não na família do esqueleto) porque a cena do osso da `ph2d-app-vec`
//! também a cria, e uma família não chama outra.

use bevy_ecs::entity::Entity;
use ph2d_constraints::Ik;
use ph2d_ecs::SimWorld;
use ph2d_skeleton_ecs::{Bone, DEFAULT_CHAIN};
use ph2d_skeleton_live::goal::{aponta_com, governed, side_for_chain, spawn_target};

use crate::ik_item::{ik_de, por_ik};

/// **Cria a IK** de `bone` (corrente [`DEFAULT_CHAIN`], força 1) com o alvo pousado na ponta, e
/// devolve o alvo. `None` se `bone` não é osso ou já tem item IK (duas a puxar a mesma corrente).
///
/// O lado da dobra é CAPTURADO da pose que o artista já posou (`side_for_chain`), não escolhido: a
/// IK nasce byte-idêntica à pose e defende-a ao esticar o membro.
pub fn add_ik(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    nasce(sim, bone, DEFAULT_CHAIN)
}

/// ⭐ **O verbo *Look At*** — a mesma IK com a corrente em UM, onde a lei do alcance APONTA.
pub fn add_look_at(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    nasce(sim, bone, 1)
}

fn nasce(sim: &mut SimWorld, bone: Entity, chain: u32) -> Option<Entity> {
    if sim.world().get::<Bone>(bone).is_none() || ik_de(sim.world(), bone).is_some() {
        return None;
    }
    let (alvo, id) = spawn_target(sim, bone)?;
    // O lado é o da corrente de nascimento do *Add IK* (o *Look At* herda-o, inerte com um osso).
    let bend = side_for_chain(sim, bone, DEFAULT_CHAIN);
    let ik = Ik {
        chain,
        bend,
        ..Ik::default()
    };
    por_ik(sim.world_mut(), bone, id, ik, 1.0);
    Some(alvo)
}

/// ⭐ **A IK deste osso APONTA (e não alcança)?** — a corrente RESOLVIDA é de um osso
/// ([`aponta_com`]). Só então o desvio (`offset`) é lido e o painel o mostra.
#[must_use]
pub fn aponta(sim: &SimWorld, bone: Entity) -> bool {
    ik_de(sim.world(), bone)
        .is_some_and(|(_, _, ik)| aponta_com(governed(sim, bone, ik.chain).len()))
}
