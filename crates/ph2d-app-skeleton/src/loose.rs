//! ⭐⭐ **A RAIZ SOLTA GANHA UM ESQUELETO** (A14) — a porta dos ossos que não têm objecto: os de um
//! projecto de antes do A14, os das cenas que os criam à mão, e o primeiro *Create* numa cena sem
//! esqueleto. É a porta e não uma migração (o precedente da `line/UIUX`: a forma solta de um
//! projecto antigo ganha o seu objecto no 1.º quadro).
//!
//! O esqueleto nasce no lugar da raiz na Hierarquia (o mesmo pai, a mesma ordem de raiz) e com a
//! ORIGEM na cabeça dela (A15): esqueleto em `T` = translação local da raiz, raiz em `0` — o
//! `Transform::compose` dá o mesmo mundo AO BIT (medido, fila §F65). Só onde é exacto: uma raiz
//! com repouso guardado noutro sítio fica com o esqueleto na identidade (rebasear um valor guardado
//! por `L − T` sai do bit).
//!
//! ⭐ **A17 — a raiz com a POSIÇÃO na timeline** (o dono, 07/10): as tracks da posição mudam para o
//! esqueleto (a linha «Skeleton» da timeline), que fica na cabeça dela e segue as keys ao bit; a
//! rotação, a escala e o skew ficam no osso. O repouso da posição vai com a track: a raiz repousa em
//! `0` dentro do esqueleto, então *Repor* devolve a direcção, a escala e o skew e deixa o osso onde
//! a timeline o põe. Onde mover não é exacto (Time Remap, auto-orient, fórmula que lê a posição
//! pelo nome — `ph2d_timeline::PlaceMoveRefusal`) fica a lei de antes: esqueleto na identidade.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, StableId, Transform};
use ph2d_skeleton_ecs::{Bone, BoneRest, Skeleton};
use ph2d_timeline::{PlaceMove, TimelineDoc, TimelineState, WireId};

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

/// A origem do esqueleto da `raiz` (a translação local dela) e se as tracks da posição vão com
/// ela; `None` = a identidade.
fn origem(sim: &SimWorld, doc: &TimelineDoc, raiz: Entity) -> Option<(ph2d_core::Vec2, bool)> {
    let w = sim.world();
    let l = w.get::<Transform>(raiz)?.translation;
    if doc.has_place_track(raiz.to_bits()) {
        let nome = w.get::<Name>(raiz).map(Name::as_str);
        return doc
            .place_move_refusal(raiz.to_bits(), nome)
            .is_none()
            .then_some((l, true));
    }
    let repouso_noutro_sitio = w
        .get::<BoneRest>(raiz)
        .is_some_and(|r| r.translation != [l.x, l.y]);
    (!repouso_noutro_sitio).then_some((l, false))
}

/// ⭐ Põe um esqueleto por cima de cada raiz solta. Devolve os esqueletos criados (vazio = nada a
/// fazer, o caso de todo quadro). As tracks da posição de uma raiz animada mudam para o esqueleto
/// (`TimelineState::move_place`, documento e histórico).
pub fn adopt_loose_roots(sim: &mut SimWorld, timeline: &mut TimelineState) -> Vec<u64> {
    let raizes = loose_roots(sim);
    let mut novos = Vec::with_capacity(raizes.len());
    let mut lugares = Vec::new();
    for raiz in raizes {
        let name = ph2d_unique_name::unique_name(sim, ph2d_i18n::tr("object_add.skeleton"));
        let pai = sim.world().get::<ChildOf>(raiz).map(|c| c.parent());
        let ordem = sim.world().get::<RootOrder>(raiz).copied();
        let mut no_esqueleto = Transform::IDENTITY;
        let origem = origem(sim, &timeline.doc, raiz);
        if let Some((t, _)) = origem {
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
        if matches!(origem, Some((_, true))) {
            lugares.push((raiz, esq));
        }
    }
    if !novos.is_empty() {
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_missing_root_order(sim.world_mut());
    }
    let id = |e: Entity| {
        sim.world()
            .get::<StableId>(e)
            .map_or(WireId::NULL, |s| WireId(s.0))
    };
    for (raiz, esq) in lugares {
        let mv = PlaceMove {
            home: id(raiz),
            owner: id(esq),
        };
        timeline.move_place(raiz.to_bits(), esq.to_bits(), mv);
    }
    novos
}

#[cfg(test)]
#[path = "loose_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "loose_track_tests.rs"]
mod track_tests;
