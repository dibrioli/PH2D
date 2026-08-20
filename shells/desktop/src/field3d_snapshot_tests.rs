//! **A metade de shell da ponte ECS do módulo de modelagem 3D** (ADR-0161).
//!
//! A `ph2d-field-ecs` prova o que o kernel sabe: que o componente serializa e que a união de uma
//! cena não depende da ordem da consulta. O que **só o shell** sabe é se o componente sobrevive à
//! máquina real de snapshot — a mesma que o Ctrl+Z e o salvar usam.
//!
//! ⚠️ **Este é o gate que separa "a crate existe" de "o app a usa".** O modo de falha do registro
//! não é um erro: é o `WorldSnapshot` **descartar o componente em silêncio**, e o sintoma aparece
//! como *o objeto sumiu ao desfazer* — três waves depois de quem esqueceu a linha em `init.rs`.
//! Foi exatamente assim que `Locked`, `GroupedChildren` e `VecPathRef` se perderam.

// ⚠️ Imports EXPLÍCITOS, e não `use super::*`. O módulo irmão a que este arquivo está pendurado é
// o smoke do traçado, cujo escopo não tem nada de ECS — herdá-lo daria uma pilha de falhas que
// parece do teste e é de onde ele foi pendurado.
use bevy_ecs::entity::Entity;
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_ecs::{FieldObject, register_field_components};

/// O registro **como o `init.rs` o monta** — se as duas listas divergirem, este gate deixa de medir
/// o que o app faz e passa a medir o que o teste faz.
fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_field_components(&mut reg);
    reg
}

fn a_doc() -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field::Node {
            xform: Xform::at(0.25, -0.5, 0.75),
            kind: ph2d_field::NodeKind::Leaf(Primitive::Box {
                half: [0.4, 0.3, 0.2],
                round: 0.05,
            }),
        }],
        NodeId(0),
    )
    .expect("documento válido")
}

/// ⭐ **A PEÇA INTEIRA atravessa o snapshot** — que é dizer: sobrevive ao desfazer e ao salvar,
/// sem uma linha de código do lado do snapshot.
///
/// ⚠️ **A afirmação mudou de tamanho na W5, e é de propósito.** Antes a peça era um componente numa
/// entidade, e o gate media a viagem de um blob. Agora ela é uma **árvore de entidades** — e o que
/// pode partir-se no caminho é a hierarquia: um filho que volta sem pai, uma ordem de irmãos
/// trocada (a subtração é `children[0]` menos os seguintes), uma pose perdida. Por isso o que se
/// compara não é o componente: é a **peça cozida** dos dois lados.
#[test]
fn the_whole_part_survives_the_world_snapshot_round_trip() {
    let reg = registry();
    let mut sim = SimWorld::new();
    // ⚠️ `TransformPropagationState::new` TOMA o mundo — é o mesmo caminho do `init.rs`. Não há
    // `::default()` de propósito: o estado é indexado pelo mundo que ele vai propagar.
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &a_doc(), "peça");
    let before = ph2d_field_ecs::cook(sim.world(), root)
        .expect("não vazia")
        .expect("válida");

    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world(), &mut prop, &mut worklist, &reg, &mut snap)
        .expect("o snapshot só falha se um componente registrado não (de)serializa");

    // Um mundo NOVO, como o undo faz: ele limpa e re-spawna do snapshot.
    let mut restored = SimWorld::new();
    snapshot_to_world(restored.world_mut(), &snap, &reg).expect("restaura");

    let mut q = restored.world_mut().query::<(Entity, &FieldObject)>();
    let roots: Vec<Entity> = q.iter(restored.world()).map(|(e, _)| e).collect();
    assert_eq!(
        roots.len(),
        1,
        "a peça não sobreviveu ao snapshot — falta o registro em `init.rs`?"
    );
    let after = ph2d_field_ecs::cook(restored.world(), roots[0])
        .expect("não vazia")
        .expect("válida");
    assert_eq!(after, before, "a peça voltou diferente do que entrou");
}

/// ⚠️ **O controle NEGATIVO, e ele é o que dá valor ao gate acima.** Sem o registro, o mesmo
/// caminho perde os componentes **sem erro nenhum** — é essa a forma exata da falha que se está a
/// prevenir. Um gate que só mostra o caso feliz não distingue *"funciona"* de *"o teste não
/// consegue falhar"*.
#[test]
fn without_the_registration_the_snapshot_drops_it_silently() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    // ⛔ de propósito: `register_field_components` NÃO é chamado aqui.

    let mut sim = SimWorld::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    ph2d_field_ecs::spawn_doc(sim.world_mut(), &a_doc(), "peça");

    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world(), &mut prop, &mut worklist, &reg, &mut snap)
        .expect("o snapshot passa — e é esse o problema");

    let mut restored = SimWorld::new();
    snapshot_to_world(restored.world_mut(), &snap, &reg).expect("restaura");

    let mut q = restored.world_mut().query::<&FieldObject>();
    assert_eq!(
        q.iter(restored.world()).count(),
        0,
        "sem registro os componentes TÊM de se perder — se sobreviveram, o gate irmão \
         está a passar por outro motivo e não prova nada"
    );
}
