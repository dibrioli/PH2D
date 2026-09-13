//! Os gates da PERTENÇA a tags — `docs/Components/08_plano_tags.md` §5.2 (W1: 6, 8, 9, 11 e as
//! leis da porta), escritos contra um *stub* (`todo!()`) e vistos vermelhos.
//!
//! ⚠️ **A fixture é a do plano §5.1** — `Enemy` › `Flying` › `Boss`, as raízes irmãs `Statue` e
//! `Player`, e sete objectos —, e cada gate que depende de uma PERTURBAÇÃO confere primeiro que ela
//! de facto perturba (senão o gate passaria por vácuo).

use super::{Tags, belongs, remap, scrub, tagged};
use crate::{Entity, Name, StableId, World};
use ph2d_tags::{Tag, TagId, TagTree};
use std::collections::{BTreeMap, BTreeSet};

struct Fixture {
    world: World,
    tree: TagTree,
    enemy: TagId,
    flying: TagId,
    boss: TagId,
    statue: TagId,
    player: TagId,
    e: BTreeMap<&'static str, Entity>,
}

fn fixture() -> Fixture {
    let mut tree = TagTree::new();
    let boss = tree.create("Enemy/Flying/Boss").expect("cria");
    let enemy = tree.find("Enemy").expect("ancestral");
    let flying = tree.find("Enemy/Flying").expect("ancestral");
    let statue = tree.create("Statue").expect("cria");
    let player = tree.create("Player").expect("cria");
    let mut world = World::new();
    let mut e = BTreeMap::new();
    for (i, (nome, tag)) in [
        ("Goblin A", enemy),
        ("Goblin B", enemy),
        ("Bat A", flying),
        ("Bat B", flying),
        ("Dragon", boss),
        ("Statue", statue),
        ("Hero", player),
    ]
    .into_iter()
    .enumerate()
    {
        // ⚠️ **Com `Transform`, como todo objecto da Hierarquia**: a captura do undo percorre as
        // RAÍZES de pose, e um objecto sem ela nem sequer entra no snapshot (a 1.ª versão desta
        // fixture não o tinha, e o restore devolveu um mundo vazio).
        let id = world
            .spawn((
                crate::Transform::IDENTITY,
                Name::new(nome),
                StableId(i as u64 + 1),
                Tags::from_ids([tag]),
            ))
            .id();
        e.insert(nome, id);
    }
    Fixture {
        world,
        tree,
        enemy,
        flying,
        boss,
        statue,
        player,
        e,
    }
}

fn nomes(world: &World, es: &[Entity]) -> Vec<String> {
    es.iter()
        .map(|&x| world.get::<Name>(x).expect("nome").as_str().to_string())
        .collect()
}

fn directas(world: &World, x: Entity) -> BTreeSet<TagId> {
    world
        .get::<Tags>(x)
        .expect("tem Tags")
        .direct_ids()
        .collect()
}

fn bytes_por_identidade(world: &mut World) -> Vec<(u64, Vec<u8>)> {
    let mut v: Vec<(u64, Vec<u8>)> = world
        .query::<(&StableId, &Tags)>()
        .iter(world)
        .map(|(s, t)| (s.0, postcard::to_allocvec(t).expect("serializa")))
        .collect();
    v.sort();
    v
}

/// ⭐⭐⭐ **Pertencer é pertencer à SUBÁRVORE, e nunca à raiz irmã.**
///
/// **Mutação que deve sangrar:** `belongs` a comparar o conjunto directo com `q` (o `Bat` deixa de
/// pertencer a `Enemy`).
#[test]
fn belonging_reaches_the_subtree_and_never_the_sibling_root() {
    let f = fixture();
    let bat = f.world.get::<Tags>(f.e["Bat A"]).expect("tem");
    assert!(belongs(bat, &f.tree, f.enemy), "Flying e' filha de Enemy");
    assert!(belongs(bat, &f.tree, f.flying));
    assert!(!belongs(bat, &f.tree, f.boss), "o pai nao pertence a filha");
    assert!(!belongs(bat, &f.tree, f.statue), "a raiz irma");
    let hero = f.world.get::<Tags>(f.e["Hero"]).expect("tem");
    assert!(belongs(hero, &f.tree, f.player));
    assert!(!belongs(hero, &f.tree, f.enemy));
}

