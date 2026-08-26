//! Os gates da marca de mestre (ADR-0164 / F4.1).

use super::{MasterPiece, MasterRoot, assign_master_pieces, is_master_piece, master_root_of};
use crate::{ChildOf, Name, SimWorld, Transform};

/// Um mestre com duas peças penduradas, e um objeto solto na cena.
fn scene() -> (
    SimWorld,
    crate::Entity,
    crate::Entity,
    crate::Entity,
    crate::Entity,
) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Ragdoll"), MasterRoot))
        .id();
    let a = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Torso"), ChildOf(root)))
        .id();
    let b = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Leg"), ChildOf(a)))
        .id();
    let loose = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Crate")))
        .id();
    (sim, root, a, b, loose)
}

/// ⭐ **A marca desce a subárvore INTEIRA, e não toca em quem está fora.**
///
/// ⚠️ A neta (`b`) é o caso que interessa: uma varredura que só olhasse os filhos diretos deixaria
/// a perna do ragdoll **simulada em silêncio** dentro da biblioteca.
///
/// (Mutação: trocar a pilha por uma passagem de um nível ⇒ a neta reprova.)
#[test]
fn the_mark_reaches_the_whole_subtree_and_nothing_else() {
    let (mut sim, root, a, b, loose) = scene();
    assert!(assign_master_pieces(sim.world_mut()), "o 1.o passe MARCA");
    let w = sim.world();
    for (e, what) in [(root, "a raiz"), (a, "o filho"), (b, "a NETA")] {
        assert!(is_master_piece(w, e), "{what} do mestre ficou por marcar");
    }
    assert!(
        !is_master_piece(w, loose),
        "um objeto da cena foi marcado como peca de mestre — ele deixaria de ser simulado"
    );
}

/// ⚠️ **Idempotente** — é isso que permite chamá-la todo o quadro.
#[test]
fn running_it_again_changes_nothing() {
    let (mut sim, ..) = scene();
    assert!(assign_master_pieces(sim.world_mut()));
    assert!(
        !assign_master_pieces(sim.world_mut()),
        "o 2.o passe mexeu em alguma coisa — ele nao e' idempotente"
    );
}

/// ⭐ **A metade que falta em toda cura pela metade: ela DESMARCA.**
///
/// ⚠️ Tirar a peça do mestre e deixar a marca faz um objeto que o artista pôs na cena e que **não
/// cai** — e o defeito é mudo, porque nada na tela explica porquê.
///
/// (Mutação: apagar o laço de `have.difference(&want)` ⇒ RED.)
#[test]
fn a_piece_dragged_out_of_the_master_is_unmarked() {
    let (mut sim, _root, a, b, _loose) = scene();
    assign_master_pieces(sim.world_mut());
    assert!(is_master_piece(sim.world(), b));

    // O artista arrasta a perna para fora da biblioteca.
    sim.world_mut().entity_mut(b).remove::<ChildOf>();
    assert!(
        assign_master_pieces(sim.world_mut()),
        "o passe tem de mexer"
    );
    assert!(
        !is_master_piece(sim.world(), b),
        "a peca saiu do mestre e ficou marcada — ela nunca mais seria simulada"
    );
    assert!(is_master_piece(sim.world(), a), "o resto do mestre fica");
}

/// **E o mesmo quando a RAIZ deixa de ser mestre** (o artista desfez o componente).
#[test]
fn unmaking_the_master_unmarks_everything() {
    let (mut sim, root, a, b, _loose) = scene();
    assign_master_pieces(sim.world_mut());
    sim.world_mut().entity_mut(root).remove::<MasterRoot>();
    assign_master_pieces(sim.world_mut());
    let w = sim.world();
    for e in [root, a, b] {
        assert!(
            !is_master_piece(w, e),
            "sobrou marca de um mestre que morreu"
        );
    }
}

/// **Quem é o dono** — a pergunta do sync e da biblioteca.
#[test]
fn a_piece_names_its_master_and_a_loose_object_names_nobody() {
    let (mut sim, root, _a, b, loose) = scene();
    assign_master_pieces(sim.world_mut());
    let w = sim.world();
    assert_eq!(master_root_of(w, b), Some(root), "a neta sobe ate' a raiz");
    assert_eq!(
        master_root_of(w, root),
        Some(root),
        "a raiz e' o proprio dono"
    );
    assert_eq!(master_root_of(w, loose), None);
}

/// ⚠️ **A marca NÃO viaja no arquivo** (é derivada), e a ausência é a decisão.
///
/// Gravá-la seria estado derivado a envenenar o undo — e um mestre editado depois do gesto ficaria
/// com peças por marcar, **simuladas em silêncio**. Quem responde é o passe, todo quadro.
#[test]
fn the_mark_is_derived_and_never_registered() {
    let mut reg = crate::scene::ComponentRegistry::new();
    crate::scene::register_ecs_components(&mut reg);
    assert!(
        reg.get_by_id(crate::scene::stable_type_id("ph2d::ecs::MasterPiece"))
            .is_none(),
        "o MasterPiece foi registado — ele e' DERIVADO, e no arquivo ele envenena o undo"
    );
    assert!(
        reg.get_by_id(crate::scene::stable_type_id("ph2d::ecs::MasterRoot"))
            .is_some(),
        "o MasterRoot TEM de ser registado — ele e' autoria, e sem ele a biblioteca nao sobrevive ao save"
    );
}

/// ⚠️ **E ele sobrevive ao respawn do undo pela mesma porta**: o `MasterRoot` viaja, e o passe
/// re-deriva as peças. Sem isto, um Ctrl+Z devolveria um mestre cujas peças voltariam a ser
/// simuladas.
#[test]
fn the_master_survives_a_capture_and_restore() {
    let (mut sim, root, _a, b, _loose) = scene();
    assign_master_pieces(sim.world_mut());
    let mut reg = crate::scene::ComponentRegistry::new();
    crate::scene::register_ecs_components(&mut reg);
    let mut prop = crate::TransformPropagationState::new(sim.world_mut());
    let mut work = crate::WorklistBuf::default();
    let mut snap = crate::scene::WorldSnapshot::default();
    crate::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg, &mut snap)
        .expect("captura");

    let mut fresh = SimWorld::new();
    crate::scene::snapshot_to_world(fresh.world_mut(), &snap, &reg).expect("restore");
    assert!(
        assign_master_pieces(fresh.world_mut()),
        "depois do restore o passe tem de re-derivar as pecas"
    );
    let names: Vec<String> = {
        let mut q = fresh
            .world_mut()
            .query_filtered::<&Name, bevy_ecs::query::With<MasterPiece>>();
        let mut v: Vec<String> = q.iter(fresh.world()).map(|n| n.0.clone()).collect();
        v.sort();
        v
    };
    assert_eq!(
        names,
        vec![
            "Leg".to_string(),
            "Ragdoll".to_string(),
            "Torso".to_string()
        ],
        "o mestre nao voltou inteiro do restore"
    );
    let _ = (root, b);
}
