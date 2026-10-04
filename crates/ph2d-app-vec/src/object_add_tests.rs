use super::*;

/// ⭐ **Cada forma do menu nasce como objecto VETORIAL e VIVA** — com `VecPathRef` (o marcador do
/// tipo) e `VecShape` (o que a forma desenhada também ganha), pela ferramenta de forma.
///
/// (Mutação: tirar o `make_committed_shape_live` ⇒ RED; trocar `on_drag` por nada ⇒ a forma não
/// vence o arrasto mínimo e não nasce ⇒ RED.)
#[test]
fn every_menu_shape_is_born_a_live_vector_object() {
    for entry in SHAPES {
        let mut sim = SimWorld::new();
        let mut scene = VecScene::default();
        let mut vec = crate::state::VecState::default();
        let bits = add(
            *entry,
            &mut sim,
            &mut scene,
            &mut vec,
            [10.0, 20.0],
            1.0,
            800.0,
        )
        .expect("a entrada é desta família")
        .unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()));
        let e = ph2d_ecs::Entity::from_bits(bits);
        assert!(
            sim.world().get::<ph2d_ecs::VecPathRef>(e).is_some(),
            "{} não é um objecto vetorial",
            entry.key.key()
        );
        assert!(
            sim.world().get::<ph2d_ecs::VecShape>(e).is_some(),
            "{} nasceu morta — a forma desenhada nasce viva",
            entry.key.key()
        );
        assert_eq!(scene.paths().len(), 1);
    }
}

/// **O tamanho sai da VISTA** — um quarto da altura visível, em mundo.
#[test]
fn the_size_follows_the_view() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::default();
    let mut vec = crate::state::VecState::default();
    add(
        RECTANGLE,
        &mut sim,
        &mut scene,
        &mut vec,
        [0.0, 0.0],
        2.0,
        400.0,
    )
    .expect("é desta família")
    .expect("nasce");
    let (lo, hi) = vec.shape.bounds();
    let side = 400.0 * SIDE_OF_VIEW * 2.0;
    assert!((hi[0] - lo[0] - side).abs() < 1e-9, "{lo:?}..{hi:?}");
    assert!((hi[1] - lo[1] - side).abs() < 1e-9, "{lo:?}..{hi:?}");
}

/// **Uma entrada de outra família não é desta** — e nada nasce.
#[test]
fn an_entry_of_another_family_is_not_ours() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::default();
    let mut vec = crate::state::VecState::default();
    let empty = ph2d_editor_core::object_add::EMPTY;
    assert!(add(empty, &mut sim, &mut scene, &mut vec, [0.0; 2], 1.0, 800.0).is_none());
    assert!(scene.paths().is_empty());
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector) — **as ferramentas de criar não criam nada**: armam a ferramenta e
/// não são formas; e o menu tem as duas listas, sem sobra.
#[test]
fn a_create_tool_entry_arms_the_tool_and_is_not_a_shape() {
    for entry in TOOLS {
        let mut vec = crate::state::VecState::default();
        let mut sim = SimWorld::new();
        let mut scene = VecScene::default();
        assert!(
            add(*entry, &mut sim, &mut scene, &mut vec, [0.0; 2], 1.0, 800.0).is_none(),
            "{} criou uma forma",
            entry.key.key()
        );
        assert!(arm(*entry, &mut vec), "{}", entry.key.key());
        assert_eq!(vec.edit.armed, tool_of(*entry));
        assert!(scene.paths().is_empty());
    }
    for entry in SHAPES {
        assert!(!arm(*entry, &mut crate::state::VecState::default()));
    }
    assert_eq!(ENTRIES.len(), SHAPES.len() + TOOLS.len());
    assert!(SHAPES.iter().chain(TOOLS).all(|e| ENTRIES.contains(e)));
}
