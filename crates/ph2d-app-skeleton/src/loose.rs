//! ⭐⭐ **A RAIZ SOLTA GANHA UM ESQUELETO** (A14) — a porta dos ossos que não têm objecto: os de um
//! projecto de antes do A14, os das cenas que os criam à mão, e o primeiro *Create* numa cena sem
//! esqueleto. É a porta e não uma migração (o precedente da `line/UIUX`: a forma solta de um
//! projecto antigo ganha o seu objecto no 1.º quadro).
//!
//! O esqueleto nasce na IDENTIDADE e no lugar da raiz na Hierarquia (o mesmo pai, a mesma ordem de
//! raiz) ⇒ o mundo de cada osso não muda AO BIT.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::{Bone, Skeleton};

/// As raízes de corrente sem esqueleto: osso cujo pai não é osso (ou não existe) e que nenhum
/// esqueleto possui.
fn loose_roots(sim: &SimWorld) -> Vec<Entity> {
    let w = sim.world();
    let mut v: Vec<Entity> = w
        .iter_entities()
        .filter(|er| er.contains::<Bone>())
        .map(|er| er.id())
        .filter(|&e| {
            let pai_osso = w
                .get::<ChildOf>(e)
                .is_some_and(|c| w.get::<Bone>(c.parent()).is_some());
            !pai_osso && ph2d_skeleton_ecs::skeleton_of(w, e).is_none()
        })
        .collect();
    v.sort_by_key(|e| e.to_bits());
    v
}

/// ⭐ Põe um esqueleto por cima de cada raiz solta. Devolve os esqueletos criados (vazio = nada a
/// fazer, o caso de todo quadro).
pub fn adopt_loose_roots(sim: &mut SimWorld) -> Vec<u64> {
    let raizes = loose_roots(sim);
    let mut novos = Vec::with_capacity(raizes.len());
    for raiz in raizes {
        let name = ph2d_unique_name::unique_name(sim, ph2d_i18n::tr("object_add.skeleton"));
        let pai = sim.world().get::<ChildOf>(raiz).map(|c| c.parent());
        let ordem = sim.world().get::<RootOrder>(raiz).copied();
        let mut esq = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new(name), Skeleton));
        match (pai, ordem) {
            (Some(p), _) => {
                esq.insert(ChildOf(p));
            }
            (None, Some(o)) => {
                esq.insert(o);
            }
            (None, None) => {}
        }
        let esq = esq.id();
        let mut r = sim.world_mut().entity_mut(raiz);
        r.remove::<RootOrder>();
        r.insert(ChildOf(esq));
        novos.push(esq.to_bits());
    }
    if !novos.is_empty() {
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
    }
    novos
}

#[cfg(test)]
#[path = "loose_tests.rs"]
mod tests;