/// ⭐⭐⭐ **Renomear ou mover uma tag nunca toca num objecto** (gate 6) — o Blender medido: a
/// coleção é referência.
///
/// **Mutações que devem sangrar:** a pertença guardada por CAMINHO (os bytes mudariam) · o `tagged`
/// sem a subárvore (o `Enemy` perdia os `Bat`).
#[test]
fn renaming_or_moving_a_tag_never_touches_a_member() {
    let mut f = fixture();
    let antes = bytes_por_identidade(&mut f.world);
    f.tree.rename(f.enemy, "Monster").expect("renomeia");
    f.tree.move_under(f.boss, None).expect("move");
    assert_eq!(
        bytes_por_identidade(&mut f.world),
        antes,
        "nenhum objecto mudou"
    );
    assert_eq!(
        nomes(&f.world, &tagged(&f.world, &f.tree, f.enemy)),
        ["Goblin A", "Goblin B", "Bat A", "Bat B"],
        "o Dragon saiu com o Boss; o resto ficou"
    );
    assert_eq!(
        nomes(&f.world, &tagged(&f.world, &f.tree, f.boss)),
        ["Dragon"]
    );
}

/// ⭐⭐⭐ **Apagar leva a subárvore E a pertença, no mesmo gesto** (gate 8).
///
/// **Mutações que devem sangrar:** o `scrub` a tirar só a raiz apagada (os `Bat` ficam com um id
/// órfão) · o `scrub` a apagar o componente (a secção sumia do Inspector).
#[test]
fn deleting_a_tag_takes_its_subtree_and_the_membership_in_one_gesture() {
    let mut f = fixture();
    let saem = f.tree.delete(f.enemy);
    assert_eq!(saem, BTreeSet::from([f.enemy, f.flying, f.boss]));
    assert_eq!(
        scrub(&mut f.world, &saem),
        5,
        "Goblin A/B, Bat A/B e Dragon"
    );

    let vivas: BTreeSet<TagId> = f.tree.tags().map(|t| t.id).collect();
    for (nome, &x) in &f.e {
        let orfas: Vec<TagId> = directas(&f.world, x).difference(&vivas).copied().collect();
        assert!(orfas.is_empty(), "{nome} ficou com ids orfaos {orfas:?}");
    }
    assert!(directas(&f.world, f.e["Goblin A"]).is_empty());
    assert!(
        f.world.get::<Tags>(f.e["Goblin A"]).is_some(),
        "o componente fica: a lista vazia continua a ser do objecto"
    );
    assert_eq!(
        directas(&f.world, f.e["Statue"]),
        BTreeSet::from([f.statue])
    );
    assert_eq!(
        nomes(&f.world, &tagged(&f.world, &f.tree, f.player)),
        ["Hero"]
    );
    assert_eq!(
        scrub(&mut f.world, &saem),
        0,
        "a segunda passagem nao tem o que tirar"
    );
}

/// ⚠️ **O `scrub` só escreve em quem muda** — o undo regista por DIFF e a captura incremental lê o
/// relógio de mudanças; carimbar quem não mudou é trabalho por nada.
///
/// **Mutação que deve sangrar:** o `scrub` a pedir `get_mut` a todo `Tags` e a filtrar depois.
#[test]
fn scrubbing_only_stamps_the_objects_it_changes() {
    use bevy_ecs::change_detection::DetectChanges;
    let mut f = fixture();
    f.world.clear_trackers();
    let saem = f.tree.delete(f.boss);
    assert_eq!(scrub(&mut f.world, &saem), 1);
    let mudou = |nome: &str| {
        f.world
            .entity(f.e[nome])
            .get_ref::<Tags>()
            .expect("tem")
            .is_changed()
    };
    assert!(mudou("Dragon"));
    assert!(
        !mudou("Statue"),
        "quem nao perdeu nada nao pode ter sido carimbado"
    );
    assert!(!mudou("Bat A"));
}

