//! ⭐⭐⭐ **A TROCA por um componente sem parentesco** (plano F5, o último critério) — os gates do
//! item de menu cujo sujeito **não é o cartão**.
//!
//! # Porque é um ficheiro próprio
//!
//! O irmão [`super::tests`] mede os verbos cujo sujeito É o cartão (editar · instanciar · quem usa ·
//! tirar da biblioteca). Esta família responde a outra pergunta — *o leitor decide, ou entrega a
//! alguém que descarta?* (§5.0) — e traz o próprio arnês de duas receitas sem parentesco.
//!
//! ⚠️ E o corte não é de gosto: juntos passavam o tecto de LOC do shell (`629` contra `600`), e a
//! lei da casa é **decompor por responsabilidade, nunca subir a allowlist**.
//!
//! ⚠️ **Estes gates atravessam a shell**, e não o construtor do mapa: o mecanismo tem gates próprios
//! em `instance_swap_match_tests.rs`.

use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, StableId, Transform};
use ph2d_editor::interaction::drag_payload::DragPayload;
use std::collections::BTreeMap;

// ── A TROCA por um componente sem parentesco (plano F5, o último critério) ───────────────────
//
// ⚠️ **Estes gates atravessam a shell**, e não o construtor do mapa: o mecanismo tem gates próprios
// em `instance_swap_match_tests.rs`. O que se mede aqui é a **terceira pergunta** do §5.0 — *o
// leitor decide, ou entrega a alguém que descarta?* — sobre o único verbo deste menu cujo sujeito
// não é o cartão.

/// Duas receitas sem antepassado comum e uma cópia da primeira na cena.
fn two_recipes_and_a_copy(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
) -> (u64, ph2d_ecs::Entity) {
    let mut make = |name: &str| {
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new(name), MasterRoot))
            .id();
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Body"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(root),
        ));
        root
    };
    let car = make("Car");
    let truck = make("Truck");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = ph2d_app_components::instance_docs::empty_docs();
    let copy = ph2d_app_components::instantiate::instantiate_master(
        sim,
        r,
        car,
        None,
        &mut ph2d_app_components::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        ph2d_app_components::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    (sim.world().get::<StableId>(truck).expect("id").0, copy)
}

/// Corre o verbo do cartão com a selecção dada, e devolve `(agiu, o que ele DISSE)`.
///
/// ⚠️ **A voz vem inteira, e não como `!toasts.is_empty()`.** Cada caminho vazio deste verbo diz
/// uma coisa diferente e as três são accionáveis; um gate que só conta toasts deixa colapsá-las
/// numa só sem sangrar — e a que sobreviveria seria a que não diz o que fazer.
fn run_replace(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    verb: ph2d_editor::action_bus::AssetCardAction,
    asset: DragPayload,
    selected: Option<u64>,
) -> (bool, String) {
    let mut echo = ph2d_app_components::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    gizmo.replace_selection(selected);
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = ph2d_app_components::instance_docs::empty_docs();
    let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let mut select_out = None;
    let acted = super::drain(
        asset,
        verb,
        sim,
        r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    let said: Vec<&str> = toasts.iter().map(|t| t.message.as_str()).collect();
    (acted, said.join(" | "))
}

fn master_of(sim: &SimWorld, e: ph2d_ecs::Entity) -> u64 {
    sim.world()
        .get::<ph2d_ecs::InstanceOf>(e)
        .expect("a copia tem elo")
        .master
}

/// ⭐⭐⭐ **O clique no item chega ao MUNDO** — a cópia passa a ser do outro componente.
///
/// ⚠️ Um seam de painel prova que o clique chega ao **barramento**; ele não prova que alguém do
/// outro lado o lê. É a espécie *«o dreno de UM BRAÇO SÓ»* do §5.0, e o `match` do
/// [`super::drain`] não termina em `_ => {}` só porque o compilador o proíbe — o do barramento
/// termina, e é por isso que o censo textual irmão existe.
///
/// **Mutação que deve sangrar:** o braço a devolver `false` sem chamar o `swap`.
#[test]
fn replacing_the_selection_makes_the_copy_belong_to_the_other_component() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    assert_ne!(
        master_of(&sim, copy),
        truck_id,
        "a fixtura ja' comecava la'"
    );

    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelectionByName,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        Some(copy.to_bits()),
    );
    assert!(acted, "o verbo nao mexeu no documento");
    assert!(
        spoke.contains("Truck") && spoke.contains("Replaced"),
        "a voz nao diz o que aconteceu nem a quem: {spoke:?}"
    );
    assert_eq!(
        master_of(&sim, copy),
        truck_id,
        "o elo da copia nao mudou — o clique morreu a um passo do efeito"
    );
}

/// ⭐⭐ **O sujeito é a RAIZ da cópia, e não a entidade clicada.**
///
/// ⚠️ O artista escolhe uma PEÇA de dentro da cópia tantas vezes quantas escolhe a raiz — clicar no
/// canvas dá a peça que está debaixo do rato. Exigir a raiz faria o gesto falhar sem dizer porquê,
/// que é o modo de falha mais caro deste menu.
///
/// **Mutação que deve sangrar:** trocar o `instance_root_of` pelos bits escolhidos.
#[test]
fn picking_a_piece_inside_the_copy_replaces_the_whole_copy() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    let piece = sim
        .world()
        .get::<ph2d_ecs::Children>(copy)
        .expect("a copia tem pecas")
        .iter()
        .copied()
        .next()
        .expect("uma peca");
    assert_ne!(piece, copy, "a fixtura tinha de dar uma PECA");

    let (acted, _) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelection,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        Some(piece.to_bits()),
    );
    assert!(acted);
    assert_eq!(
        master_of(&sim, copy),
        truck_id,
        "escolher uma peca de dentro nao alcancou a copia"
    );
}

