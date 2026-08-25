//! **AS SETAS ATRAVESSAM O ARQUIVO** — e sem mexer no `PROJECT_SCHEMA`.
//!
//! ⚠️ **A afirmação que este gate corrige.** O plano 32 §3 dizia *"o grafo é conteúdo autorado ⇒
//! `PROJECT_SCHEMA` sobe"*. **Não sobe**, e o mecanismo é o `ComponentBlob`: ele é chaveado por
//! `blake3(nome canónico)`, não por posição. Um ficheiro gravado antes desta wave simplesmente
//! **não tem** o blob, e a entidade volta **sem máquina** — que é a leitura correcta de *"ninguém
//! desenhou seta nenhuma"*. É a mesma lição que a `line/3DModeling` registou na W35: *a peça
//! atravessa o arquivo e o `PROJECT_SCHEMA` não se mexe; a nota que dizia o contrário era velha.*
//!
//! ⇒ o que **de facto** sobe é a **contagem do registo**, nos **três** sítios (`ph2d-ecs` e os dois
//! espelhos) — e esse é um número que **soma entre linhas**.

use bevy_ecs::world::World;
use ph2d_ecs::scene::{
    ComponentRegistry, ComponentSnapshot, extract_component_snapshot, register_ecs_components,
};
use ph2d_ecs::{VecMorph, VecMorphMachine};
use ph2d_morph_machine::{MorphEdge, MorphGraph};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::default();
    register_ecs_components(&mut reg);
    reg
}

fn authored() -> VecMorphMachine {
    let mut m = VecMorphMachine::new(10);
    let mut jump = MorphEdge::new(10, 20);
    jump.when = "jump".to_string();
    jump.duration_s = 0.4;
    let mut dash = MorphEdge::new(20, 30);
    dash.when = "dash".to_string();
    dash.spring = Some(Default::default());
    m.graph = MorphGraph {
        start: 10,
        edges: vec![jump, dash],
    };
    m
}

/// **O que o artista desenhou volta inteiro** — as setas, as condições e o ritmo de cada uma.
///
/// **Mutação que deve sangrar:** tirar o `reg.register::<VecMorphMachine>` do registo — o snapshot
/// DESCARTA o componente em silêncio e o undo/save perdem o grafo.
#[test]
fn the_arrows_and_their_conditions_come_back() {
    let reg = registry();
    let mut w = World::new();
    let e = w.spawn((VecMorph::new(10, 20), authored())).id();

    let mut snap = ComponentSnapshot::new();
    extract_component_snapshot(&w, e, &reg, &mut snap).expect("extrai");
    // O CONTROLE POSITIVO: se o extractor devolver vazio, o gate abaixo passaria a comparar
    // ausência com ausência.
    assert!(
        !snap.entries.is_empty(),
        "o extractor nao devolveu blob nenhum -- este gate deixou de medir o que diz medir"
    );

    let (back, e2) = restore_world(&reg, &snap);
    assert_eq!(
        back.get::<VecMorphMachine>(e2),
        Some(&authored()),
        "as setas nao sobreviveram ao snapshot"
    );
    assert_eq!(
        back.get::<VecMorph>(e2).map(|m| m.sources),
        Some([10, 20]),
        "o CONTROLE: o irmao que ja' viajava tem de continuar a viajar"
    );
}

/// ⭐ **Um morph SEM máquina volta SEM máquina** — a ausência é uma resposta, não um buraco.
///
/// ⚠️ É isto que faz um `.ph2dproj` gravado antes desta wave abrir sem migração: o blob não existe,
/// e *"ninguém desenhou seta nenhuma"* é exactamente o que o ficheiro diz.
#[test]
fn a_morph_without_arrows_comes_back_without_them() {
    let reg = registry();
    let mut w = World::new();
    let e = w.spawn(VecMorph::new(10, 20)).id();
    let mut snap = ComponentSnapshot::new();
    extract_component_snapshot(&w, e, &reg, &mut snap).expect("extrai");

    let (back, e2) = restore_world(&reg, &snap);
    assert!(
        back.get::<VecMorph>(e2).is_some(),
        "o CONTROLE: o morph em si tem de voltar"
    );
    assert!(
        back.get::<VecMorphMachine>(e2).is_none(),
        "uma maquina nasceu do nada -- um ficheiro antigo passaria a abrir com um grafo que \
         ninguem desenhou, e o `start` dele seria uma forma que nao existe"
    );
}

/// **Repõe os blobs num mundo novo** — pela MESMA porta que o restore do undo usa
/// (`RegistryEntry::insert_from_bytes`), e não por um `insert` escrito à mão: um caminho próprio
/// aqui provaria que o `serde` funciona, não que o produto o faz.
fn restore_world(
    reg: &ComponentRegistry,
    snap: &ComponentSnapshot,
) -> (World, bevy_ecs::entity::Entity) {
    let mut back = World::new();
    let e = back.spawn(()).id();
    for entry in &snap.entries {
        let r = reg
            .iter()
            .find(|r| r.type_id == entry.type_id)
            .expect("o registo conhece o blob que ele proprio produziu");
        (r.insert_from_bytes)(&mut back, e, &entry.data).expect("repoe");
    }
    (back, e)
}
