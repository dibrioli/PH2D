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

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef, Visibility};
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

    // 3. Paths novos ganham entidade — **no FIM da lista**, que desde a lei de Godot
    //    (2026-08-04) é a FRENTE, e é onde o artista acabou de desenhar.
    //
    //    Enio: *"quando se cria um objeto novo ele vai para o último abaixo na hierarquia, mas
    //    aqui no nosso ele aparece no topo"*. Não é uma segunda decisão: é a MESMA — enquanto a
    //    primeira linha era a da frente, nascer na frente obrigava a abrir espaço no começo e a
    //    empurrar toda raiz um lugar para baixo; com a lista a correr fundo → topo, nascer na
    //    frente é simplesmente **apender**.
    //
    //        RootOrder BAIXO  → primeira linha  → FUNDO
    //        RootOrder ALTO   → última linha    → FRENTE
    let missing: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| !map.contains_key(id))
        .collect();
    if missing.is_empty() {
        return;
    }
    // ⚠️ O `next_root_order` é a porta única de *"qual é o próximo lugar livre?"* — a mesma que o
    // Flip e o envelope usam. Sem o shift, o custo deixou de ser O(raízes) por forma nova.
    let base = next_root_order(sim);
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
        let order = base.saturating_add(u32::try_from(k).unwrap_or(0));
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
pub(crate) fn next_root_order(sim: &mut SimWorld) -> u32 {
    let mut q = sim.world_mut().query::<&RootOrder>();
    let max = q
        .iter(sim.world())
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max();
    max.map_or(0, |m| m.saturating_add(1))
}

/// **O estado de vista que um GESTO deve consultar** — o que a árvore diz AGORA, mais os fatos
/// que o último DESENHO derivou.
///
/// ⚠️ Existe porque as duas metades têm relógios diferentes, e as duas estão certas:
///
/// - *escondido* e *travado* saem do mundo **deste instante** (o artista pode ter acabado de
///   apagar o olhinho, e o clique tem de honrá-lo já);
/// - os **intervalos das molduras**, as **poses do auto layout** e **quem foi ABSORVIDO por uma
///   booleana viva** são resultado do passe de desenho — e o artista clica no que está na TELA,
///   que é o último frame desenhado.
///
/// Sem esta fusão o gesto vê as três listas VAZIAS e decide como se nenhuma moldura existisse:
/// a moldura ganha o clique dos próprios filhos, o hit-test procura cada forma colocada no
/// lugar de onde ela saiu, e um operando absorvido fica inalcançável pelo canvas.
#[must_use]
pub(crate) fn view_state_for_pick(
    sim: &SimWorld,
    map: &VecEntityMap,
    derived: &VecViewState,
) -> VecViewState {
    let mut v = view_state(sim, map);
    v.clips.clone_from(&derived.clips);
    v.poses.clone_from(&derived.poses);
    v.absorbed.clone_from(&derived.absorbed);
    v
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

/// Teto de profundidade das caminhadas de ancestral (defesa, não limite de produto).
/// Partilhado com o irmão [`selection`] — uma árvore corrompida tem UMA profundidade máxima.
pub(crate) const MAX_DEPTH: usize = 64;

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

/// **A ancestralidade e o que uma SELEÇÃO significa** — módulo irmão, pelo teto de 600 LOC
/// da shell. O corte é por assunto: aqui em cima mora o que a ponte MANTÉM (a identidade
/// path ⟺ entidade, a ordem, o que a árvore esconde); ali, o que a árvore RESPONDE — *quem é
/// o objeto que este clique nomeia, e o que selecioná-lo significa*.
#[path = "vec_entities_selection.rs"]
mod selection;
pub(crate) use selection::{object_selection_for, selection_paths, subtree_paths, top_ancestor};

#[cfg(test)]
fn setup() -> (SimWorld, VecScene, VecEntityMap) {
    (SimWorld::default(), VecScene::new(), VecEntityMap::new())
}

#[cfg(test)]
fn bits(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(map[&id])
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

    /// **Duplicar uma forma dá ao clone o PRÓPRIO path, e o `sync` cunha UMA entidade para ele.**
    ///
    /// ⚠️ É o invariante que a row **Duplicate** da Hierarchy violava: ela clonava a ENTIDADE
    /// (Transform + Name + ChildOf) e não o path, então o clone nascia sem `VecPathRef` — uma
    /// linha na Hierarchy sobre geometria nenhuma. E copiar o `VecPathRef` teria sido pior: duas
    /// entidades a apontar para o MESMO path, num mapa que é um-para-um.
    #[test]
    fn duplicating_a_shape_gives_the_copy_its_own_path_and_its_own_entity() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(
            map.len(),
            1,
            "o CONTROLE falhou: o sync nao cunhou a origem"
        );

        let mut history = ph2d_vec_edit::History::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        assert!(
            !history.can_undo(),
            "o CONTROLE falhou: o historico ja nascia sujo"
        );
        assert!(
            crate::input_dispatch::duplicate_vec_paths(
                &mut scene,
                &mut history,
                &mut pen,
                &[a],
                5.0,
                5.0
            ),
            "a porta recusou duplicar um path que existe"
        );
        sync(&mut sim, &mut scene, &mut map);

        assert_eq!(scene.paths().len(), 2, "o documento nao ganhou a copia");
        assert_eq!(map.len(), 2, "o sync nao cunhou UMA entidade para a copia");
        let copy = scene
            .paths()
            .iter()
            .map(|p| p.id)
            .find(|id| *id != a)
            .expect("a copia tem id proprio");
        // As duas entidades apontam para paths DIFERENTES — nenhuma aliasing.
        let (ea, ec) = (bits(&map, a), bits(&map, copy));
        assert_ne!(ea, ec, "a copia herdou a entidade da origem");
        let ra = sim
            .world()
            .get::<VecPathRef>(ea)
            .copied()
            .expect("origem sem ref");
        let rc = sim
            .world()
            .get::<VecPathRef>(ec)
            .copied()
            .expect("copia sem ref");
        assert_ne!(ra.0, rc.0, "as duas entidades apontam para o MESMO path");

        // ⚠️ **UM Ctrl+Z desfaz a cópia inteira.** O oráculo é o GESTO, não o comprimento da
        // pilha: sem esta metade, tirar o `push_undo` da porta passava na suíte INTEIRA (medido)
        // — duplicar ficava fora do Ctrl+Z, e o artista descobria isso com a forma já na tela.
        assert!(
            history.can_undo(),
            "duplicar nao gravou passo de undo nenhum"
        );
        let back = history.undo(&scene).expect("o passo de undo existe");
        assert_eq!(
            back.paths().len(),
            1,
            "um Ctrl+Z nao devolveu o documento ao estado de antes da copia"
        );
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
}