/// ⛔ **Sem nada escolhido a recusa DIZ o que fazer** — o sujeito deste verbo não está no cartão,
/// então «não aconteceu nada» seria a leitura de um item partido.
#[test]
fn replacing_with_nothing_picked_says_what_to_pick() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, _copy) = two_recipes_and_a_copy(&mut sim, &r);
    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelection,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        None,
    );
    assert!(!acted, "sem sujeito nao ha' edicao");
    assert!(
        spoke.contains("Pick the copy"),
        "a recusa nao diz O QUE FAZER — «nada aconteceu» le-se como item partido: {spoke:?}"
    );
}

/// ⛔ **Uma imagem não é um componente**, e a quarta recusa da tabela plana também fala.
#[test]
fn an_image_cannot_replace_a_copy_and_it_says_so() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (_truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    let before = master_of(&sim, copy);
    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelectionByTree,
        DragPayload::Image { asset: [3; 32] },
        Some(copy.to_bits()),
    );
    assert!(!acted);
    assert!(
        spoke.contains("not a prefab"),
        "a recusa nao nomeia o FACTO: {spoke:?}"
    );
    assert_eq!(master_of(&sim, copy), before, "a copia mexeu-se");
}

/// ⭐⭐⭐ **Os três itens são TRÊS LEIS, e o modo sai do VERBO.**
///
/// ⚠️ O `match verb` do [`super::replace_selection`] termina em `_ =>`, então um modo novo cairia no
/// *«não leves nada»* **em silêncio** — a espécie *«o dreno de um braço só»* do §5.0, aqui dentro de
/// uma função em vez de num barramento. Este gate mede as três células de uma vez: com a ordem dos
/// irmãos TROCADA entre as duas receitas, cada modo re-chaveia a excepção para uma peça diferente.
///
/// **Mutação que deve sangrar:** colapsar dois braços do `match verb`.
#[test]
fn the_three_replace_items_are_three_different_laws() {
    use ph2d_editor::action_bus::AssetCardAction as A;

    let kid = |sim: &SimWorld, root: ph2d_ecs::Entity, name: &str| -> u64 {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
                return sim.world().get::<StableId>(e).expect("id").0;
            }
            if let Some(k) = sim.world().get::<ph2d_ecs::Children>(e) {
                stack.extend(k.iter().copied());
            }
        }
        panic!("nao achei {name:?}");
    };

    for (verb, want) in [
        (A::ReplaceSelectionByName, "Body"),
        (A::ReplaceSelectionByTree, "Wheel"),
    ] {
        let mut sim = SimWorld::new();
        let r = crate::init::build_component_registry();
        // ⚠️ A ordem dos irmãos ao CONTRÁRIO — o único arranjo em que os dois modos discordam.
        let mut make = |name: &str, pieces: [&str; 2]| {
            let root = sim
                .world_mut()
                .spawn((Transform::IDENTITY, Name::new(name), MasterRoot))
                .id();
            for p in pieces {
                sim.world_mut().spawn((
                    Transform::IDENTITY,
                    Name::new(p),
                    ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
                    ChildOf(root),
                ));
            }
            root
        };
        let car = make("Car", ["Body", "Wheel"]);
        let truck = make("Truck", ["Wheel", "Body"]);
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        let (mut sc, mut mp) = ph2d_app_components::instance_docs::empty_docs();
        let copy = ph2d_app_components::instantiate::instantiate_master(
            &mut sim,
            &r,
            car,
            None,
            &mut ph2d_app_components::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
            ph2d_app_components::instantiate::ArtLink::Own,
        )
        .expect("instanciou");
        // A excepção do artista, escrita à mão na chave: é ela que o re-key move.
        let car_body = kid(&sim, car, "Body");
        let truck_id = sim.world().get::<StableId>(truck).expect("id").0;
        let type_id = 7u64;
        sim.world_mut()
            .entity_mut(copy)
            .insert(ph2d_ecs::ObjectInstance {
                overrides: std::collections::BTreeSet::from([ph2d_ecs::OverrideKey {
                    piece: car_body,
                    type_id,
                }]),
                ..Default::default()
            });

        let (acted, _) = run_replace(
            &mut sim,
            &r,
            verb,
            DragPayload::Prefab {
                stable_id: truck_id,
            },
            Some(copy.to_bits()),
        );
        assert!(acted, "{verb:?} nao agiu");
        let landed = sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(copy)
            .expect("instancia")
            .overrides
            .iter()
            .map(|k| k.piece)
            .collect::<Vec<_>>();
        assert_eq!(
            landed,
            vec![kid(&sim, truck, want)],
            "{verb:?} nao levou a excepcao para a peca {want:?} — os modos colapsaram"
        );
    }
}
