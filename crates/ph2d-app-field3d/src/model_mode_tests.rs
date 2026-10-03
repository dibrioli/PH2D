use super::*;
use bevy_ecs::entity::Entity;

const A: u64 = 10;
const B: u64 = 20;

fn leaves(sim: &SimWorld, root: u64) -> usize {
    let world = sim.world();
    ph2d_field_ecs::walk(world, Entity::from_bits(root))
        .into_iter()
        .filter(|(e, _)| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e),
                Some(ph2d_field_ecs::FieldNode {
                    shape: ph2d_field::NodeShape::Leaf(_)
                })
            )
        })
        .count()
}

fn cooked(sim: &SimWorld, root: u64) -> Option<ph2d_field::FieldDoc> {
    ph2d_field_ecs::cook(sim.world(), Entity::from_bits(root)).and_then(Result::ok)
}

/// ⭐ GATE — as três leis puras do modo.
#[test]
fn the_three_laws_of_the_model_mode() {
    // tem em mãos: Edit, a peça do modo, o painel aberto — as três.
    assert!(holds(ObjectMode::Edit, true, true));
    assert!(!holds(ObjectMode::Edit, false, true), "outra peça");
    assert!(!holds(ObjectMode::Edit, true, false), "o X fechou o painel");
    assert!(
        !holds(ObjectMode::Draw, true, true),
        "o Edit do Flip não é este"
    );
    // o painel segue o modo: só quando um Edit do Model ACABOU.
    assert!(releases(false, true));
    assert!(!releases(true, true), "o modo continua");
    assert!(
        !releases(false, false),
        "nada estava em curso — não fecha o painel de ninguém"
    );
    // quem pede o Edit.
    assert_eq!(
        wanted(Some(B), Some(A), false, false, false, false),
        Some(B),
        "a nascida"
    );
    assert_eq!(
        wanted(None, Some(A), true, false, false, false),
        Some(A),
        "a porta antiga"
    );
    assert_eq!(
        wanted(None, Some(A), false, false, false, false),
        None,
        "painel fechado"
    );
    assert_eq!(
        wanted(None, Some(A), true, true, false, false),
        None,
        "o painel está a fechar"
    );
    assert_eq!(
        wanted(None, Some(A), true, false, true, false),
        None,
        "já está em Edit"
    );
    assert_eq!(
        wanted(None, Some(A), true, false, false, true),
        None,
        "já pediu nesta abertura"
    );
    assert_eq!(
        wanted(None, None, true, false, false, false),
        None,
        "sem peça não há Edit"
    );
}

/// ⭐⭐ GATE (spec/06 F3, o gate por módulo) — **duas peças, Edit numa, a outra intocada**: o que se
/// coze e desenha é a peça do modo, a forma acrescentada cai nela, e a outra não muda ao bit.
///
/// (Mutação: `root_in_hand` a ignorar o alvo ⇒ a forma cai na 1.ª ⇒ RED.)
#[test]
fn two_pieces_edit_in_one_and_the_other_is_untouched() {
    forget();
    let mut sim = SimWorld::new();
    let a = crate::object_add::add(&mut sim);
    let b = crate::object_add::add(&mut sim);
    let mut tools = ToolRegistry::new();
    let mut family = Family::new(&mut sim, false);
    assert!(family.enter(ObjectMode::Edit, b, &mut tools));
    assert_eq!(target(), Some(b));
    assert_eq!(
        take_panel_request(),
        Some(true),
        "entrar abre o painel (que arma o módulo)"
    );
    assert_eq!(
        crate::scene::root_in_hand(sim.world_mut()).map(Entity::to_bits),
        Some(b)
    );
    let a_before = cooked(&sim, a);
    let slot = crate::shapes::slot_of("panel.model3d.add.sphere").expect("a esfera");
    crate::smoke::ask_shape(slot);
    crate::scene::sync_scene(&mut sim, None, 0.0);
    let doc = crate::scene::sync_scene(&mut sim, None, 0.0);
    assert_eq!(leaves(&sim, b), 2, "a forma não caiu na peça em Edit");
    assert_eq!(leaves(&sim, a), 1);
    assert_eq!(cooked(&sim, a), a_before, "a outra peça mudou");
    assert_eq!(
        doc,
        cooked(&sim, b),
        "o documento cozido não é o da peça em Edit"
    );
}

