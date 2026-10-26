//! ⭐ **Nascer uma âncora** — o gesto *Add IK* / *Look At*: o alvo na ponta do osso e o item (IK ou
//! Look At) no fim da pilha dele. Mora aqui (e não na família do esqueleto) porque a cena do osso da `ph2d-app-vec`
//! também a cria, e uma família não chama outra.

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Ik, Kind, LookAt};
use ph2d_ecs::SimWorld;
use ph2d_skeleton_ecs::{Bone, DEFAULT_CHAIN};
use ph2d_skeleton_live::goal::{aponta_com, governed, side_for_chain, spawn_target};

use crate::Constraints;
use crate::ik_item::{Ancora, ancora_de, por_ik};

/// **Cria a IK** de `bone` (corrente [`DEFAULT_CHAIN`], força 1) com o alvo pousado na ponta, e
/// devolve o alvo. `None` se `bone` não é osso ou já tem âncora (duas a puxar a mesma corrente).
///
/// O lado da dobra é CAPTURADO da pose que o artista já posou (`side_for_chain`), não escolhido: a
/// IK nasce byte-idêntica à pose e defende-a ao esticar o membro.
pub fn add_ik(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    if !livre(sim, bone) {
        return None;
    }
    let (alvo, id) = spawn_target(sim, bone)?;
    let ik = Ik {
        chain: DEFAULT_CHAIN,
        bend: side_for_chain(sim, bone, DEFAULT_CHAIN),
        ..Ik::default()
    };
    por_ik(sim.world_mut(), bone, id, ik, 1.0);
    Some(alvo)
}

/// ⭐ **O verbo *Look At*** — um `Kind::LookAt` no fim da pilha, o alvo na ponta: o osso olha para ele
/// (a porta única de «olhar», objectos e ossos; a IK começa nos 2 ossos). Nasce sem mover nada.
pub fn add_look_at(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    if !livre(sim, bone) {
        return None;
    }
    let (alvo, id) = spawn_target(sim, bone)?;
    let mut e = sim.world_mut().entity_mut(bone);
    if e.get::<Constraints>().is_none() {
        e.insert(Constraints::default());
    }
    e.get_mut::<Constraints>()
        .expect("acabada de pôr")
        .push(Kind::LookAt(LookAt::default()), Some(id));
    Some(alvo)
}

fn livre(sim: &SimWorld, bone: Entity) -> bool {
    sim.world().get::<Bone>(bone).is_some() && ancora_de(sim.world(), bone).is_none()
}

/// ⭐ **A âncora deste osso OLHA (e não alcança)?** — um Look At, ou uma IK cuja corrente RESOLVIDA é
/// de um osso ([`aponta_com`]). Só então o desvio é lido e o painel o mostra.
#[must_use]
pub fn aponta(sim: &SimWorld, bone: Entity) -> bool {
    match ancora_de(sim.world(), bone) {
        Some((_, _, Ancora::LookAt(_))) => true,
        Some((_, _, Ancora::Ik(ik))) => aponta_com(governed(sim, bone, ik.chain).len()),
        None => false,
    }
}
