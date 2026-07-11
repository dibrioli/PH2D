//! A ponte entre o documento Flip e a árvore do editor (ADR-0113), espelhando
//! [`crate::vec_entities`] (o vetor).
//!
//! Cada [`ph2d_flip::FlipObject`] tem uma entidade ECS que o referencia
//! (`FlipObjectRef`). O documento é dono das **camadas/frames/desenhos**; a
//! entidade é dona da **identidade e do lugar na árvore** — nome, visibilidade,
//! trava, pai, ordem. É por isso que um objeto Flip pode ser filho de um grupo,
//! e o gizmo/parentesco/agrupamento (compartilhados) valem pra ele sem nada novo.
//!
//! O único invariante mantido aqui: **um objeto ⟺ uma entidade**. Objeto novo ⇒
//! entidade spawnada. Objeto apagado ⇒ entidade despawnada. Entidade apagada
//! pela Hierarquia ⇒ objeto removido do documento. Tudo em [`sync`].

use ph2d_ecs::{Entity, FlipObjectRef, Name, RootOrder, SimWorld, Transform};
use ph2d_flip::{FlipDoc, FlipObjectId};
use std::collections::BTreeMap;

/// `FlipObjectId` → `Entity::to_bits()`. Autoritativo: só ele decide se um objeto
/// perdeu a entidade (ou vice-versa), então nem respawn fantasma nem objeto
/// órfão acontecem.
pub(crate) type FlipEntityMap = BTreeMap<FlipObjectId, u64>;

/// Reconcilia documento e árvore. Chamado **antes** de ler ordem/visibilidade/
/// seleção, para o frame ver um estado consistente. Três direções, nesta ordem:
/// 1. entidade sumiu (Delete na Hierarquia) ⇒ apaga o objeto;
/// 2. objeto sumiu (Delete no canvas) ⇒ despawna a entidade;
/// 3. objeto novo ⇒ spawna a entidade, no topo da ordem de raiz.
pub(crate) fn sync(sim: &mut SimWorld, doc: &mut FlipDoc, map: &mut FlipEntityMap) {
    // 1. Entidades que a Hierarquia apagou levam o objeto junto.
    let vanished: Vec<FlipObjectId> = map
        .iter()
        .filter(|(_, bits)| sim.world().get_entity(Entity::from_bits(**bits)).is_err())
        .map(|(&id, _)| id)
        .collect();
    for id in vanished {
        doc.remove_object(id);
        map.remove(&id);
    }

    // 2. Objetos que sumiram do documento levam a entidade junto.
    let dead: Vec<(FlipObjectId, u64)> = map
        .iter()
        .filter(|(id, _)| !doc.objects().iter().any(|o| o.id == **id))
        .map(|(&id, &bits)| (id, bits))
        .collect();
    for (id, bits) in dead {
        if let Ok(e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
            e.despawn();
        }
        map.remove(&id);
    }

    // 3. Objetos novos ganham entidade, no topo da ordem de raiz. O nome da
    //    entidade espelha o nome do objeto (a Hierarquia renomeia como qualquer
    //    entidade).
    let missing: Vec<(FlipObjectId, String)> = doc
        .objects()
        .iter()
        .filter(|o| !map.contains_key(&o.id))
        .map(|o| (o.id, o.name.clone()))
        .collect();
    if missing.is_empty() {
        return;
    }
    let mut next_order = next_root_order(sim);
    for (id, name) in missing {
        let e = sim.world_mut().spawn((
            Transform::default(),
            Name::new(name),
            FlipObjectRef(id.0),
            RootOrder(next_order),
        ));
        map.insert(id, e.id().to_bits());
        next_order = next_order.saturating_add(1);
    }
}

/// Reconstrói o mapa objeto↔entidade **a partir do mundo** — varre cada
/// `FlipObjectRef` e devolve `FlipObjectId → Entity::to_bits()`.
///
/// É o que um restore (undo ou load de projeto) precisa ANTES do primeiro
/// [`sync`]: o mapa é runtime-only e não é serializado; sem o rebuild, o `sync`
/// veria o mapa vazio e trataria cada objeto restaurado como novo — spawnando um
/// SEGUNDO conjunto de entidades e deixando as restauradas órfãs.
#[must_use]
pub(crate) fn rebuild_map(sim: &mut SimWorld) -> FlipEntityMap {
    let mut map = FlipEntityMap::new();
    let mut q = sim.world_mut().query::<(Entity, &FlipObjectRef)>();
    for (e, fo) in q.iter(sim.world()) {
        map.insert(FlipObjectId(fo.0), e.to_bits());
    }
    map
}

