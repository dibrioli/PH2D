use super::*;

fn registry() -> ComponentRegistry {
    crate::component_registry_for_tests::registo()
}

/// ⭐ **Cada objecto de jogo nasce com o componente que lhe dá o nome** — pela porta do `+` do
/// Inspector, e na raiz.
///
/// (Mutação: o som anexar `ph2d::ecs::GameCamera` ⇒ RED — sobrevivia antes do lado independente.)
#[test]
fn every_game_object_is_born_with_its_component() {
    let reg = registry();
    for entry in ENTRIES {
        let mut sim = SimWorld::new();
        let bits = add(*entry, &mut sim, &reg, &[])
            .expect("a entrada é desta família")
            .unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()));
        let e = ph2d_ecs::Entity::from_bits(bits);
        let name = component_of(*entry).expect("tem componente");
        let reg_entry = reg
            .get_by_id(ph2d_ecs::scene::stable_type_id(name))
            .unwrap_or_else(|| panic!("{name} não está no registo"));
        assert!(
            matches!((reg_entry.serialize)(sim.world(), e), Ok(Some(_))),
            "{} nasceu sem {name}",
            entry.key.key()
        );
        // ⚠️ **O lado INDEPENDENTE:** o tipo concreto que cada entrada promete, escrito aqui e não
        // lido do `component_of` — a 1.ª redacção lia-o de lá e a mutação «o som anexa uma
        // câmara» sobreviveu (03/10).
        let w = sim.world();
        let concrete = match entry.key.key() {
            "object_add.game.camera" => w.get::<ph2d_ecs::GameCamera>(e).is_some(),
            "object_add.game.body" => w.get::<ph2d_physics_ecs::RigidBody>(e).is_some(),
            "object_add.game.sound" => w.get::<ph2d_ecs::AudioSource2D>(e).is_some(),
            "object_add.game.hud" => w.get::<ph2d_ecs::UiCanvas>(e).is_some(),
            other => panic!("entrada de jogo sem tipo esperado: {other}"),
        };
        assert!(concrete, "{} nasceu sem o seu componente", entry.key.key());
        assert!(sim.world().get::<ph2d_ecs::ChildOf>(e).is_none());
        assert_eq!(
            sim.world().get::<Name>(e).map(|n| n.0.as_str()),
            Some(entry.key.tr()),
            "o nome é o rótulo do menu"
        );
    }
}

/// **Uma entrada de outra família não é desta** — e nada nasce.
#[test]
fn an_entry_of_another_family_is_not_ours() {
    let mut sim = SimWorld::new();
    assert!(
        add(
            ph2d_editor_core::object_add::EMPTY,
            &mut sim,
            &registry(),
            &[]
        )
        .is_none()
    );
    let mut q = sim.world_mut().query::<&Transform>();
    assert_eq!(q.iter(sim.world()).count(), 0, "nasceu um objecto");
}

/// ⭐ **O vazio nasce com a BASE, e nada além dela** (ADR-0166 / F3).
///
/// (Mutação: acrescentar `ph2d_render::Sprite::default()` ao spawn ⇒ RED.)
#[test]
fn an_empty_object_is_born_with_the_base_and_nothing_else() {
    let mut sim = SimWorld::new();
    let e = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim, "Object"));
    let w = sim.world();
    assert!(w.get::<Transform>(e).is_some(), "falta o Transform");
    assert!(w.get::<Name>(e).is_some(), "falta o Name");
    assert!(
        w.get::<ph2d_render::Sprite>(e).is_none(),
        "um objeto VAZIO nasceu com Sprite — o Inspector mostraria as seccoes de imagem"
    );
    assert!(
        w.get::<ph2d_ecs::SliceNine>(e).is_none()
            && w.get::<ph2d_ecs::NamedAnchorList>(e).is_none()
            && w.get::<ph2d_physics_ecs::RigidBody>(e).is_none(),
        "um objeto VAZIO nasceu com um componente autorado"
    );
}

