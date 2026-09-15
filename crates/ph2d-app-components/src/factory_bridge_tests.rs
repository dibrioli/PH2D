//! Os gates da ponte (`docs/Components/09_plano_spawner.md` §5, W2).

use super::*;
use crate::instance_smoke::spawn_master;
use ph2d_ecs::{DeathCause, Factory, Name, SimWorld, tick_factories};
use ph2d_tags::TagTree;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    r
}

fn docs() -> (
    ph2d_vec_scene::VecScene,
    ph2d_vec_entities::entities::VecEntityMap,
) {
    crate::instance_docs::empty_docs()
}

/// Uma cena com um mestre e uma fábrica ligada a ele.
fn cena(f: Factory) -> (SimWorld, ComponentRegistry, Entity, Entity) {
    let r = reg();
    let mut sim = SimWorld::new();
    let mestre = spawn_master(&mut sim);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = sim.world().get::<StableId>(mestre).expect("id").0;
    let fab = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(5.0, 2.0)),
            Name::new("Fabrica"),
            Factory { master: id, ..f },
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, r, mestre, fab)
}

/// ⭐⭐⭐ **O caminho inteiro: um sinal faz nascer, e a cópia fica onde a fábrica mandou.**
#[test]
fn a_signal_makes_copies_and_they_land_where_the_factory_said() {
    let (mut sim, r, _, _) = cena(Factory {
        on_signal: "vai".into(),
        burst: 3,
        ..Default::default()
    });
    let tree = TagTree::new();
    let tick = tick_factories(sim.world_mut(), &tree, &["vai"]);
    assert_eq!(tick.births.len(), 3);
    let (mut sc, mut mp) = docs();
    let rel = apply_births(
        &mut sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        &tick.births,
        7,
    );
    assert_eq!(rel.nasceram, 3);
    assert_eq!(rel.recusadas, 0);
    // As três estão na pose da fábrica, marcadas, e com o tique do nascimento.
    let mut q = sim.world_mut().query::<(&Transform, &Spawned)>();
    let mut vistos = 0;
    let poses: Vec<(f32, f32, u64)> = q
        .iter(sim.world())
        .map(|(t, s)| (t.translation.x, t.translation.y, s.born_tick))
        .collect();
    for (x, y, born) in poses {
        assert!(
            (x - 5.0).abs() < 1e-5 && (y - 2.0).abs() < 1e-5,
            "pose errada"
        );
        assert_eq!(born, 7);
        vistos += 1;
    }
    assert_eq!(vistos, 3, "as copias nao ficaram marcadas");
}

/// ⭐⭐ **Uma receita que já não existe é RECUSA CONTADA, nunca um pânico nem um silêncio.**
#[test]
fn a_recipe_that_no_longer_exists_is_a_counted_refusal() {
    let (mut sim, r, mestre, _) = cena(Factory {
        on_signal: "vai".into(),
        burst: 2,
        ..Default::default()
    });
    let tree = TagTree::new();
    let tick = tick_factories(sim.world_mut(), &tree, &["vai"]);
    assert_eq!(tick.births.len(), 2);
    // O mestre desaparece ENTRE a lei e a ponte — que é o que um Ctrl+Z faz.
    sim.world_mut().entity_mut(mestre).despawn();
    let (mut sc, mut mp) = docs();
    let rel = apply_births(
        &mut sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        &tick.births,
        1,
    );
    assert_eq!(rel.nasceram, 0);
    assert_eq!(rel.recusadas, 2);
}

/// ⭐⭐ **O dreno da morte DEDUPLICA** — uma bala que morre de velha E por sair do ecrã aparece nas
/// duas listas, e `despawn` duas vezes entra em pânico.
#[test]
fn the_death_drain_deduplicates() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Bala")))
        .id();
    let mortes = vec![
        ph2d_ecs::Death {
            entity: e,
            signal: "a".into(),
            why: DeathCause::Aged,
        },
        ph2d_ecs::Death {
            entity: e,
            signal: String::new(),
            why: DeathCause::Outside,
        },
    ];
    assert_eq!(apply_deaths(&mut sim, &mortes), 1, "matou duas vezes");
    assert!(sim.world().get_entity(e).is_err());
}

/// ⭐⭐ **Morrer leva a SUBÁRVORE** — senão ficam órfãos sem `ChildOf`, que a captura volta a ver
/// como raízes do documento.
#[test]
fn a_death_takes_the_whole_subtree() {
    let mut sim = SimWorld::new();
    let raiz = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Mob")))
        .id();
    let filho = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Arma"),
            ph2d_ecs::ChildOf(raiz),
        ))
        .id();
    apply_deaths(
        &mut sim,
        &[ph2d_ecs::Death {
            entity: raiz,
            signal: String::new(),
            why: DeathCause::Aged,
        }],
    );
    assert!(
        sim.world().get_entity(filho).is_err(),
        "o filho ficou orfao"
    );
}

/// ⭐⭐ **Rebobinar varre o que nasceu, e NÃO toca no que o artista desenhou.**
#[test]
fn the_sweep_takes_the_born_and_leaves_the_authored() {
    let mut sim = SimWorld::new();
    let autorado = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Parede")))
        .id();
    let nascido = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Bala"),
            Spawned {
                by: 1,
                born_tick: 0,
            },
        ))
        .id();
    assert_eq!(sweep_spawned(&mut sim), 1);
    assert!(sim.world().get_entity(nascido).is_err());
    assert!(sim.world().get_entity(autorado).is_ok(), "varreu documento");
}
