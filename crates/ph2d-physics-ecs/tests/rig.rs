//! **`rig_edges` / `subtree_parts` — a árvore que o artista desenhou É o rig.**
//!
//! Uma corrente liga uma FILA; um ragdoll é uma ÁRVORE (a pelve tem três
//! filhos). Estes gates pinam a topologia que o gerador do shell consome —
//! puros, headless, sobre o ECS autorado, sem um dispatch no caminho (o mesmo
//! corte do `joint_group.rs` ao lado).

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{rig_edges, subtree_parts};
use std::collections::BTreeSet;

fn node(sim: &mut SimWorld, name: &str, parent: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new(name),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    if let Some(p) = parent {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

/// As arestas em vocabulário de NOME, para as asserções lerem como a fixture.
fn edge_names(sim: &mut SimWorld, parts: &[Entity]) -> BTreeSet<String> {
    let edges = rig_edges(sim.world(), parts);
    edges
        .into_iter()
        .map(|(a, b)| {
            let na = sim
                .world()
                .get::<Name>(a)
                .expect("name")
                .as_str()
                .to_string();
            let nb = sim
                .world()
                .get::<Name>(b)
                .expect("name")
                .as_str()
                .to_string();
            format!("{na}->{nb}")
        })
        .collect()
}

/// Um boneco: tronco com cabeça e DOIS braços — a forma que uma corrente não
/// consegue expressar, porque um nó tem mais de um filho.
fn doll() -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let torso = node(&mut sim, "Torso", None);
    let head = node(&mut sim, "Head", Some(torso));
    let arm_l = node(&mut sim, "ArmL", Some(torso));
    let arm_r = node(&mut sim, "ArmR", Some(torso));
    let parts = vec![torso, head, arm_l, arm_r];
    (sim, parts)
}

/// **A ÁRVORE, não a fila.** Três arestas saindo do MESMO pai é exatamente o que
/// a corrente por seleção não produz — ela ligaria `Torso→Head→ArmL→ArmR`, uma
/// linha que descreve um boneco que não existe.
#[test]
fn a_branching_rig_gets_one_edge_per_child_not_a_chain() {
    let (mut sim, parts) = doll();
    let edges = edge_names(&mut sim, &parts);
    assert_eq!(
        edges,
        ["Torso->ArmL", "Torso->ArmR", "Torso->Head"]
            .into_iter()
            .map(String::from)
            .collect::<BTreeSet<_>>(),
        "o rig tem de ramificar no tronco; uma corrente daria Head->ArmL"
    );
}

/// **A raiz não gera aresta** — ela não tem de onde pender, e inventar uma
/// (contra o mundo, digamos) seria o gerador decidindo por conta própria que o
/// personagem fica pendurado.
#[test]
fn the_root_of_the_subtree_hangs_from_nothing() {
    let (mut sim, parts) = doll();
    let edges = rig_edges(sim.world(), &parts);
    assert_eq!(edges.len(), 3, "quatro partes, três arestas");
    let torso = named(&mut sim, "Torso");
    assert!(
        edges.iter().all(|&(_, child)| child != torso),
        "a raiz apareceu como FILHO de alguma aresta"
    );
}

/// **Um grupo é TRANSPARENTE.** Um nó sem desenho é organização, não osso: ele
/// fica fora da lista de partes, e o filho tem de se ligar ao AVÔ. Ligar ao pai
/// literal exigiria dar corpo ao grupo (um collider invisível no meio do
/// personagem); pular a aresta sem mais nada DESCONECTARIA o filho do rig.
#[test]
fn a_group_in_the_middle_is_transparent_and_the_grandchild_joins_the_grandparent() {
    let mut sim = SimWorld::new();
    let torso = node(&mut sim, "Torso", None);
    let group = node(&mut sim, "ArmsGroup", Some(torso));
    let arm = node(&mut sim, "ArmL", Some(group));
    // `group` NÃO entra: ele não desenha nada.
    let parts = vec![torso, arm];

    let edges = edge_names(&mut sim, &parts);
    assert_eq!(
        edges,
        ["Torso->ArmL"]
            .into_iter()
            .map(String::from)
            .collect::<BTreeSet<_>>(),
        "o grupo interrompeu a linhagem em vez de deixar passar"
    );
}

/// **Dois personagens selecionados são DOIS rigs.** Nenhuma aresta cruza raízes
/// — não há ancestral comum, então a pergunta *"qual o ancestral mais próximo que
/// também é parte?"* responde `None` nas duas raízes, e é isso que impede o
/// gerador de soldar dois bonecos que só estavam juntos na seleção.
#[test]
fn two_separate_characters_are_two_rigs_with_nothing_between_them() {
    let mut sim = SimWorld::new();
    let a = node(&mut sim, "A", None);
    let a_head = node(&mut sim, "AHead", Some(a));
    let b = node(&mut sim, "B", None);
    let b_head = node(&mut sim, "BHead", Some(b));
    let parts = vec![a, a_head, b, b_head];

    let edges = edge_names(&mut sim, &parts);
    assert_eq!(
        edges,
        ["A->AHead", "B->BHead"]
            .into_iter()
            .map(String::from)
            .collect::<BTreeSet<_>>(),
        "uma aresta cruzou de um personagem para o outro"
    );
}

/// **A subárvore é do que foi selecionado**, e a raiz entra nela — selecionar o
/// tronco é o gesto de *"rigar este personagem"*.
#[test]
fn the_subtree_of_a_root_includes_the_root_and_everything_under_it() {
    let (mut sim, _) = doll();
    let outsider = node(&mut sim, "Prop", None);
    let torso = named(&mut sim, "Torso");

    let all: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Transform)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    let parts = subtree_parts(sim.world(), &[torso], all, |_| true);

    let names: BTreeSet<String> = parts
        .iter()
        .map(|&e| {
            sim.world()
                .get::<Name>(e)
                .expect("name")
                .as_str()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        ["ArmL", "ArmR", "Head", "Torso"]
            .into_iter()
            .map(String::from)
            .collect::<BTreeSet<_>>()
    );
    assert!(
        !parts.contains(&outsider),
        "um objeto fora da subárvore entrou no rig"
    );
}

/// **Um candidato que o predicado recusa fica de fora** — é por aqui que o grupo
/// (sem `Sprite`) não vira osso, sem esta crate precisar saber o que um `Sprite`
/// é.
#[test]
fn a_candidate_the_predicate_refuses_is_not_a_part() {
    let (mut sim, _) = doll();
    let torso = named(&mut sim, "Torso");
    let head = named(&mut sim, "Head");

    let all: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Transform)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    let parts = subtree_parts(sim.world(), &[torso], all, |e| e != head);
    assert!(!parts.contains(&head), "o predicado foi ignorado");
    assert_eq!(parts.len(), 3);
}

/// **A ordem é ESTÁVEL.** A query do chamador entrega ordem de arquétipo, que
/// não é reproduzível; sem ordenar, dois rigs do mesmo boneco criariam os mesmos
/// joints em ordens diferentes — e a ordem é o que decide qual deles ganha o
/// sufixo quando dois nomes colidem.
#[test]
fn the_part_list_does_not_depend_on_query_order() {
    let (mut sim, _) = doll();
    let torso = named(&mut sim, "Torso");
    let all: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Transform)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    let mut reversed = all.clone();
    reversed.reverse();

    assert_eq!(
        subtree_parts(sim.world(), &[torso], all, |_| true),
        subtree_parts(sim.world(), &[torso], reversed, |_| true)
    );
}
