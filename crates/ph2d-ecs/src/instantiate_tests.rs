//! Os gates da cópia profunda (ADR-0164 / F4.2).

use super::{InstanceOf, deep_copy_subtree, remap_instance_of};
use crate::scene::{ComponentRegistry, register_ecs_components};
use crate::{
    ChildOf, Children, Entity, MasterRoot, Name, RootOrder, SiblingOrder, SimWorld, StableId,
    Transform, Visibility,
};
use ph2d_core::Vec2;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// Um mestre com filho e neta, e um nome por nível.
fn scene() -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 2.0)),
            Name::new("Ragdoll"),
            MasterRoot,
        ))
        .id();
    let torso = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 1.0)),
            Name::new("Torso"),
            Visibility { hidden: true },
            ChildOf(root),
        ))
        .id();
    let leg = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, -1.0)),
            Name::new("Leg"),
            ChildOf(torso),
        ))
        .id();
    crate::assign_missing_stable_ids(sim.world_mut());
    crate::assign_missing_root_order(sim.world_mut());
    crate::assign_missing_sibling_order(sim.world_mut());
    (sim, root, torso, leg)
}

fn name_of(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map(|n| n.0.clone())
        .unwrap_or_default()
}

/// ⭐ **A subárvore INTEIRA vem, com os valores dos componentes** — e a neta é o caso que
/// interessa (uma cópia de um nível deixaria a perna para trás).
#[test]
fn the_whole_subtree_comes_with_its_values() {
    let (mut sim, root, ..) = scene();
    let r = reg();
    let copy = deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copia");
    assert_eq!(copy.entities.len(), 3, "a copia perdeu um nivel");

    let w = sim.world();
    let kids: Vec<Entity> = w
        .get::<Children>(copy.root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    assert_eq!(kids.len(), 1, "a copia da raiz ficou sem o filho");
    let torso2 = kids[0];
    assert_eq!(name_of(&sim, torso2), "Torso");
    assert!(
        sim.world()
            .get::<Visibility>(torso2)
            .is_some_and(|v| v.hidden),
        "o VALOR do componente nao veio — a copia so' recriou o archetype"
    );
    let grandkids: Vec<Entity> = sim
        .world()
        .get::<Children>(torso2)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    assert_eq!(grandkids.len(), 1, "a NETA ficou por copiar");
    assert_eq!(name_of(&sim, grandkids[0]), "Leg");
    assert_eq!(
        sim.world()
            .get::<Transform>(grandkids[0])
            .expect("pose")
            .translation
            .y,
        -1.0
    );
}

/// ⭐ **A cópia tem IDENTIDADE NOVA, e o mapa liga as duas.**
///
/// ⚠️ É esta a razão de o `StableId` não ser registado: se ele viajasse no blob, a cópia
/// nasceria com a identidade do ORIGINAL e o mundo teria duas entidades a responder pelo mesmo
/// id — a corrupção que a F1 inteira existe para tornar impossível.
///
/// (Mutação: registar o `StableId` ⇒ este gate reprova nomeando o id repetido.)
#[test]
fn the_copy_gets_a_new_identity_and_the_map_links_them() {
    let (mut sim, root, torso, leg) = scene();
    let r = reg();
    let copy = deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copia");

    let mut seen = std::collections::BTreeSet::new();
    for (&src, &dst) in &copy.entities {
        let a = sim.world().get::<StableId>(src).expect("id do original").0;
        let b = sim.world().get::<StableId>(dst).expect("id da copia").0;
        assert_ne!(a, b, "a copia herdou a identidade do original");
        assert!(seen.insert(b), "duas copias com o mesmo id: {b}");
        assert_eq!(
            copy.stable_ids.get(&a),
            Some(&b),
            "o mapa nao liga {a} a' copia dele"
        );
    }
    assert_eq!(copy.stable_ids.len(), 3);
    let _ = (torso, leg);
}

/// ⚠️ **A raiz da cópia NÃO herda a ordem** — dois irmãos com a mesma ordem é um empate, e a
/// casa não desempata: ela não tem empates.
///
/// (Mutação: não remover o `RootOrder` ⇒ as duas raízes trazem o mesmo número.)
#[test]
fn the_copy_root_does_not_inherit_the_order() {
    let (mut sim, root, ..) = scene();
    let r = reg();
    let before = sim.world().get::<RootOrder>(root).expect("ordem").0;
    let copy = deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copia");
    assert!(
        sim.world().get::<RootOrder>(copy.root).is_none(),
        "a copia trouxe a ordem do original"
    );
    crate::assign_missing_root_order(sim.world_mut());
    let after = sim
        .world()
        .get::<RootOrder>(copy.root)
        .expect("ordem nova")
        .0;
    assert_ne!(after, before, "a copia ficou com a ordem do original");
}

/// **E a ordem INTERNA fica** — o `SiblingOrder` de uma peça é a receita, não um empate.
#[test]
fn the_pieces_keep_their_sibling_order() {
    let (mut sim, root, torso, _leg) = scene();
    let r = reg();
    let want = sim.world().get::<SiblingOrder>(torso).map(|s| s.0);
    let copy = deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copia");
    let kids: Vec<Entity> = sim
        .world()
        .get::<Children>(copy.root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    let got = sim.world().get::<SiblingOrder>(kids[0]).map(|s| s.0);
    assert_eq!(got, want, "a peca da copia perdeu a ordem dela");
}

/// **Onde a cópia aterra é do chamador** — `Some(p)` pendura, `None` deixa na raiz.
#[test]
fn the_caller_says_where_the_copy_lands() {
    let (mut sim, root, ..) = scene();
    let r = reg();
    let host = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Host")))
        .id();
    let hung = deep_copy_subtree(sim.world_mut(), &r, root, Some(host)).expect("copia");
    assert_eq!(
        sim.world().get::<ChildOf>(hung.root).map(|c| c.0),
        Some(host)
    );
    let loose = deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copia");
    assert!(sim.world().get::<ChildOf>(loose.root).is_none());
}

/// ⚠️ **O elo remapeia quando o mestre veio junto, e FICA quando ele está fora.**
///
/// O segundo caso é o normal — duplicar uma instância dá outra instância do MESMO mestre —, e é
/// o que uma busca sem cuidado estragaria (pondo `0`, ou o id da cópia da raiz).
#[test]
fn the_link_follows_the_master_only_when_the_master_was_copied() {
    let mut sim = SimWorld::new();
    let outside = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Master")))
        .id();
    let inst = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Copy"),
            InstanceOf { master: 0 },
        ))
        .id();
    crate::assign_missing_stable_ids(sim.world_mut());
    let outside_id = sim.world().get::<StableId>(outside).expect("id").0;
    sim.world_mut()
        .entity_mut(inst)
        .insert(InstanceOf { master: outside_id });

    let r = reg();
    let copy = deep_copy_subtree(sim.world_mut(), &r, inst, None).expect("copia");
    let hits = remap_instance_of(sim.world_mut(), &copy.copies(), &copy.stable_ids);
    assert_eq!(
        hits, 0,
        "remapeou um elo cujo alvo esta' FORA do que se copiou"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(copy.root).map(|i| i.master),
        Some(outside_id),
        "a copia perdeu o mestre dela"
    );

    // Agora com o mestre DENTRO: copiar os dois juntos tem de religar a cópia à cópia.
    let pair = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Pair")))
        .id();
    sim.world_mut().entity_mut(outside).insert(ChildOf(pair));
    sim.world_mut().entity_mut(inst).insert(ChildOf(pair));
    let both = deep_copy_subtree(sim.world_mut(), &r, pair, None).expect("copia");
    let hits = remap_instance_of(sim.world_mut(), &both.copies(), &both.stable_ids);
    assert_eq!(hits, 1, "o elo INTERNO ficou a apontar para fora da copia");
    let new_master = both.stable_ids[&outside_id];
    let relinked = both
        .copies()
        .into_iter()
        .filter_map(|e| sim.world().get::<InstanceOf>(e).map(|i| i.master))
        .collect::<Vec<_>>();
    assert_eq!(relinked, vec![new_master]);
}

