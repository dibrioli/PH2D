//! W11 do Kanban — **a cena responde aos cartões**: publica o que tem (objectos por `StableId`,
//! scripts, clips da timeline) para o verso do cartão resolver as ligações, e serve os pedidos
//! «abrir» da cena (seleccionar o objecto) e da timeline (ir ao trecho e seleccionar o dono). A
//! shell chama [`frame`] uma vez por quadro; as macros publica-as a família delas.

use ph2d_ecs::{Name, SimWorld, StableId, entity_with_stable_id};
use ph2d_editor_core::action_bus::{EditorAction, SelectModifier};
use ph2d_editor_core::kanban_links::{Catalog, Family, Target, publish_scene, take};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_script::ProjectScripts;
use ph2d_timeline::{StripSource, TimelineDoc, TimelineIntent};

/// O catálogo da cena agora: objectos com nome, scripts do projecto e os clips da pilha.
#[must_use]
pub fn catalog(sim: &mut SimWorld, tl: &TimelineDoc) -> Catalog {
    let world = sim.world_mut();
    let objects = world
        .query::<(&StableId, &Name)>()
        .iter(world)
        .map(|(s, n)| (s.0, n.0.clone()))
        .collect();
    let scripts = world
        .get_resource::<ProjectScripts>()
        .map(|lib| lib.iter().map(|(id, d)| (id.0, d.name.clone())).collect())
        .unwrap_or_default();
    let strips = tl
        .stack()
        .iter()
        .flat_map(|lane| &lane.strips)
        .filter_map(|s| {
            let name = match s.source {
                StripSource::Clip(i) => &tl.clips().get(usize::from(i))?.name,
                StripSource::Container(i) => &tl.containers().get(usize::from(i))?.name,
            };
            Some((s.id.0, name.clone()))
        })
        .collect();
    Catalog {
        objects,
        scripts,
        strips,
        ..Catalog::default()
    }
}

/// Selecciona o objecto `stable` como um clique (a Hierarquia segue a selecção). `false` = não existe.
fn select(sim: &SimWorld, hero: &mut HeroScreen, stable: u64) -> bool {
    let Some(e) = entity_with_stable_id(sim.world(), StableId(stable)) else {
        return false;
    };
    hero.bus.push(EditorAction::SelectSprite {
        entity_bits: e.to_bits(),
        modifier: SelectModifier::Replace,
    });
    true
}

/// ⭐ Um quadro: publica o catálogo e serve os pedidos da cena e da timeline (a aba da cena fica
/// activa, para o artista ver o que o cartão abriu).
pub fn frame(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &TimelineDoc,
    intents: &mut Vec<TimelineIntent>,
) {
    publish_scene(catalog(sim, tl));
    let mut shown = false;
    for t in take(Family::Scene)
        .into_iter()
        .chain(take(Family::Timeline))
    {
        match t {
            Target::Object(o) => shown |= select(sim, hero, o),
            Target::Span { object, from, .. } => {
                intents.push(TimelineIntent::SeekFrame(from));
                if let Some(o) = object {
                    select(sim, hero, o);
                }
                shown = true;
            }
            _ => {}
        }
    }
    if shown {
        hero.documents.activate(None);
    }
}

#[cfg(test)]
#[path = "kanban_scene_tests.rs"]
mod tests;
