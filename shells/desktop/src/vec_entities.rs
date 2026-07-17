//! A ponte entre o documento vetorial e a árvore do editor (ADR-0110).
//!
//! Cada `VecPath` tem uma entidade ECS que o referencia (`VecPathRef`). O
//! documento é dono da **geometria**; a entidade é dona da **identidade e do lugar
//! na árvore** — nome, visibilidade, trava, pai, ordem. É por isso que um path
//! vetorial pode ser filho de um sprite, e que um grupo é só uma entidade comum
//! com filhos: não há tipo de nó especial, e nada teve de ser inventado.
//!
//! Este módulo mantém o único invariante que a ponte exige: **um path ⟺ uma
//! entidade**. Path novo ⇒ entidade spawnada. Path apagado ⇒ entidade despawnada.
//! Entidade apagada pela Hierarquia ⇒ path removido do documento. Tudo em
//! [`sync`], uma vez por frame, antes de qualquer leitura.

use ph2d_ecs::{
    ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef, Visibility, Without,
};
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState};
use std::collections::BTreeMap;

/// `VecPathId` → `Entity::to_bits()`. Autoritativo: só ele decide se um path
/// perdeu a entidade (ou vice-versa), então nem um respawn fantasma nem um path
/// órfão podem acontecer.
pub(crate) type VecEntityMap = BTreeMap<VecPathId, u64>;

/// Nome inicial de um path novo. O usuário renomeia pela Hierarquia como qualquer
/// entidade; o id só garante unicidade no nascimento.
fn initial_name(id: VecPathId) -> String {
    format!("Path {id}")
}

/// Reconcilia documento e árvore. Chamado **antes** de ler ordem, visibilidade ou
/// seleção, para que o frame veja um estado consistente.
///
/// Três direções, nesta ordem:
/// 1. entidade sumiu (Delete na Hierarquia) ⇒ apaga o path;
/// 2. path sumiu (Delete no canvas / booleana / cut) ⇒ despawna a entidade;
/// 3. path novo ⇒ spawna a entidade, no topo da ordem de raiz.
pub(crate) fn sync(sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) {
    // 1. Entidades que a Hierarquia apagou levam o path junto.
    let vanished: Vec<VecPathId> = map
        .iter()
        .filter(|(_, bits)| sim.world().get_entity(Entity::from_bits(**bits)).is_err())
        .map(|(&id, _)| id)
        .collect();
    for id in vanished {
        scene.remove_path(id);
        map.remove(&id);
    }

    // 2. Paths que sumiram do documento levam a entidade junto.
    let dead: Vec<(VecPathId, u64)> = map
        .iter()
        .filter(|(id, _)| !scene.paths().iter().any(|p| p.id == **id))
        .map(|(&id, &bits)| (id, bits))
        .collect();
    for (id, bits) in dead {
        if let Ok(e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
            e.despawn();
        }
        map.remove(&id);
    }

    // 3. Paths novos ganham entidade — **na FRENTE**, que é onde o artista acabou de desenhar.
    //
    //    E "na frente" é o RootOrder mais BAIXO, não o mais alto. A Hierarquia lista as raízes
    //    em ordem CRESCENTE de `RootOrder` e a primeira linha é a da FRENTE (convenção
    //    Illustrator/Figma); a pilha de z é a projeção invertida dessa lista. Logo:
    //
    //        RootOrder BAIXO  → primeira linha  → FRENTE
    //        RootOrder ALTO   → última linha    → FUNDO
    //
    //    A versão anterior dava ao path novo o maior `RootOrder` ("entra no fim da lista"), e o
    //    comentário dela dizia "em cima" — mas o fim da lista é o FUNDO. Toda forma recém-criada
    //    nascia ATRÁS das que já estavam lá: o Enio viu isso no Blend (a 1ª forma gerada saiu
    //    debaixo do círculo que a originou), e vale igual para uma forma desenhada à mão.
    let missing: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| !map.contains_key(id))
        .collect();
    if missing.is_empty() {
        return;
    }
    // Abre espaço na frente de todo mundo — `missing.len()` lugares. É O(raízes), e uma cena de
    // editor tem dezenas: o preço de manter a ordem EXPLÍCITA (sem empate, sem `to_bits`).
    let shift = u32::try_from(missing.len()).unwrap_or(u32::MAX);
    let roots: Vec<(Entity, u32)> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &RootOrder), Without<ChildOf>>();
        q.iter(sim.world())
            .filter(|(_, r)| r.0 != u32::MAX)
            .map(|(e, r)| (e, r.0))
            .collect()
    };
    for (e, order) in roots {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(RootOrder(order.saturating_add(shift)));
        }
    }
    // O último da lista é o mais recente ⇒ ele fica com o menor número ⇒ mais à frente.
    for (k, id) in missing.iter().enumerate() {
        // **O nome passa pela porta do nome ÚNICO** (`name_unique::unique_name`), a mesma que o
        // import e o rename usam. `initial_name` é único entre PATHS (o id é), mas não no MUNDO:
        // basta o artista ter renomeado um sprite para "Path 3" e a próxima forma com id 3 nasce
        // homônima dele.
        //
        // Nome duplicado não é cosmético desde o W4.T6: a animação reencontra o objeto **pelo
        // nome** (`wire_id` = hash do `Name`), então dois homônimos fazem duas tracks colarem no
        // MESMO objeto — e a outra fica sem dono, em silêncio. O nome é identidade agora.
        let name = crate::name_unique::unique_name(sim, &initial_name(*id));
        let order = shift
            .saturating_sub(u32::try_from(k).unwrap_or(0))
            .saturating_sub(1);
        let e = sim.world_mut().spawn((
        Transform::default(),
        Name::new(name),
        VecPathRef(*id),
        RootOrder(order),
    ));
        map.insert(*id, e.id().to_bits());
    }
}