/// ⭐ GATE — as PARTES do Edit são as formas da peça e as luzes da cena, nunca a outra peça; e uma
/// forma responde pelo dono dela (o seletor e o `Tab` sobre uma forma).
#[test]
fn the_parts_of_a_piece_are_its_shapes_and_the_lights() {
    forget();
    let mut sim = SimWorld::new();
    let a = crate::object_add::add(&mut sim);
    let b = crate::object_add::add(&mut sim);
    let mut family = Family::new(&mut sim, false);
    let parts_a = family.parts(a).expect("o Edit do Model edita partes");
    let parts_b = family.parts(b).expect("idem");
    let mut q = sim
        .world_mut()
        .query::<(Entity, &ph2d_field_ecs::FieldLight)>();
    let light = q
        .iter(sim.world())
        .next()
        .map(|(e, _)| e.to_bits())
        .expect("a luz");
    assert!(
        parts_a.contains(&light) && parts_b.contains(&light),
        "a luz é da cena"
    );
    assert!(
        !parts_a.contains(&b) && !parts_a.contains(&a),
        "uma peça não é parte"
    );
    let shape_a = *parts_a
        .iter()
        .find(|p| **p != light)
        .expect("a esfera de A");
    assert!(!parts_b.contains(&shape_a), "a forma de A é parte de B");
    assert_eq!(family.owner_of(shape_a), Some(a));
    assert_eq!(family.owner_of(light), None, "a luz não tem dono");
    assert_eq!(family.parts(light), None, "uma luz não é uma peça");
}

/// ⭐ GATE — **o painel segue o modo**: um Edit que acabou por outra porta (o X, a peça apagada)
/// larga a peça e fecha o painel; a porta antiga (o painel aberto sem o modo) pede o Edit uma vez.
#[test]
fn the_panel_follows_the_mode_and_an_old_door_asks_once() {
    forget();
    let mut sim = SimWorld::new();
    let a = crate::object_add::add(&mut sim);
    let mut tools = ToolRegistry::new();
    let mut family = Family::new(&mut sim, true);
    assert_eq!(
        family.wants(&mut tools),
        Some((a, ObjectMode::Edit)),
        "a nascida pede o Edit"
    );
    assert!(family.enter(ObjectMode::Edit, a, &mut tools));
    let _ = take_panel_request();
    let edit = ActiveMode {
        entity: a,
        mode: ObjectMode::Edit,
    };
    family.follow(Some(edit), &mut tools);
    assert_eq!(target(), Some(a));
    assert_eq!(
        take_panel_request(),
        None,
        "o modo em curso não mexe no painel"
    );
    // O modo acabou sem passar pelo `leave` (o X: `holds` falso ⇒ o quadro não chama `leave`).
    family.follow(None, &mut tools);
    assert_eq!(target(), None, "a peça ficou em mãos sem o modo");
    assert_eq!(
        take_panel_request(),
        Some(false),
        "o painel ficou aberto sem o modo"
    );
    // Uma porta antiga reabre o painel sem o modo: pede o Edit UMA vez.
    let mut family = Family::new(&mut sim, false);
    family.follow(None, &mut tools);
    let mut family = Family::new(&mut sim, true);
    assert_eq!(family.wants(&mut tools), Some((a, ObjectMode::Edit)));
    assert_eq!(
        family.wants(&mut tools),
        None,
        "pediu duas vezes na mesma abertura"
    );
}
