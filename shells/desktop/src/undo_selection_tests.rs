//! Gates da **seleção que sobrevive ao undo** — a política que o `apply_project` aplica.
//!
//! O `apply_project` exige `gfx` (janela + GPU) e não é alcançável headless; por isso a política
//! mora numa função pura. Estes gates são sobre ela, e o irmão de ARQUITETURA
//! (`tests/the_undo_preserves_the_vector_selection.rs`) prova que o `apply_project` a CHAMA — sem
//! ele, alguém pode reintroduzir o `vec_pen.clear()` sozinho e estes ficam todos verdes.

use super::{field_selection_back, field_selection_ids, surviving_selection};
use ph2d_vec_scene::{VecScene, rectangle};

/// **O QUE AINDA EXISTE CONTINUA SELECIONADO.** É o que devolve o overlay do envelope depois do
/// Ctrl+Z: a gaiola/os pinos são desenhados a partir da seleção, e sem ela a ferramenta fica
/// funcionando e **invisível** (foi assim que o Enio o descreveu).
#[test]
fn a_shape_that_survives_the_restore_stays_selected() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    assert_eq!(surviving_selection(&[a, b], &scene), vec![a, b]);
}

/// **O QUE O UNDO FEZ DESAPARECER SAI DA SELEÇÃO.** Desfazer a criação de uma forma tem de a tirar
/// da seleção — mantê-la deixaria um id pendurado que o mapa nunca resolve, e o gizmo apontaria
/// para nada.
#[test]
fn a_shape_the_restore_removed_is_dropped() {
    let mut before = VecScene::new();
    let a = before.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = before.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));

    // O estado restaurado é ANTERIOR à criação de `b`.
    let mut restored = VecScene::new();
    let a2 = restored.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    assert_eq!(
        a, a2,
        "os ids da cena são determinísticos — premissa do fixture"
    );

    assert_eq!(
        surviving_selection(&[a, b], &restored),
        vec![a],
        "a forma que o undo desfez continuou selecionada"
    );
}

/// **Seleção vazia continua vazia** — e uma cena vazia não ressuscita nada.
#[test]
fn nothing_selected_stays_nothing() {
    assert!(surviving_selection(&[], &VecScene::new()).is_empty());
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    assert!(surviving_selection(&[], &scene).is_empty());
    assert!(surviving_selection(&[a], &VecScene::new()).is_empty());
}

/// ⭐⭐⭐ **A SELEÇÃO DE UMA PEÇA 3D SOBREVIVE AO RESPAWN** (W113) — a irmã da de cima, e o report
/// que a comprou é *«o undo/redo não obedece cada etapa, principalmente se transformação»*
/// (Enio, 2026-09-03).
///
/// ⚠️ O undo **respawna** o mundo: os `Entity::to_bits()` são todos novos. Este gate encena
/// exactamente isso — dois mundos, os mesmos `StableId`, entidades diferentes — e exige que os bits
/// que voltam sejam os do mundo NOVO.
#[test]
fn a_three_d_node_that_survives_the_respawn_keeps_its_selection() {
    use ph2d_ecs::StableId;
    let leaf = || ph2d_field_ecs::FieldNode {
        shape: ph2d_field::NodeShape::Leaf(ph2d_field::Primitive::Sphere { radius: 1.0 }),
    };
    // O mundo ANTES: dois nós do modelador, e um objecto que não é do módulo.
    let mut antes = bevy_ecs::world::World::new();
    let a = antes.spawn((leaf(), StableId(11))).id();
    let b = antes.spawn((leaf(), StableId(22))).id();
    let alheio = antes.spawn(StableId(33)).id();
    let ids = field_selection_ids(&antes, &[a.to_bits(), b.to_bits(), alheio.to_bits()]);
    assert_eq!(
        ids,
        vec![StableId(11), StableId(22)],
        "só os nós do MODELADOR entram — alargar isto mudaria módulos que não o pediram"
    );

    // O mundo DEPOIS do respawn: os mesmos ids, entidades diferentes, e o `b` não sobreviveu.
    let mut depois = bevy_ecs::world::World::new();
    // ⚠️ Queima algumas entidades para garantir que os bits NÃO coincidem com os de antes.
    for _ in 0..5 {
        depois.spawn_empty();
    }
    let a2 = depois.spawn((leaf(), StableId(11))).id();
    assert_ne!(
        a.to_bits(),
        a2.to_bits(),
        "o controle: o respawn mudou os bits"
    );
    assert_eq!(
        field_selection_back(&mut depois, &ids),
        vec![a2.to_bits()],
        "quem sobreviveu volta com os bits NOVOS, e quem morreu simplesmente não volta"
    );
}

/// ⛔ **E um `StableId` VAZIO não conta** — guardá-lo traria de volta o primeiro nó sem identidade
/// que o mundo tivesse, que é pior do que nenhuma seleção.
#[test]
fn a_node_without_a_stable_id_is_not_carried_across() {
    use ph2d_ecs::StableId;
    let mut w = bevy_ecs::world::World::new();
    let sem = w
        .spawn((
            ph2d_field_ecs::FieldNode {
                shape: ph2d_field::NodeShape::Leaf(ph2d_field::Primitive::Sphere { radius: 1.0 }),
            },
            StableId::NONE,
        ))
        .id();
    assert!(field_selection_ids(&w, &[sem.to_bits()]).is_empty());
}