/// A ordem de **z** (a projeção da árvore) e quem a reescreve — módulo irmão, pelo teto de 600
/// LOC da shell.
#[path = "vec_zorder.rs"]
pub(crate) mod zorder;
pub(crate) use zorder::{restack, z_order};

/// Reconstrói o mapa path↔entidade **a partir do mundo** — varre cada `VecPathRef`
/// e devolve `VecPathId → Entity::to_bits()`.
///
/// É o que um restore (undo ou load de projeto) precisa ANTES do primeiro [`sync`].
/// O mapa é runtime-only e não é serializado; sem este rebuild, o `sync` veria o
/// mapa vazio e trataria cada path restaurado como novo — spawnando um SEGUNDO
/// conjunto de entidades e deixando as restauradas órfãs. Com o rebuild, as três
/// direções do `sync` viram no-op e a ponte fica consistente de graça.
#[must_use]
pub(crate) fn rebuild_map(sim: &mut SimWorld) -> VecEntityMap {
    let mut map = VecEntityMap::new();
    let mut q = sim.world_mut().query::<(Entity, &VecPathRef)>();
    for (e, vp) in q.iter(sim.world()) {
        map.insert(vp.0, e.to_bits());
    }
    map
}

/// O próximo `RootOrder` livre (o maior em uso + 1). `RootOrder(u32::MAX)` é o
/// "sem ordem" das raízes que nunca receberam uma, então não conta.
fn next_root_order(sim: &mut SimWorld) -> u32 {
    let mut q = sim.world_mut().query::<&RootOrder>();
    let max = q
        .iter(sim.world())
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max();
    max.map_or(0, |m| m.saturating_add(1))
}

/// O que a árvore esconde ou trava, com a herança já resolvida.
///
/// Visibilidade é `AND` da cadeia de pais (esconder um grupo esconde os filhos sem
/// tocar no flag deles). Trava usa `is_locked_for_edit`, o mesmo predicado do gizmo
/// de sprite: `Locked` no próprio, ou `GroupedChildren` em algum ancestral.
#[must_use]
pub(crate) fn view_state(sim: &SimWorld, map: &VecEntityMap) -> VecViewState {
    let w = sim.world();
    let mut view = VecViewState::default();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        if !visible_chain(w, e) {
            view.hidden.push(id);
        }
        if ph2d_ecs::is_locked_for_edit(w, e) {
            view.locked.push(id);
        }
    }
    view
}