/// O próximo `RootOrder` livre (o maior em uso + 1). `RootOrder(u32::MAX)` é o
/// "sem ordem" das raízes que nunca receberam uma, então não conta. Idêntico ao
/// de `vec_entities` (a árvore é única — a ordem de raiz é compartilhada entre os
/// meios).
fn next_root_order(sim: &mut SimWorld) -> u32 {
    let mut q = sim.world_mut().query::<&RootOrder>();
    let max = q
        .iter(sim.world())
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max();
    max.map_or(0, |m| m.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (SimWorld, FlipDoc, FlipEntityMap) {
        (SimWorld::default(), FlipDoc::new(), FlipEntityMap::new())
    }

    fn bits(map: &FlipEntityMap, id: FlipObjectId) -> Entity {
        Entity::from_bits(map[&id])
    }

    /// O invariante da ponte: um objeto ⟺ uma entidade. Nas duas direções, e o
    /// sync é idempotente (rodar de novo não spawna fantasma).
    #[test]
    fn sync_keeps_one_entity_per_object_in_both_directions() {
        let (mut sim, mut doc, mut map) = setup();
        let a = doc.push_object("A");
        let b = doc.push_object("B");

        sync(&mut sim, &mut doc, &mut map);
        assert_eq!(map.len(), 2);
        let ea = bits(&map, a);
        assert!(
            sim.world()
                .get::<FlipObjectRef>(ea)
                .is_some_and(|v| v.0 == a.0)
        );
        assert!(sim.world().get::<Name>(ea).is_some());

        // Idempotente.
        sync(&mut sim, &mut doc, &mut map);
        assert_eq!(map.len(), 2);
        assert_eq!(bits(&map, a), ea, "não respawnou");

        // Objeto apagado no documento ⇒ entidade despawnada.
        doc.remove_object(b);
        sync(&mut sim, &mut doc, &mut map);
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key(&b));

        // Entidade apagada pela Hierarquia ⇒ objeto removido do documento.
        sim.world_mut().despawn(ea);
        sync(&mut sim, &mut doc, &mut map);
        assert!(map.is_empty());
        assert!(doc.objects().is_empty(), "o objeto foi junto");
    }

    /// A entidade nasce com o NOME do objeto e um `RootOrder` crescente.
    #[test]
    fn new_objects_get_name_and_root_order() {
        let (mut sim, mut doc, mut map) = setup();
        doc.push_object("Hero");
        let second = doc.push_object("Sidekick");
        sync(&mut sim, &mut doc, &mut map);

        let e2 = bits(&map, second);
        assert_eq!(
            sim.world().get::<Name>(e2).map(|n| n.as_str().to_owned()),
            Some("Sidekick".to_owned())
        );
        // Ordens de raiz distintas (o 2º entra acima do 1º).
        let orders: Vec<u32> = map
            .values()
            .map(|&b| {
                sim.world()
                    .get::<RootOrder>(Entity::from_bits(b))
                    .unwrap()
                    .0
            })
            .collect();
        assert_eq!(orders.len(), 2);
        assert_ne!(orders[0], orders[1]);
    }

    /// rebuild_map reconstrói a ponte a partir dos `FlipObjectRef` do mundo (o que
    /// um restore precisa), e o `sync` seguinte vira no-op (não duplica).
    #[test]
    fn rebuild_map_then_sync_is_a_noop() {
        let (mut sim, mut doc, mut map) = setup();
        doc.push_object("A");
        doc.push_object("B");
        sync(&mut sim, &mut doc, &mut map);

        // Simula um restore: o mapa runtime some, mas as entidades (com
        // FlipObjectRef) continuam. rebuild_map o reconstrói.
        let rebuilt = rebuild_map(&mut sim);
        assert_eq!(rebuilt.len(), 2);
        let mut map2 = rebuilt;
        let before = map2.len();
        sync(&mut sim, &mut doc, &mut map2);
        assert_eq!(map2.len(), before, "sync não spawnou nada");
        // Uma entidade por objeto (contagem de FlipObjectRef == objetos).
        let ref_count = {
            let mut q = sim.world_mut().query::<&FlipObjectRef>();
            q.iter(sim.world()).count()
        };
        assert_eq!(ref_count, doc.objects().len());
    }
}