/// ⚠️ **Ele nasce na RAIZ, com ordem e identidade explícitas.**
///
/// (Mutação: tirar qualquer um dos dois assigners ⇒ RED.)
#[test]
fn it_is_a_root_with_an_explicit_order_and_identity() {
    let mut sim = SimWorld::new();
    let e = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim, "Object"));
    let w = sim.world();
    assert!(w.get::<ph2d_ecs::ChildOf>(e).is_none());
    assert!(
        w.get::<ph2d_ecs::RootOrder>(e).is_some(),
        "raiz sem RootOrder: a ordem passa a desempatar por bits de alocacao"
    );
    assert!(
        w.get::<ph2d_ecs::StableId>(e)
            .is_some_and(|id| !id.is_none()),
        "raiz sem StableId: a identidade nao sobrevive ao respawn do undo"
    );
}

/// **Dois cliques dão dois objetos, com nomes e identidades diferentes.**
#[test]
fn two_clicks_make_two_objects_with_different_names() {
    let mut sim = SimWorld::new();
    let a = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim, "Object"));
    let b = ph2d_ecs::Entity::from_bits(spawn_empty_root(&mut sim, "Object"));
    assert_ne!(a, b);
    let w = sim.world();
    let na = w.get::<Name>(a).map(|n| n.0.clone());
    let nb = w.get::<Name>(b).map(|n| n.0.clone());
    assert!(na.is_some() && nb.is_some());
    assert_ne!(na, nb, "dois objetos novos com o MESMO nome");
    assert_ne!(
        w.get::<ph2d_ecs::StableId>(a),
        w.get::<ph2d_ecs::StableId>(b),
        "dois objetos novos com a mesma identidade"
    );
}

/// ⭐⭐ **Cada marcador deriva o SEU tipo** — o objecto que o menu cria lê-se como o tipo que o
/// menu prometeu.
///
/// Os dois lados ficam amarrados: o tipo CONCRETO que o [`kind_of`] pergunta e o NOME canónico do
/// [`ObjectKind::marker`] têm de ser o mesmo componente (o registo reconhece-o no objecto).
///
/// (Mutação: apagar o braço do Flip no `kind_of` ⇒ RED — era o estado até 03/10.)
#[test]
fn every_marker_derives_its_kind() {
    use crate::component_attach::kind_of;
    use ph2d_component_desc::ObjectKind;
    let reg = registry();
    for kind in ObjectKind::ALL {
        let mut sim = SimWorld::new();
        let mut e = sim.world_mut().spawn(Transform::IDENTITY);
        match kind {
            ObjectKind::Empty => {}
            ObjectKind::Image => {
                e.insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]));
            }
            ObjectKind::Vector => {
                e.insert(ph2d_ecs::VecPathRef(1));
            }
            ObjectKind::Flip => {
                e.insert(ph2d_ecs::FlipObjectRef(1));
            }
            ObjectKind::Painted => {
                e.insert(ph2d_ecs::PaintedDoc(1));
            }
            ObjectKind::Skeleton => {
                e.insert(ph2d_skeleton_ecs::Skeleton);
            }
        }
        let e = e.id();
        assert_eq!(kind_of(sim.world(), e), kind, "{kind:?}");
        if let Some(marker) = kind.marker() {
            let entry = reg
                .get_by_id(ph2d_ecs::scene::stable_type_id(marker))
                .unwrap_or_else(|| panic!("{marker} não está no registo"));
            assert!(
                matches!((entry.serialize)(sim.world(), e), Ok(Some(_))),
                "{kind:?}: o marcador {marker} não é o componente que o kind_of pergunta"
            );
        }
    }
}

/// **A imagem pintada é uma IMAGEM** — pintar é um modo dela (escolha 1 do dono, spec/06).
#[test]
fn a_painted_image_is_an_image() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::PaintedDoc(1),
        ))
        .id();
    assert_eq!(
        crate::component_attach::kind_of(sim.world(), e),
        ph2d_component_desc::ObjectKind::Image
    );
}
