//! Os gates da cena 4 (ADR-0164 / F5, o último critério).
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente* — a
//! ausente não é acreditada (CLAUDE.md §5.0). Estes gates medem as **promessas dos `println!`**:
//! que os três itens existem com aqueles nomes, que as duas receitas de facto não têm parentesco, e
//! que a ordem trocada das peças produz de facto duas respostas diferentes.

use super::*;
use crate::instance_swap_match::WhenUnrelated;
use ph2d_ecs::{Children, StableId};

fn build() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    Entity,
    Entity,
    Vec<Entity>,
) {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (car, truck, cars) = spawn_replace_scene(
        &mut sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    (sim, r, car, truck, cars)
}

fn sid(sim: &SimWorld, e: Entity) -> u64 {
    sim.world().get::<StableId>(e).expect("sid").0
}

fn named(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' {name:?} debaixo de {root:?}");
}

/// ⭐⭐⭐ **As duas receitas NÃO TÊM PARENTESCO** — se tivessem, o mapa derivado responderia sozinho
/// e os três itens do menu nunca seriam consultados. *A cena mediria outra coisa e ninguém saberia.*
#[test]
fn the_two_recipes_have_no_kinship_which_is_the_whole_point_of_the_scene() {
    let (mut sim, _r, car, truck, _cars) = build();
    let (a, b) = (sid(&sim, car), sid(&sim, truck));
    assert!(
        crate::instance_variant::piece_map(&mut sim, a, b).is_none(),
        "a cena montou duas receitas APARENTADAS — os tres modos ficariam mudos"
    );
}

/// ⭐⭐⭐ **A ordem trocada produz de facto DUAS respostas** — a promessa central dos passos 3 e 4.
///
/// ⛔ Com as peças na mesma ordem dos dois lados os dois modos coincidiriam, e a cena ensinaria que
/// dois dos três itens são enfeite.
#[test]
fn by_name_and_by_position_land_on_different_pieces_in_this_scene() {
    let (mut sim, _r, car, truck, _cars) = build();
    let body = sid(&sim, named(&sim, car, "Body"));
    let (t_body, t_wheel) = (
        sid(&sim, named(&sim, truck, "Body")),
        sid(&sim, named(&sim, truck, "Wheel")),
    );
    let (a, b) = (sid(&sim, car), sid(&sim, truck));
    let by_id: std::collections::BTreeMap<u64, Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &StableId)>();
        q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
    };
    let name = crate::instance_swap_match::rematch(&sim, &by_id, a, b, WhenUnrelated::ByName)
        .expect("por nome");
    let tree = crate::instance_swap_match::rematch(&sim, &by_id, a, b, WhenUnrelated::ByHierarchy)
        .expect("por posicao");
    assert_eq!(
        name.map.get(&body),
        Some(&t_body),
        "por NOME a barra tinha de achar a barra"
    );
    assert_eq!(
        tree.map.get(&body),
        Some(&t_wheel),
        "por POSICAO a barra tinha de achar a roda — a ordem nao ficou trocada"
    );
}

/// ⭐⭐⭐ **Os três itens que os passos nomeiam saem da TABELA do menu, e existem lá.**
///
/// ⚠️ Duas metades. A primeira: o `item()` da cena resolve os três ids — sem ela um item renomeado
/// ou removido faria os passos imprimirem *«(este item saiu do menu)»* ao Enio, em silêncio para
/// toda a suíte. A segunda: **nenhum rótulo está escrito à mão no fonte da cena** — é o que impede
/// alguém de «simplificar» a derivação de volta para uma cópia que envelhece.
#[test]
fn the_printed_steps_read_their_labels_from_the_menu_table() {
    let src = include_str!("instance_replace_smoke.rs");
    for id in [
        ph2d_editor::ids::CTX_MENU_ASSET_REPLACE,
        ph2d_editor::ids::CTX_MENU_ASSET_REPLACE_BY_NAME,
        ph2d_editor::ids::CTX_MENU_ASSET_REPLACE_BY_TREE,
    ] {
        let label = super::item(id);
        assert_ne!(
            label, "(este item saiu do menu)",
            "a cena promete um item que a tabela do menu ja' nao tem"
        );
        // ⚠️ A varredura é só de CÓDIGO: o doc acima fala de rótulos de propósito.
        for (i, l) in src.lines().enumerate() {
            let code = l.split_once("//").map_or(l, |(before, _)| before);
            assert!(
                !code.contains(label),
                "linha {}: o rotulo {label:?} esta' COPIADO no fonte da cena — ele tem de vir da \
                 tabela, senao um renomear deixa o smoke a ensinar o nome velho",
                i + 1
            );
        }
    }
}

/// ⚠️ **`\\` num literal de Rust NÃO é continuação de linha** — é uma barra, uma quebra e a
/// indentação, no meio da frase que o Enio lê. Custou um report em 2026-09-05.
#[test]
fn the_printed_steps_have_no_stray_backslash() {
    let src = include_str!("instance_replace_smoke.rs");
    // ⚠️ A própria linha do doc acima contém o padrão — por isso a varredura é só de CÓDIGO.
    for (i, l) in src.lines().enumerate() {
        let code = l.split_once("//").map_or(l, |(before, _)| before);
        assert!(
            !code.trim_end().ends_with("\\\\"),
            "linha {}: `\\\\` no fim de um literal parte a mensagem em duas",
            i + 1
        );
    }
}

/// ⭐⭐ **O segundo Carro é a TESTEMUNHA** — trocar um não toca no outro, que é o que o passo 3
/// promete entre parênteses.
#[test]
fn replacing_one_car_leaves_the_other_alone() {
    let (mut sim, _r, car, truck, cars) = build();
    assert_eq!(cars.len(), 2, "a cena tem de montar DOIS carros");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let (car_id, truck_id) = (sid(&sim, car), sid(&sim, truck));
    crate::instance_variant::swap(
        &mut sim,
        &mut echo,
        cars[0],
        truck_id,
        WhenUnrelated::ByName,
    )
    .expect("trocar o da esquerda");
    let of = |sim: &SimWorld, e: Entity| {
        sim.world()
            .get::<ph2d_ecs::InstanceOf>(e)
            .expect("elo")
            .master
    };
    assert_eq!(of(&sim, cars[0]), truck_id);
    assert_eq!(
        of(&sim, cars[1]),
        car_id,
        "o carro da direita mudou — a cena mentiria no parentese do passo 3"
    );
}
