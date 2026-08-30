//! ⭐ **DESTACAR e APLICAR AO MESTRE** — irmão de `instance_verbs_tests` por ASSUNTO (e pelo tecto
//! de 600 LOC do shell). Lá ficam os gates do *Make*; aqui os dos dois verbos que mexem no VÍNCULO
//! já existente.
//!
//! ⚠️ **Este corte foi imposto por um gate que esteve VERMELHO sem ninguém ver**
//! (`shell_files_respect_hr18_loc_cap`): ele vive em `shells/desktop/tests/` e o portão de fecho
//! desta linha corria `cargo test --bins`, que **não toca** naquele diretório.
//!
//! ⚠️ Os ajudantes são re-declarados aqui, e é o precedente do `instance_place_tests.rs`: eles são
//! embrulhos de cinco linhas, e partilhá-los obrigaria a inventar um terceiro ficheiro só para
//! eles.

use super::{VerbRefusal, detach};
use crate::instance_smoke::{spawn_master, spawn_ragdoll_scene};
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{Entity, InstanceOf, ObjectInstance, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// O *Apply to Master*, com o par de documentos vazio.
fn apply(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    clicked: Entity,
) -> Result<usize, VerbRefusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::apply_to_master(
        sim,
        r,
        echo,
        clicked,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// Um passe de sync, com o par de documentos vazio.
fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        bridge,
        echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

// ── DESTACAR ───────────────────────────────────────────────────────────────────────────────

/// ⭐ **Destacar corta o vínculo e não muda mais nada** — os objetos continuam iguais, só deixam
/// de seguir a receita.
///
/// (Mutação: não remover o `InstanceOf` das PEÇAS ⇒ o sync continua a alcançá-las e o gate
/// reprova quando a receita muda.)
#[test]
fn detaching_stops_the_following_and_changes_nothing_else() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    let before = tint(&sim, mine);
    // O gesto, feito a partir de uma PEÇA — e ele solta a instância INTEIRA.
    assert_eq!(detach(&mut sim, mine), Ok(4));
    assert!(sim.world().get::<InstanceOf>(roots[0]).is_none());
    assert!(sim.world().get::<InstanceOf>(mine).is_none());
    assert_eq!(tint(&sim, mine), before, "destacar mudou o que se ve'");

    // A receita muda; a solta não ouve mais, e as outras duas ouvem.
    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, master_arm, [0.9, 0.9, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, mine),
        before,
        "a solta continuou a seguir a receita"
    );
    assert_eq!(
        tint(&sim, piece(&sim, roots[1], "Arm")),
        [0.9, 0.9, 0.1, 1.0],
        "as outras deixaram de seguir — destacar UMA soltou todas"
    );
}

/// ⛔ **Destacar o que não é instância é recusado** (a receita não é cópia de ninguém).
#[test]
fn detaching_something_that_is_not_an_instance_is_refused() {
    let mut sim = SimWorld::new();
    let master = spawn_master(&mut sim);
    assert_eq!(detach(&mut sim, master), Err(VerbRefusal::NotAnInstance));
}

// ── APLICAR AO MESTRE ──────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **APLICAR promove a excepção a padrão** — o valor entra na receita, e as OUTRAS cópias
/// recebem-no.
///
/// ⚠️ **É a régua inteira do verbo**: se ele só apagasse a excepção, a cópia voltaria à cor antiga
/// (isso é o *Redefinir*); se só escrevesse na receita sem limpar a chave, a cópia continuaria
/// surda e a diferença voltaria no gesto seguinte.
///
/// (Mutação: trocar o `insert_from_bytes` no mestre por um no-op ⇒ as outras não recebem nada.)
#[test]
fn applying_an_override_makes_it_the_recipe_for_everyone() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a fixtura tem de conter a excepcao, senao o gate nao mede nada"
    );

    assert_eq!(apply(&mut sim, &r, &mut echo, mine), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Arm")),
        [0.1, 0.2, 0.9, 1.0],
        "o valor nao chegou a' RECEITA"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        0,
        "a excepcao sobreviveu ao Apply — a copia fica surda ao proprio valor que promoveu"
    );

    pass(&mut sim, &r, &bridge, &mut echo);
    for (i, &root) in roots.iter().enumerate() {
        assert_eq!(
            tint(&sim, piece(&sim, root, "Arm")),
            [0.1, 0.2, 0.9, 1.0],
            "a instancia {} nao recebeu o valor promovido",
            i + 1
        );
    }
}

/// ⚠️ **O ESCOPO é o que se clicou** — numa peça, só a excepção dela.
#[test]
fn applying_from_a_piece_promotes_only_that_piece() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let arm = piece(&sim, roots[0], "Arm");
    let hub = piece(&sim, roots[0], "Hub");
    let hub_before = tint(&sim, piece(&sim, master, "Hub"));
    paint(&mut sim, arm, [0.1, 0.2, 0.9, 1.0]);
    paint(&mut sim, hub, [0.9, 0.1, 0.9, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(apply(&mut sim, &r, &mut echo, arm), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Hub")),
        hub_before,
        "aplicar o BRACO promoveu tambem o eixo — o escopo esta' errado"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a excepcao do eixo tinha de ficar"
    );
}

/// **Sem excepção nenhuma o verbo responde ZERO** — e não um erro: o artista clicou no sítio certo.
#[test]
fn applying_with_nothing_overridden_answers_zero() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (_master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(apply(&mut sim, &r, &mut echo, roots[0]), Ok(0));
}
