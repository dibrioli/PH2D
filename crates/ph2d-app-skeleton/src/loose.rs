//! ⭐⭐ **A RAIZ SOLTA GANHA UM ESQUELETO** (A14) — a porta dos ossos que não têm objecto: os de um
//! projecto de antes do A14, os das cenas que os criam à mão, e o primeiro *Create* numa cena sem
//! esqueleto. É a porta e não uma migração (o precedente da `line/UIUX`: a forma solta de um
//! projecto antigo ganha o seu objecto no 1.º quadro).
//!
//! O esqueleto nasce no lugar da raiz na Hierarquia (o mesmo pai, a mesma ordem de raiz) e com a
//! ORIGEM na cabeça dela (A15): esqueleto em `T` = translação local da raiz, raiz em `0` — o
//! `Transform::compose` dá o mesmo mundo AO BIT (medido, fila §F65). Só onde é exacto: uma raiz com
//! repouso guardado noutro sítio ou com a posição na timeline fica com o esqueleto na identidade
//! (rebasear um valor guardado por `L − T` sai do bit).

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::{Bone, BoneRest, Skeleton};
use ph2d_timeline::{PropKind, TimelineDoc};

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

/// A origem do esqueleto da `raiz`: a translação local dela, se movê-la para o esqueleto é exacto
/// para TODO valor guardado da posição (o repouso e a timeline); senão a identidade.
fn origem(sim: &SimWorld, doc: &TimelineDoc, raiz: Entity) -> Option<ph2d_core::Vec2> {
    let l = sim.world().get::<Transform>(raiz)?.translation;
    let repouso_noutro_sitio = sim
        .world()
        .get::<BoneRest>(raiz)
        .is_some_and(|r| r.translation != [l.x, l.y]);
    let animada = [
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Position,
    ]
    .into_iter()
    .any(|p| doc.binding_for(raiz.to_bits(), p).is_some());
    (!repouso_noutro_sitio && !animada).then_some(l)
}

/// ⭐ Põe um esqueleto por cima de cada raiz solta. Devolve os esqueletos criados (vazio = nada a
/// fazer, o caso de todo quadro). `doc` diz quais raízes têm a posição animada.
pub fn adopt_loose_roots(sim: &mut SimWorld, doc: &TimelineDoc) -> Vec<u64> {
    let raizes = loose_roots(sim);
    let mut novos = Vec::with_capacity(raizes.len());
    for raiz in raizes {
        let name = ph2d_unique_name::unique_name(sim, ph2d_i18n::tr("object_add.skeleton"));
        let pai = sim.world().get::<ChildOf>(raiz).map(|c| c.parent());
        let ordem = sim.world().get::<RootOrder>(raiz).copied();
        let mut no_esqueleto = Transform::IDENTITY;
        if let Some(t) = origem(sim, doc, raiz) {
            no_esqueleto.translation = t;
            let zero = ph2d_core::Vec2::new(0.0, 0.0);
            let mut r = sim.world_mut().entity_mut(raiz);
            if let Some(mut t) = r.get_mut::<Transform>() {
                t.translation = zero;
            }
            if let Some(mut rest) = r.get_mut::<BoneRest>() {
                rest.translation = [0.0, 0.0];
            }
        }
        let mut esq = sim
            .world_mut()
            .spawn((no_esqueleto, Name::new(name), Skeleton));
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
