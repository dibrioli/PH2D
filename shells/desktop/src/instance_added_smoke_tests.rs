//! Os gates da cena 6 (ADR-0164 / F5.11).
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente.*
//! Este gate corre o CAMINHO INTEIRO que os passos impressos descrevem — duplicar o braço, aplicar
//! — e mede o que a tela mostraria no fim, robô a robô.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{Children, Entity, Name, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

fn build() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    Entity,
    Vec<Entity>,
) {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (master, copies) = crate::instance_removed_smoke::spawn_robot_scene(
        &mut sim,
        &r,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    (sim, r, master, copies)
}

fn pass(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
    for _ in 0..2 {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        sync_instances(
            sim,
            r,
            &PhysicsBridge::new(),
            echo,
            &mut OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
    }
}

/// Quantas peças desta cópia têm nome de braço — é isso que o dono conta na tela.
fn arms(sim: &SimWorld, root: Entity) -> usize {
    let mut n = 0;
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root
            && sim
                .world()
                .get::<Name>(e)
                .is_some_and(|x| x.0.starts_with("Arm"))
        {
            n += 1;
        }
        if let Some(k) = sim.world().get::<Children>(e) {
            stack.extend(k.iter().copied());
        }
    }
    n
}

fn piece(sim: &SimWorld, root: Entity, name: &str) -> Option<Entity> {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return Some(e);
        }
        if let Some(k) = sim.world().get::<Children>(e) {
            stack.extend(k.iter().copied());
        }
    }
    None
}

/// ⭐⭐⭐ **O CAMINHO INTEIRO dos passos impressos**, medido: duplicar · aplicar · as três recebem.
///
/// ⚠️ As asserções são as promessas dos `println!`, uma a uma — incluindo as **duas** metades do
/// sinal de erro impresso: *«só o robô do meio ficar com dois braços»* (a peça não chegou à receita)
/// e *«o robô do meio ficar com TRÊS»* (o elo não nasceu no original, e o passe materializou uma
/// segunda na cópia onde o dono trabalhou).
///
/// ⚠️ **Ele passa pelas PORTAS do produto** — o `duplicate_subtree` é o que o *Duplicate* da
/// Hierarquia chama, e o `promote` é o que o botão do cartão chama.
#[test]
fn the_printed_steps_are_what_the_scene_actually_does() {
    let (mut sim, r, master, copies) = build();
    assert_eq!(copies.len(), 3, "os passos falam de TRES robos");
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (mid, witness) = (copies[1], copies[2]);

    // PASSO 2 — duplicar o braço do robô do meio, pela porta que a Hierarquia usa.
    let arm = piece(&sim, mid, "Arm").expect("o braco do meio");
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let copy = crate::instantiate::duplicate_subtree(
        &mut sim,
        &r,
        arm,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        [0.4, 0.0],
    )
    .expect("duplicou o braco");
    // ⚠️ **O nome é o que o smoke IMPRIME no rótulo do botão** — se o `unique_name` mudar de
    // sufixo, o passo 3 manda o dono procurar um botão que não existe.
    assert_eq!(
        sim.world().get::<Name>(copy).map(|n| n.0.clone()),
        Some("Arm (1)".to_string()),
        "o Duplicate deixou outro nome — o rotulo impresso no PASSO 3 deixou de ser o do botao"
    );
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        arms(&sim, mid),
        2,
        "o robo do meio nao ficou com dois bracos"
    );
    assert_eq!(
        arms(&sim, witness),
        1,
        "duplicar dentro de UMA copia chegou a's outras antes de alguem aplicar"
    );

    // PASSO 3 — o botão do cartão.
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instance_added::promote(
        &mut sim,
        &r,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        copy,
    )
    .expect("promoveu");
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert_eq!(arms(&sim, master), 2, "a receita nao ficou com dois bracos");
    for (i, &c) in copies.iter().enumerate() {
        assert_eq!(
            arms(&sim, c),
            2,
            "o robo {} ficou com {} bracos — a promessa e' DOIS em todos",
            i + 1,
            arms(&sim, c)
        );
    }
}

/// ⛔⛔ **O rótulo impresso é DERIVADO, e este ficheiro não pode ter a frase escrita à mão.**
///
/// É a lei que a cena `=4` já paga para os itens de menu: um smoke que promete uma frase que o
/// painel já não pinta manda o dono procurar um botão que não existe, e o report que volta é
/// *«não funcionou»* — indistinguível de um defeito real.
#[test]
fn the_button_label_is_never_hand_copied_into_this_scene() {
    let src = include_str!("instance_added_smoke.rs");
    let code: String = src
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(before, _)| before))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains("Add \u{201c}"),
        "a frase do botao foi escrita a' mao nesta cena — ela tem de sair do `AddedRow::label`"
    );
}

/// ⚠️ **`\\` num literal de Rust NÃO é continuação de linha** — a irmã desta régua na cena 5.
#[test]
fn the_printed_steps_have_no_stray_backslash() {
    let src = include_str!("instance_added_smoke.rs");
    for (i, l) in src.lines().enumerate() {
        let code = l.split_once("//").map_or(l, |(before, _)| before);
        assert!(
            !code.trim_end().ends_with("\\\\"),
            "linha {}: `\\\\` no fim de um literal parte a mensagem em duas",
            i + 1
        );
    }
}
