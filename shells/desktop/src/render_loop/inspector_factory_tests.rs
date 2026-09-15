//! Os gates do instantâneo e das edições das secções FACTORY / LIFECYCLE (plano §5, W3).

use super::{apply_factory_edit, build_factory_info};
use ph2d_ecs::{
    DestroyOutside, Entity, Factory, Lifetime, MasterRoot, Name, Pick, SimWorld, SpawnAt, StableId,
    Transform,
};
use ph2d_editor_core::screens::hero::{FactoryFieldEdit, InspectorSpawnWhere};
use ph2d_tags::TagTree;

/// Um mundo com uma receita chamada `Mob` e uma fábrica que lhe aponta.
fn cena() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let mestre = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Mob"), MasterRoot))
        .id();
    let fab = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Spawner"),
            Factory::default(),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = sim.world().get::<StableId>(mestre).expect("id").0;
    sim.world_mut()
        .get_mut::<Factory>(fab)
        .expect("a fabrica")
        .master = id;
    (sim, mestre, fab)
}

/// ⭐⭐⭐ **A receita viaja pelo NOME nos DOIS sentidos** — o componente guarda a identidade e o
/// painel mostra e aceita um nome.
///
/// (Mutação: o snapshot a devolver o id como texto ⇒ RED; o `apply` a guardar o nome ⇒ não compila.)
#[test]
fn the_recipe_is_a_name_on_the_panel_and_an_identity_in_the_component() {
    let (mut sim, mestre, fab) = cena();
    let tree = TagTree::new();
    let info = build_factory_info(sim.world_mut(), &tree, fab.to_bits(), 1, false)
        .expect("a fabrica tem seccao");
    let f = info.factory.expect("o corpo da fabrica");
    assert_eq!(f.recipe, "Mob", "o painel tem de ver o NOME");
    assert!(f.recipe_found);

    // E o caminho de volta: escrever outro nome escreve a IDENTIDADE dele.
    let outro = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Coin"), MasterRoot))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id_outro = sim.world().get::<StableId>(outro).expect("id").0;
    apply_factory_edit(
        sim.world_mut(),
        &tree,
        fab.to_bits(),
        &FactoryFieldEdit::Recipe("Coin".into()),
    );
    assert_eq!(
        sim.world().get::<Factory>(fab).expect("fabrica").master,
        id_outro
    );
    assert_ne!(id_outro, sim.world().get::<StableId>(mestre).expect("id").0);
}

/// ⭐⭐ **Um nome que não é de um MESTRE não é uma receita** — nem na leitura nem na escrita.
///
/// ⚠️ Sem esta cerca a fábrica aceitaria apontar a um objecto comum, e a recusa só apareceria na
/// corrida, uma vez por tique, em silêncio.
#[test]
fn a_name_that_is_not_a_master_is_not_a_recipe() {
    let (mut sim, _, fab) = cena();
    let tree = TagTree::new();
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new("Wall")));
    apply_factory_edit(
        sim.world_mut(),
        &tree,
        fab.to_bits(),
        &FactoryFieldEdit::Recipe("Wall".into()),
    );
    assert_eq!(
        sim.world().get::<Factory>(fab).expect("fabrica").master,
        0,
        "um objecto comum passou por receita"
    );
    let info = build_factory_info(sim.world_mut(), &tree, fab.to_bits(), 1, false).expect("seccao");
    assert!(
        !info.factory.expect("corpo").recipe_found,
        "o painel nao avisa que a receita nao existe"
    );
}

/// ⭐⭐ **O contador de vivas é DESTA fábrica** — a fixtura tem cópias de outra.
///
/// (Mutação: somar todo o `Spawned` do mundo ⇒ RED.)
#[test]
fn the_snapshot_counts_only_this_factorys_copies() {
    let (mut sim, _, fab) = cena();
    let tree = TagTree::new();
    let meu = sim.world().get::<StableId>(fab).expect("id").0;
    for _ in 0..2 {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            ph2d_ecs::Spawned {
                by: meu,
                born_tick: 0,
            },
        ));
    }
    for _ in 0..5 {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            ph2d_ecs::Spawned {
                by: meu + 999,
                born_tick: 0,
            },
        ));
    }
    let info = build_factory_info(sim.world_mut(), &tree, fab.to_bits(), 1, false).expect("seccao");
    assert_eq!(info.factory.expect("corpo").alive, 2);
}

/// ⭐⭐ **A tag do ponto de nascimento resolve-se pela porta que compara DOBRADO.**
///
/// ⚠️ `SPAWNPOINT` e `spawnpoint` são a MESMA tag (a decisão do dono na wave #9), e uma comparação
/// crua aqui faria o campo recusar o que o painel das tags aceita.
#[test]
fn the_spawn_point_tag_is_resolved_folded() {
    let (mut sim, _, fab) = cena();
    let mut tree = TagTree::new();
    let ponto = tree.create("SpawnPoint").expect("cria");
    apply_factory_edit(
        sim.world_mut(),
        &tree,
        fab.to_bits(),
        &FactoryFieldEdit::Where(InspectorSpawnWhere::Tagged),
    );
    apply_factory_edit(
        sim.world_mut(),
        &tree,
        fab.to_bits(),
        &FactoryFieldEdit::Tag("spawnpoint".into()),
    );
    match sim.world().get::<Factory>(fab).expect("fabrica").at {
        SpawnAt::Tagged { tag, pick } => {
            assert_eq!(tag, ponto.0, "a dobra nao foi consultada");
            assert_eq!(pick, Pick::Cycle);
        }
        outro => panic!("o modo nao ficou em tag: {outro:?}"),
    }
    // E o snapshot devolve o CAMINHO, nunca o id.
    let info = build_factory_info(sim.world_mut(), &tree, fab.to_bits(), 1, false).expect("seccao");
    assert_eq!(info.factory.expect("corpo").tag, "SpawnPoint");
}

/// ⭐⭐⭐ **A metade honesta**: um objecto que nenhuma fábrica fez nascer diz-se inerte.
#[test]
fn the_panel_says_when_the_lifecycle_is_inert() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Wall"),
            Lifetime::default(),
            DestroyOutside::default(),
        ))
        .id();
    let tree = TagTree::new();
    let info = build_factory_info(sim.world_mut(), &tree, e.to_bits(), 1, false).expect("seccao");
    assert!(info.factory.is_none(), "ela nao e' uma fabrica");
    assert!(info.lifecycle.is_some());
    assert!(
        !info.is_spawned,
        "o painel diria que ela nasceu numa corrida"
    );
    assert!(
        !info.has_game_camera,
        "sem GameCamera o fora-do-ecra nao mede nada, e o painel tem de o dizer"
    );
}

/// **Sem nenhum dos três componentes não há secção** (ADR-0166).
#[test]
fn an_object_without_any_of_the_three_has_no_section() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Plain")))
        .id();
    let tree = TagTree::new();
    assert!(build_factory_info(sim.world_mut(), &tree, e.to_bits(), 1, false).is_none());
}