/// `Visibility` do próprio E de cada ancestral.
fn visible_chain(w: &ph2d_ecs::World, entity: Entity) -> bool {
    let mut cur = Some(entity);
    // A cadeia é acíclica (`hierarchy_set_parent` recusa ciclos); o teto defende
    // contra um save corrompido.
    for _ in 0..MAX_DEPTH {
        let Some(e) = cur else { return true };
        if w.get::<Visibility>(e).is_some_and(|v| v.hidden) {
            return false;
        }
        cur = w.get::<ChildOf>(e).map(|c| c.parent());
    }
    true
}

/// Teto de profundidade das caminhadas de ancestral (defesa, não limite de produto).
const MAX_DEPTH: usize = 64;

/// O ancestral de topo de `entity` (ele mesmo, se já é raiz).
#[must_use]
pub(crate) fn top_ancestor(sim: &SimWorld, entity: Entity) -> Entity {
    let w = sim.world();
    let mut cur = entity;
    for _ in 0..MAX_DEPTH {
        match w.get::<ChildOf>(cur) {
            Some(c) => cur = c.parent(),
            None => break,
        }
    }
    cur
}

/// Todos os paths vetoriais na sub-árvore de `entity` (ele mesmo, se for um path),
/// **na ordem de z do documento** — a seleção fica estável entre frames.
#[must_use]
pub(crate) fn subtree_paths(sim: &SimWorld, scene: &VecScene, entity: Entity) -> Vec<VecPathId> {
    let w = sim.world();
    let mut found: Vec<VecPathId> = Vec::new();
    let mut stack = vec![entity];
    while let Some(e) = stack.pop() {
        if let Some(vp) = w.get::<VecPathRef>(e) {
            found.push(vp.0);
        }
        if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| found.contains(id))
        .collect()
}

/// A seleção de objeto que tocar `path` produz: as folhas vetoriais do ancestral
/// de topo dele. Fora de qualquer grupo isso é só ele — o comportamento de sempre.
#[must_use]
pub(crate) fn object_selection_for(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    path: VecPathId,
) -> Vec<VecPathId> {
    let Some(&bits) = map.get(&path) else {
        return vec![path];
    };
    let top = top_ancestor(sim, Entity::from_bits(bits));
    let leaves = subtree_paths(sim, scene, top);
    if leaves.is_empty() {
        vec![path]
    } else {
        leaves
    }
}

/// Agrupa as entidades `members` sob uma **entidade comum nova** (nome, `Transform`,
/// `RootOrder`), preservando a ordem. É o mesmo grupo que os sprites usam — por isso
/// ele aceita qualquer mistura de tipos (ADR-0110). Devolve o grupo, ou `None` se
/// sobrar menos de 2 ancestrais de topo distintos.
///
/// Agrupar normaliza para os ancestrais de topo: pegar um filho traz o grupo dele
/// junto (aninhamento), não o filho solto — a convenção de qualquer editor.
pub(crate) fn group_entities(sim: &mut SimWorld, members: &[u64], name: String) -> Option<u64> {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let t = top_ancestor(sim, e);
        if !tops.contains(&t) {
            tops.push(t);
        }
    }
    if tops.len() < 2 {
        return None;
    }
    let order = next_root_order(sim);
    let group = sim
        .world_mut()
        .spawn((Transform::default(), Name::new(name), RootOrder(order)))
        .id();
    // `Children` preserva a ordem de inserção → os membros entram na ordem de z.
    for t in tops {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(t) {
            e.remove::<RootOrder>();
            e.insert(ChildOf(group));
        }
    }
    Some(group.to_bits())
}

/// Dissolve os grupos de topo tocados por `members`. Um "grupo" aqui é o que a
/// árvore chama de grupo: uma entidade **sem geometria própria** (nem `VecPathRef`
/// nem sprite) que tem filhos. Um sprite com filhos é um pai, não um grupo — e
/// dissolvê-lo apagaria um objeto. Devolve quantos grupos sumiram.
pub(crate) fn ungroup_entities(sim: &mut SimWorld, members: &[u64]) -> usize {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let t = top_ancestor(sim, e);
        if t != e && !tops.contains(&t) && is_plain_group(sim, t) {
            tops.push(t);
        }
    }
    let mut order = next_root_order(sim);
    for g in &tops {
        let parent = sim.world().get::<ChildOf>(*g).map(|c| c.parent());
        let kids: Vec<Entity> = sim
            .world()
            .get::<ph2d_ecs::Children>(*g)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        for k in kids {
            if let Ok(mut e) = sim.world_mut().get_entity_mut(k) {
                match parent {
                    Some(p) => {
                        e.insert(ChildOf(p));
                    }
                    None => {
                        e.remove::<ChildOf>();
                        e.insert(RootOrder(order));
                        order = order.saturating_add(1);
                    }
                }
            }
        }
        if let Ok(e) = sim.world_mut().get_entity_mut(*g) {
            e.despawn();
        }
    }
    tops.len()
}