/// ⭐⭐⭐ **Um documento com GÉMEOS funde-os e nenhum objecto perde a pertença** (gate 9).
///
/// ⚠️ **A fixture reproduz o fenómeno MEDIDO no ficheiro de catálogos que o Blender 5.2.1 distribui**
/// (2 caminhos de 63 aparecem duas vezes, com UUIDs diferentes — um deles é
/// `Geometry Nodes/Generate`) sem copiar esse ficheiro para cá: ele é um dado distribuído com o
/// programa, não SAÍDA de uma entrada nossa (CLAUDE.md §0.9). E junta o gémeo DOBRADO (`Enemy` /
/// `énemy`), que é o que a decisão D2 acrescenta.
///
/// **Mutação que deve sangrar:** não aplicar o `remap` (o objecto do gémeo descartado fica com um id
/// órfão e sai do `tagged`).
#[test]
fn a_document_with_twins_merges_them_and_keeps_every_member() {
    let tag = |id: u64, path: &str| Tag {
        id: TagId(id),
        path: path.into(),
    };
    let lido = vec![
        tag(7, "Geometry Nodes/Generate"),
        tag(3, "Geometry Nodes/Generate"),
        tag(1, "Geometry Nodes"),
        tag(4, "Enemy"),
        tag(9, "énemy"),
    ];
    let mut world = World::new();
    let mut spawn = |nome: &'static str, s: u64, ids: &[u64]| {
        world
            .spawn((
                Name::new(nome),
                StableId(s),
                Tags::from_ids(ids.iter().map(|&i| TagId(i))),
            ))
            .id()
    };
    let a = spawn("A", 1, &[7]);
    let b = spawn("B", 2, &[3]);
    let c = spawn("C", 3, &[9, 4]);
    let d = spawn("D", 4, &[1]);

    let (tree, mapa) = TagTree::restore(lido, 10);
    assert_eq!(remap(&mut world, &mapa), 2, "A e C mudaram; B e D nao");

    assert_eq!(directas(&world, a), BTreeSet::from([TagId(3)]));
    assert_eq!(
        directas(&world, c),
        BTreeSet::from([TagId(4)]),
        "os dois gemeos no mesmo objecto dao UMA tag"
    );
    assert_eq!(directas(&world, b), BTreeSet::from([TagId(3)]));
    assert_eq!(directas(&world, d), BTreeSet::from([TagId(1)]));
    assert_eq!(nomes(&world, &tagged(&world, &tree, TagId(3))), ["A", "B"]);
    assert_eq!(
        nomes(&world, &tagged(&world, &tree, TagId(1))),
        ["A", "B", "D"],
        "a hierarquia continua a valer depois da fusao"
    );
    assert_eq!(nomes(&world, &tagged(&world, &tree, TagId(4))), ["C"]);
}

/// ⭐⭐⭐ **A ordem é a da IDENTIDADE, nunca a do arquétipo** (gate 11) — sob as DUAS perturbações do
/// plano §5.1: um componente alheio inserido no Goblin A, e um restore do snapshot (bits novos).
///
/// **Mutação que deve sangrar:** tirar a ordenação por `StableId` do `tagged`.
#[test]
fn the_query_order_is_the_identity_not_the_archetype() {
    let mut f = fixture();
    f.world.entity_mut(f.e["Goblin A"]).insert(crate::Locked);

    // ⚠️ O controlo: a perturbação tem de de facto perturbar a ordem da query.
    let crua: Vec<Entity> = f
        .world
        .query::<(Entity, &Tags)>()
        .iter(&f.world)
        .map(|(x, _)| x)
        .collect();
    assert_ne!(
        nomes(&f.world, &crua).first().map(String::as_str),
        Some("Goblin A"),
        "o componente alheio nao mudou a ordem do arquetipo -- a fixture nao tem o fenomeno"
    );

    let esperado = ["Goblin A", "Goblin B", "Bat A", "Bat B", "Dragon"];
    assert_eq!(
        nomes(&f.world, &tagged(&f.world, &f.tree, f.enemy)),
        esperado
    );

    let mut reg = crate::scene::ComponentRegistry::new();
    crate::scene::register_ecs_components(&mut reg);
    let mut prop = crate::TransformPropagationState::new(&mut f.world);
    let mut work = crate::WorklistBuf::default();
    let mut snap = crate::scene::WorldSnapshot::default();
    crate::scene::world_to_snapshot(&mut f.world, &mut prop, &mut work, &reg, &mut snap)
        .expect("captura");
    // ⚠️ Piso: uma captura vazia daria um restore vazio, e a asserção de ordem leria `[]`.
    assert_eq!(
        snap.entities.len(),
        7,
        "a captura tem de levar os sete objectos"
    );
    let mut fresco = World::new();
    crate::scene::snapshot_to_world(&mut fresco, &snap, &reg).expect("restore");
    assert_eq!(
        nomes(&fresco, &tagged(&fresco, &f.tree, f.enemy)),
        esperado,
        "o undo respawna com bits novos e a ordem nao pode mudar"
    );
}

