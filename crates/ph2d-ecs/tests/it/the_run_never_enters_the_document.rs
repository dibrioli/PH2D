//! ⭐⭐⭐ **A LEI DA WAVE DA FÁBRICA, dos dois lados** (`docs/Components/09_plano_spawner.md` §2.1):
//! *o que nasce numa corrida não é documento.*
//!
//! Ela tem **dois leitores que não se conhecem** — o `world_to_snapshot` (o que se grava e o que o
//! `Ctrl+Z` fotografa) e o `build_hierarchy_snapshot` (a lista que o artista lê) —, e por isso tem
//! **quatro** asserções: cada leitor com o seu controlo. ⛔ *Uma cerca só num dos dois é a forma que
//! o `CLAUDE.md` §5.0 chama de «dreno de um braço só».*

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, HierarchySnapshot, HierarchyWalkState, WorldSnapshot,
    build_hierarchy_snapshot, register_ecs_components, world_to_snapshot,
};
use ph2d_ecs::{
    Entity, Name, SimWorld, Spawned, Transform, TransformPropagationState, WorklistBuf,
};

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// Uma cena com **um objecto autorado** e **uma cópia nascida numa corrida**, cada uma com um filho
/// — a fixtura tem de ter subárvore, senão a poda podia estar a saltar só a raiz.
fn cena() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let autorado = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Parede")))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Tijolo"),
        ph2d_ecs::ChildOf(autorado),
    ));
    let nascido = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(3.0, 0.0)),
            Name::new("Bala"),
            Spawned {
                by: 7,
                born_tick: 1,
            },
        ))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Rasto"),
        ph2d_ecs::ChildOf(nascido),
    ));
    (sim, autorado, nascido)
}

fn nomes_no_documento(sim: &mut SimWorld) -> Vec<String> {
    let r = reg();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &r, &mut snap).expect("captura");
    let mut v: Vec<String> = snap
        .entities
        .iter()
        .filter_map(|e| {
            e.components
                .iter()
                .find(|c| {
                    r.get_by_id(c.type_id)
                        .is_some_and(|x| x.canonical_name == "ph2d::ecs::Name")
                })
                .and_then(|c| postcard::from_bytes::<Name>(&c.data).ok())
                .map(|n| n.0)
        })
        .collect();
    v.sort();
    v
}

fn nomes_na_hierarquia(sim: &mut SimWorld) -> Vec<String> {
    let mut st = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut out = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut st, &mut scratch, &mut out);
    let mut v: Vec<String> = out.entries.iter().filter_map(|e| e.name.clone()).collect();
    v.sort();
    v
}

/// ⭐⭐⭐ **O que nasceu numa corrida não entra no ficheiro nem na pilha de `Ctrl+Z`** — e o que o
/// artista desenhou entra.
///
/// (Mutação: apagar o `is_transient` do `world_to_snapshot` ⇒ a bala e o rasto aparecem.)
#[test]
fn what_was_born_in_a_run_never_enters_the_document() {
    let (mut sim, _, _) = cena();
    assert_eq!(
        nomes_no_documento(&mut sim),
        vec!["Parede".to_string(), "Tijolo".to_string()],
        "a copia de fabrica entrou no documento — cada tique dela seria um passo de undo"
    );
}

/// **O controlo do gate acima**: sem a marca de nascimento, os MESMOS objectos entram.
///
/// ⚠️ Sem este controlo, uma poda que saltasse **tudo** ficaria verde — e o produto gravaria
/// ficheiros vazios.
#[test]
fn the_same_objects_enter_the_document_when_nobody_was_born() {
    let (mut sim, _, nascido) = cena();
    sim.world_mut().entity_mut(nascido).remove::<Spawned>();
    assert_eq!(
        nomes_no_documento(&mut sim),
        vec![
            "Bala".to_string(),
            "Parede".to_string(),
            "Rasto".to_string(),
            "Tijolo".to_string()
        ],
        "a poda saltou objectos autorados"
    );
}

/// ⭐⭐ **A Hierarquia mostra o documento** — a cópia vê-se no canvas, não na lista.
///
/// (Mutação: apagar o `is_transient` do `build_hierarchy_snapshot` ⇒ RED.)
#[test]
fn the_hierarchy_shows_the_document_not_the_run() {
    let (mut sim, _, _) = cena();
    assert_eq!(
        nomes_na_hierarquia(&mut sim),
        vec!["Parede".to_string(), "Tijolo".to_string()]
    );
}

/// **O controlo do leitor da Hierarquia.**
#[test]
fn the_hierarchy_shows_everything_when_nobody_was_born() {
    let (mut sim, _, nascido) = cena();
    sim.world_mut().entity_mut(nascido).remove::<Spawned>();
    assert_eq!(nomes_na_hierarquia(&mut sim).len(), 4);
}

/// ⭐⭐⭐ **Mil cópias nascidas deixam o documento BYTE A BYTE igual** — e é isto que impede uma
/// corrida de encher a pilha de `Ctrl+Z`.
///
/// ⚠️ **O oráculo são os BYTES do snapshot, não a contagem de entidades.** O undo desta casa
/// regista por **diff** contra o baseline: se a fotografia mudar de qualquer maneira — uma linha a
/// mais, uma ordem diferente —, cada clique dado durante a corrida vira um passo. Contar entidades
/// deixaria passar uma poda que mudasse a ORDEM das que ficam.
#[test]
fn a_thousand_born_copies_leave_the_document_byte_identical() {
    let r = reg();
    let (mut sim, _, nascido) = cena();
    sim.world_mut().entity_mut(nascido).despawn();
    let bytes = |sim: &mut SimWorld| {
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::default();
        let mut snap = WorldSnapshot::default();
        world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &r, &mut snap)
            .expect("captura");
        postcard::to_allocvec(&snap).expect("bytes")
    };
    let antes = bytes(&mut sim);
    for i in 0..1_000u64 {
        let raiz = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(i as f32, 0.0)),
                Name::new(format!("bala{i}")),
                Spawned {
                    by: 7,
                    born_tick: i,
                },
            ))
            .id();
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(format!("rasto{i}")),
            ph2d_ecs::ChildOf(raiz),
        ));
    }
    assert_eq!(
        antes,
        bytes(&mut sim),
        "a corrida mexeu no documento — cada clique durante ela seria um passo de Ctrl+Z"
    );
    // O controlo: UM objecto autorado muda os bytes (senão a igualdade acima seria trivial).
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new("desenhado")));
    assert_ne!(antes, bytes(&mut sim), "a captura nao ve' nada");
}