/// **Uma raiz que não existe é um erro, não uma cópia vazia.**
#[test]
fn copying_a_dead_entity_is_an_error() {
    let (mut sim, root, ..) = scene();
    let r = reg();
    sim.world_mut().entity_mut(root).despawn();
    assert!(deep_copy_subtree(sim.world_mut(), &r, root, None).is_err());
}

/// ⭐⭐ **A PONTE para um documento POSSUÍDO não é copiada** — copiá-la daria duas entidades a
/// escrever no mesmo documento.
///
/// ⚠️ **O oráculo é o DESCRITOR**, não uma lista aqui: o teste percorre os quatro que o catálogo
/// declara `owned_document` e exige que nenhum sobreviva à cópia. Uma ponte nova entra sozinha.
///
/// (Mutação: apagar o `continue` do `owned_document` ⇒ RED nomeando o componente partilhado.)
#[test]
fn a_bridge_to_an_owned_document_is_not_copied() {
    let mut sim = SimWorld::new();
    let src = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Painted"),
            crate::PaintedDoc(7),
            crate::VecPathRef(3),
        ))
        .id();
    let r = reg();
    let copy = deep_copy_subtree(sim.world_mut(), &r, src, None).expect("copia");
    assert!(
        sim.world().get::<crate::PaintedDoc>(copy.root).is_none(),
        "a copia herdou o documento do Painter — as duas escrevem no mesmo"
    );
    assert!(
        sim.world().get::<crate::VecPathRef>(copy.root).is_none(),
        "a copia herdou o path vetorial — o `vec_entities::sync` mantem UMA entidade por path"
    );
    // Controle positivo: o resto veio.
    assert_eq!(name_of(&sim, copy.root), "Painted");
    // ⚠️ E o ORIGINAL continua ligado ao documento dele.
    assert_eq!(
        sim.world().get::<crate::PaintedDoc>(src).map(|d| d.0),
        Some(7)
    );
}

/// ⚠️ **A família das pontes É o conjunto dos documentos possuídos** — as duas listas são a mesma,
/// e sem este gate elas separavam-se em silêncio.
#[test]
fn the_bridges_are_the_owned_documents() {
    let bridges: Vec<&str> = ph2d_component_desc::catalog::bridges::DESCS
        .iter()
        .map(|d| d.canonical_name)
        .collect();
    let owned: Vec<&str> = ph2d_component_desc::all()
        .filter(|d| d.owned_document)
        .map(|d| d.canonical_name)
        .collect();
    assert_eq!(
        owned, bridges,
        "alguem declarou (ou tirou) um documento possuido fora da familia das pontes"
    );
    assert_eq!(
        owned.len(),
        4,
        "o controle positivo: o censo nao esta' vazio"
    );
}