/// A entidade é um grupo puro: sem geometria própria, mas com filhos. Um sprite ou
/// um path com filhos NÃO é um grupo — dissolvê-lo apagaria um objeto.
fn is_plain_group(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    w.get::<VecPathRef>(e).is_none()
        && w.get::<ph2d_render::Sprite>(e).is_none()
        && w.get::<ph2d_ecs::Children>(e)
            .is_some_and(|c| !c.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::rectangle;

    fn setup() -> (SimWorld, VecScene, VecEntityMap) {
        (SimWorld::default(), VecScene::new(), VecEntityMap::new())
    }

    fn bits(map: &VecEntityMap, id: VecPathId) -> Entity {
        Entity::from_bits(map[&id])
    }

    /// O invariante da ponte: um path ⟺ uma entidade. Nas duas direções, e o
    /// sync é idempotente (rodar de novo não spawna fantasma).
    #[test]
    fn sync_keeps_one_entity_per_path_in_both_directions() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));

        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 2);
        let ea = bits(&map, a);
        assert!(sim.world().get::<VecPathRef>(ea).is_some_and(|v| v.0 == a));
        assert!(sim.world().get::<Name>(ea).is_some());

        // Idempotente.
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 2);
        assert_eq!(bits(&map, a), ea, "não respawnou");

        // Path apagado no canvas ⇒ entidade despawnada.
        scene.remove_path(b);
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key(&b));

        // Entidade apagada pela Hierarquia ⇒ path removido do documento.
        sim.world_mut().despawn(ea);
        sync(&mut sim, &mut scene, &mut map);
        assert!(map.is_empty());
        assert!(scene.paths().is_empty(), "o path foi junto");
    }

    /// **A forma nova nasce com nome ÚNICO no MUNDO** — não só entre formas.
    ///
    /// `initial_name(id)` é único entre paths (o id é), mas o mundo tem sprites e objetos Flip
    /// junto: basta o artista ter renomeado um sprite para "Path 1" e a próxima forma nasceria
    /// homônima dele. Desde o W4.T6 isso não é cosmético — a animação reencontra o objeto **pelo
    /// nome** (`wire_id` = hash do `Name`), então dois homônimos são dois donos para a mesma
    /// track. O nome é IDENTIDADE agora, e passa pela mesma porta que o import e o rename usam.
    #[test]
    fn a_new_shape_never_takes_a_name_the_world_already_uses() {
        let (mut sim, mut scene, mut map) = setup();
        // O artista renomeou um sprite exatamente com o nome que a próxima forma pediria.
        let first = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let squatter = initial_name(first);
        let mut scene2 = ph2d_vec_scene::VecScene::new();
        std::mem::swap(&mut scene, &mut scene2); // a forma ainda não entrou no mundo
        sim.world_mut()
            .spawn((Transform::default(), Name::new(squatter.clone())));
        std::mem::swap(&mut scene, &mut scene2);

        sync(&mut sim, &mut scene, &mut map);

        let names: Vec<String> = {
            let mut q = sim.world_mut().query::<&Name>();
            q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect()
        };
        assert_eq!(names.len(), 2);
        assert_ne!(
            names[0], names[1],
            "a forma nova pegou o nome do sprite — duas tracks colariam no mesmo objeto: {names:?}"
        );
        assert!(
            names.contains(&squatter),
            "e o nome do sprite continua o dele"
        );
    }

    /// Grupo é uma ENTIDADE COMUM: aceita path vetorial e sprite no mesmo saco, e
    /// agrupar normaliza para os ancestrais de topo (aninha, não reparenta o filho).
    #[test]
    fn group_entities_nests_and_accepts_mixed_types() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        // Um "sprite": entidade sem VecPathRef (o que importa é não ter geometria vetorial).
        let sprite = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("Spr")))
            .id();

        let inner = group_entities(&mut sim, &[map[&a], map[&b]], "in".into()).unwrap();
        assert_eq!(
            subtree_paths(&sim, &scene, Entity::from_bits(inner)),
            vec![a, b]
        );

        // Agrupa o path `a` (que já está em `inner`) com o SPRITE → `inner` aninha.
        let outer = group_entities(&mut sim, &[map[&a], sprite.to_bits()], "out".into())
            .expect("`a` traz o grupo `inner` junto");
        assert_eq!(
            top_ancestor(&sim, bits(&map, a)).to_bits(),
            outer,
            "o topo agora é o grupo de fora"
        );
        assert_eq!(
            sim.world()
                .get::<ChildOf>(Entity::from_bits(inner))
                .unwrap()
                .parent()
                .to_bits(),
            outer
        );
        assert_eq!(
            top_ancestor(&sim, sprite).to_bits(),
            outer,
            "o sprite entrou junto"
        );
        assert_eq!(
            subtree_paths(&sim, &scene, Entity::from_bits(outer)),
            vec![a, b]
        );

        // Menos de 2 topos distintos = no-op.
        assert_eq!(
            group_entities(&mut sim, &[map[&a], map[&b]], "x".into()),
            None
        );
    }

    /// Desagrupar dissolve só GRUPOS PUROS: um sprite (ou path) com filhos é um pai,
    /// não um grupo — dissolvê-lo apagaria um objeto.
    #[test]
    fn ungroup_dissolves_plain_groups_but_never_a_parent_object() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);

        let g = group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();
        assert_eq!(ungroup_entities(&mut sim, &[map[&a]]), 1);
        assert!(
            sim.world().get_entity(Entity::from_bits(g)).is_err(),
            "o grupo sumiu"
        );
        assert!(
            sim.world().get::<ChildOf>(bits(&map, a)).is_none(),
            "voltou pra raiz"
        );
        assert!(sim.world().get::<RootOrder>(bits(&map, a)).is_some());
        // Os paths continuam lá.
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(scene.paths().len(), 2);

        // `a` como PAI de `b` (não um grupo): ungroup não o dissolve.
        sim.world_mut()
            .entity_mut(bits(&map, b))
            .insert(ChildOf(bits(&map, a)));
        assert_eq!(ungroup_entities(&mut sim, &[map[&b]]), 0);
        assert!(
            sim.world().get_entity(bits(&map, a)).is_ok(),
            "o path-pai sobreviveu"
        );
    }

    /// Visibilidade e trava são HERDADAS, e o flag próprio do filho nunca é tocado.
    #[test]
    fn view_state_inherits_hiding_and_locking_from_the_ancestors() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let g =
            Entity::from_bits(group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap());

        assert_eq!(
            view_state(&sim, &map),
            VecViewState::default(),
            "tudo livre"
        );

        sim.world_mut()
            .entity_mut(g)
            .insert(Visibility { hidden: true });
        let v = view_state(&sim, &map);
        assert!(v.is_hidden(a) && v.is_hidden(b));
        assert!(
            sim.world().get::<Visibility>(bits(&map, a)).is_none(),
            "o flag do filho nunca foi tocado"
        );
        sim.world_mut().entity_mut(g).remove::<Visibility>();
        assert!(!view_state(&sim, &map).is_hidden(a), "reabrir devolve");

        // `GroupedChildren` no grupo trava os descendentes (predicado do gizmo).
        sim.world_mut()
            .entity_mut(g)
            .insert(ph2d_ecs::GroupedChildren);
        let v = view_state(&sim, &map);
        assert!(!v.is_pickable(a) && !v.is_pickable(b));
        assert!(!v.is_hidden(a), "travado não é escondido");
    }

    /// Clicar num filho seleciona o grupo inteiro; fora de grupo, só ele.
    #[test]
    fn object_selection_expands_to_the_top_group() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        let c = scene.push_path(rectangle([4.0, 0.0], [5.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(object_selection_for(&sim, &scene, &map, a), vec![a]);

        group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();
        assert_eq!(object_selection_for(&sim, &scene, &map, a), vec![a, b]);
        assert_eq!(
            object_selection_for(&sim, &scene, &map, c),
            vec![c],
            "solto"
        );
    }
}
