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
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
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
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap)
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

/// ⭐ **A escultura atravessa o snapshot a carregar o NOME do arquivo** — a premissa da W23.
///
/// ⚠️ **É a única coisa que o documento guarda de uma escultura**, e é onde ela é diferente de todo
/// o resto da peça: as outras formas viajam como números, esta viaja como um `String` — o caminho do
/// arquivo. Se ele não atravessar (ou atravessar truncado, ou com os bytes trocados), regenerar
/// procura o arquivo errado, e o sintoma é o da wave inteira: a peça abre sem a escultura.
///
/// Por isso o nome do fixture tem **espaço e acento**: um caminho de verdade tem-nos, e um
/// serializador que os estrague só o mostra num deles.
#[test]
fn a_sculpture_crosses_the_snapshot_carrying_its_file_name() {
    const KEY: &str = "/tmp/uma escultura àé.obj";
    let reg = registry();
    let mut sim = SimWorld::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    let doc = FieldDoc::new(
        vec![ph2d_field::Node {
            xform: Xform::at(0.1, 0.2, 0.3),
            kind: ph2d_field::NodeKind::Sampled { key: KEY.into() },
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("documento válido");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let before = ph2d_field_ecs::cook(sim.world(), root)
        .expect("não vazia")
        .expect("válida");

    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap)
        .expect("o snapshot só falha se um componente registrado não (de)serializa");
    let mut restored = SimWorld::new();
    snapshot_to_world(restored.world_mut(), &snap, &reg).expect("restaura");

    let mut q = restored.world_mut().query::<(Entity, &FieldObject)>();
    let root = q
        .iter(restored.world())
        .map(|(e, _)| e)
        .next()
        .expect("a peça voltou");
    let after = ph2d_field_ecs::cook(restored.world(), root)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(after, before, "a escultura voltou diferente do que entrou");
    let keys: Vec<&str> = after
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            ph2d_field::NodeKind::Sampled { key } => Some(key.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        keys,
        vec![KEY],
        "o caminho do arquivo é a ÚNICA coisa que o documento guarda da escultura — sem ele, \
         regenerar procura o arquivo errado"
    );
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
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap)
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

/// Uma peça com **hierarquia e ordem**: `A ∪ (B − C)`. Ela existe porque o que se parte num
/// arquivo não é um número — é a árvore. Um filho que volta sem pai, ou dois irmãos trocados
/// (a subtração é `children[0]` menos os seguintes), dá uma peça diferente com os mesmos nós.
fn a_nested_doc() -> FieldDoc {
    let ball = |x: f32| ph2d_field::Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
        verb: None,
    };
    let combine = |op, children| ph2d_field::Node {
        xform: Xform::IDENTITY,
        kind: ph2d_field::NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.6),
            ball(0.9),
            combine(
                ph2d_field::Op::Difference(ph2d_field::Blend::Sharp),
                vec![NodeId(1), NodeId(2)],
            ),
            combine(
                ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                vec![NodeId(0), NodeId(3)],
            ),
        ],
        NodeId(4),
    )
    .expect("o aninhado")
}

/// ⭐ **A PEÇA ATRAVESSA O ARQUIVO**, e não só o snapshot em memória.
///
/// # O que este gate mede que o irmão não mede
///
/// O `the_whole_part_survives_the_world_snapshot_round_trip` prova a viagem
/// `world → WorldSnapshot → world`, **dentro do processo**. Um Ctrl+S/Ctrl+O acrescenta dois
/// passos que aquele nunca toca: o `ProjectState` inteiro passa por **postcard** (bytes em disco) e
/// volta pelo `ProjectState::restore` — que **apaga a cena antes de re-spawnar**.
///
/// ⚠️ **A limpeza é o passo que quase não apanhava esta peça, e a razão é medida:** ela consulta
/// `With<Transform>`, e os nós deste módulo **não** carregam `Transform` — a pose deles é
/// [`ph2d_field_ecs::FieldPose`], porque o `Transform` da casa é uma afim **2D**. O que salva é a
/// **raiz** levar `Transform` e o despawn cascatear por `ChildOf`. *É uma dependência entre dois
/// arquivos que nada obrigava a continuar verdadeira*, e é isso que este gate prende: um nó de campo
/// que um dia nasça sem raiz com `Transform` sobrevive à limpeza, e o load passa a **empilhar** a
/// peça velha com a nova em vez de a substituir.
#[test]
fn the_part_crosses_the_project_file_and_the_load_replaces_it_instead_of_stacking() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &a_nested_doc(), "peça");
    let before = ph2d_field_ecs::cook(sim.world(), root)
        .expect("não vazia")
        .expect("válida");

    // A captura REAL — a mesma função que o `App::capture_project` chama.
    let state = crate::undo::ProjectState::capture(
        // Nada sob condução: este gate é da peça no arquivo, não do ledger de preview.
        &crate::preview_drive::PreviewDrive::default(),
        &mut sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
    );

    // ⭐ **Os BYTES**: é aqui que um componente que não serializa, ou um `String` de caminho
    // truncado, se perde — e o `ProjectFile` não é mais do que isto com anexos.
    let bytes = postcard::to_allocvec(&state).expect("o estado serializa");
    let back: crate::undo::ProjectState = postcard::from_bytes(&bytes).expect("e desserializa");

    // ⚠️ O destino **já tem uma peça**, e é de propósito: é a cena que o artista tinha aberta
    // quando carregou em Ctrl+O. Sem isto o gate mediria um load sobre o vazio, que é o caso fácil.
    let mut target = SimWorld::new();
    ph2d_field_ecs::spawn_doc(target.world_mut(), &a_doc(), "a peça anterior");
    let _ = back.restore(&mut target, &reg);

    let mut q = target.world_mut().query::<(Entity, &FieldObject)>();
    let roots: Vec<Entity> = q.iter(target.world()).map(|(e, _)| e).collect();
    assert_eq!(
        roots.len(),
        1,
        "o load tem de SUBSTITUIR a cena, não empilhar — sobraram {} peças",
        roots.len()
    );
    let after = ph2d_field_ecs::cook(target.world(), roots[0])
        .expect("não vazia")
        .expect("válida");
    assert_eq!(
        after, before,
        "a peça voltou diferente do arquivo — hierarquia, ordem de irmãos ou pose"
    );
}