/// ⚠️ **Os bytes não dependem da ordem em que as tags foram postas** (gate 11) — o undo regista por
/// DIFF de bytes.
///
/// **Mutação que deve sangrar:** guardar um `Vec` em vez de um conjunto ordenado.
#[test]
fn tags_bytes_do_not_depend_on_insertion_order() {
    let a = Tags::from_ids([TagId(3), TagId(1), TagId(2)]);
    let mut b = Tags::default();
    for i in [1, 2, 3] {
        assert!(b.insert(TagId(i)));
    }
    assert!(!b.insert(TagId(2)), "repetir nao acrescenta");
    assert!(!b.insert(TagId(0)), "o reservado nunca entra");
    assert_eq!(
        postcard::to_allocvec(&a).expect("serializa"),
        postcard::to_allocvec(&b).expect("serializa")
    );
    assert_eq!((a.len(), b.len()), (3, 3));
    assert!(b.remove(TagId(2)) && !b.remove(TagId(2)));
    assert_eq!(b.direct_ids().collect::<Vec<_>>(), [TagId(1), TagId(3)]);
}

/// ⚠️ **O objecto não aceita mais tags do que o painel mostra** — a lei que o `ANIM_TAGS_MAX` já
/// pagou: *um modelo que aceita o que o painel não mostra produz estado inalcançável*.
///
/// ⚠️ O número sai da SECÇÃO: ela desenha um chip por tag dentro da largura do Inspector, e
/// [`super::TAGS_MAX`] é o que cabe sem a secção sozinha passar a altura útil da coluna — o mesmo
/// argumento do `TIMERS_MAX`.
///
/// **Mutação que deve sangrar:** o `insert` sem o tecto (o 17.º chip nascia inalcançável).
#[test]
fn an_object_takes_no_more_tags_than_the_panel_shows() {
    let mut t = Tags::default();
    for i in 1..=super::TAGS_MAX {
        assert!(t.insert(TagId(i as u64)), "a {i}.ª tag foi recusada");
    }
    assert_eq!(t.len(), super::TAGS_MAX);
    assert!(
        !t.insert(TagId(999)),
        "o objecto aceitou mais tags do que a seccao desenha"
    );
    assert_eq!(t.len(), super::TAGS_MAX, "a recusa acrescentou na mesma");
    // ⚠️ E uma que JÁ lá está continua a ser recusada por ser repetida, não por falta de espaço.
    assert!(!t.insert(TagId(1)));
    // Tirar uma abre espaço para outra — o tecto é do CONJUNTO, não um contador à parte.
    assert!(t.remove(TagId(1)) && t.insert(TagId(999)));
}

/// ⛔ **Uma tag que já não existe não alcança ninguém** — nem como pergunta, nem como id órfão num
/// objecto (a lei do alvo que não existe: falha FECHADA).
///
/// **Mutação que deve sangrar:** o `belongs` a aceitar `q` no conjunto directo sem a árvore.
#[test]
fn a_tag_that_no_longer_exists_reaches_nobody() {
    let mut f = fixture();
    let _ = f.tree.delete(f.boss); // ⚠️ SEM o `scrub`: o Dragon fica com o id órfão de propósito.
    let dragon = f.world.get::<Tags>(f.e["Dragon"]).expect("tem");
    assert!(
        !belongs(dragon, &f.tree, f.boss),
        "a pergunta por uma tag apagada"
    );
    assert!(
        !belongs(dragon, &f.tree, f.enemy),
        "um id orfao nao e' descendente de ninguem"
    );
    assert!(tagged(&f.world, &f.tree, f.boss).is_empty());
    assert_eq!(tagged(&f.world, &f.tree, f.enemy).len(), 4);
}
