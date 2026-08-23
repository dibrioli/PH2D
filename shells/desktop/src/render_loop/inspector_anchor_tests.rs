//! **Os gates de [`super::inspector_anchor`]** — irmão por CAP de LOC (HR-18, 600 no shell).
//!
//! ⚠️ **Corte mecânico, conteúdo verbatim.** O módulo de testes saiu inteiro quando os
//! quatro pedidos do Enio de 2026-08-23 levaram o ficheiro acima do teto. A regra da casa é
//! cortar para o IRMÃO, nunca declarar exceção — e o idioma é o do `anchor_gizmo_tests.rs`.

use super::*;

/// **(pedido 1 e 4) Montar POUSA na âncora, e «—» NÃO.**
///
/// ⚠️ As duas metades no mesmo teste porque é a assimetria que importa: escolher uma âncora
/// põe o objeto nela (Enio, 2026-08-23), mas **desmontar é largar** — atirar o objeto para a
/// origem do pai sem ninguém pedir seria uma teleportação surpresa.
#[test]
fn choosing_an_anchor_lands_the_object_on_it_and_unmounting_leaves_it_alone() {
    use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
    use ph2d_ecs::{AnchorMount, ChildOf, NamedAnchorList, Transform};

    let mut registry = ComponentRegistry::new();
    register_ecs_components(&mut registry);

    let mut sim = SimWorld::new();
    let mut list = NamedAnchorList::new();
    list.insert(NamedAnchor::socket("hand_r")).unwrap();
    let host = sim.world_mut().spawn((Transform::default(), list)).id();
    let displaced = Transform {
        translation: ph2d_core::Vec2::new(0.3, -0.2),
        rotation: 0.5,
        ..Transform::default()
    };
    let rider = sim.world_mut().spawn((displaced, ChildOf(host))).id();

    // (a) escolher a âncora ⇒ pousa.
    let queue = EditorCommandQueue::new();
    apply_anchor_edit(
        &sim,
        rider.to_bits(),
        &AnchorFieldEdit::Mount(Some("hand_r".into())),
        &queue,
        &registry,
        100.0,
    );
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("o commit aplica");
    let t = *sim.world().get::<Transform>(rider).expect("tem pose");
    assert_eq!(
        t.translation,
        ph2d_core::Vec2::ZERO,
        "escolher a ancora tem de pousar o objeto nela"
    );
    assert_eq!(
        t.rotation, 0.5,
        "o snap zera a POSICAO e mais nada — o angulo da espada na mao e' do artista"
    );
    assert_eq!(
        sim.world()
            .get::<AnchorMount>(rider)
            .map(|m| m.anchor.as_str()),
        Some("hand_r")
    );

    // (b) desmontar ⇒ fica onde está. Mexe-se o objeto primeiro, para o zero não mentir.
    sim.world_mut().entity_mut(rider).insert(Transform {
        translation: ph2d_core::Vec2::new(1.0, 2.0),
        ..displaced
    });
    let queue = EditorCommandQueue::new();
    apply_anchor_edit(
        &sim,
        rider.to_bits(),
        &AnchorFieldEdit::Mount(None),
        &queue,
        &registry,
        100.0,
    );
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("o commit aplica");
    assert_eq!(
        sim.world().get::<Transform>(rider).unwrap().translation,
        ph2d_core::Vec2::new(1.0, 2.0),
        "largar nao pode teleportar"
    );
}

/// **(pedido 4) «Reset to Anchor» faz o mesmo, sozinho** — e o snapshot passa a dizer que já
/// não há deslocamento, que é o que faz o botão desaparecer.
#[test]
fn the_reset_button_lands_the_object_and_the_snapshot_agrees() {
    use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
    use ph2d_ecs::{AnchorMount, ChildOf, NamedAnchorList, Transform};

    let mut registry = ComponentRegistry::new();
    register_ecs_components(&mut registry);
    let mut sim = SimWorld::new();
    let mut list = NamedAnchorList::new();
    list.insert(NamedAnchor::socket("hand_r")).unwrap();
    let host = sim.world_mut().spawn((Transform::default(), list)).id();
    let rider = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(0.12, -0.04),
                ..Transform::default()
            },
            ChildOf(host),
            AnchorMount::new("hand_r"),
        ))
        .id();

    let before =
        build_anchor_info(sim.world(), rider.to_bits(), &[rider.to_bits()], 1, 100.0).unwrap();
    assert_eq!(before.mount_offset, [12.0, -4.0], "o deslocamento e' em px");
    assert!(before.is_off_anchor(), "o botao tem de existir aqui");

    let queue = EditorCommandQueue::new();
    apply_anchor_edit(
        &sim,
        rider.to_bits(),
        &AnchorFieldEdit::SnapToAnchor,
        &queue,
        &registry,
        100.0,
    );
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("o commit aplica");

    let after =
        build_anchor_info(sim.world(), rider.to_bits(), &[rider.to_bits()], 1, 100.0).unwrap();
    assert_eq!(after.mount_offset, [0.0, 0.0]);
    assert!(
        !after.is_off_anchor(),
        "o botao tem de desaparecer depois de fazer o seu trabalho"
    );
}

/// **(pedido 3) As duas caixas são independentes** — ligar uma não repõe a outra.
///
/// ⚠️ É a lei do ler-modificar-escrever, a mesma da lista de âncoras. Sem ela, marcar
/// «runtime» apagaria «always show» em silêncio, e o artista culparia o clique errado.
#[test]
fn the_two_visibility_boxes_do_not_erase_each_other() {
    use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
    use ph2d_ecs::{AnchorVisibility, NamedAnchorList, Transform};

    let mut registry = ComponentRegistry::new();
    register_ecs_components(&mut registry);
    let mut sim = SimWorld::new();
    let mut list = NamedAnchorList::new();
    list.insert(NamedAnchor::socket("muzzle")).unwrap();
    let host = sim.world_mut().spawn((Transform::default(), list)).id();

    for edit in [
        AnchorFieldEdit::VisibilityInEditor(true),
        AnchorFieldEdit::VisibilityAtRuntime(true),
    ] {
        let queue = EditorCommandQueue::new();
        apply_anchor_edit(&sim, host.to_bits(), &edit, &queue, &registry, 100.0);
        apply_editor_commands(sim.world_mut(), &queue, &registry).expect("o commit aplica");
    }
    assert_eq!(
        sim.world().get::<AnchorVisibility>(host).copied(),
        Some(AnchorVisibility {
            in_editor: true,
            at_runtime: true
        }),
        "a segunda caixa apagou a primeira"
    );

    // E desligar UMA deixa a outra.
    let queue = EditorCommandQueue::new();
    apply_anchor_edit(
        &sim,
        host.to_bits(),
        &AnchorFieldEdit::VisibilityInEditor(false),
        &queue,
        &registry,
        100.0,
    );
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("o commit aplica");
    assert_eq!(
        sim.world().get::<AnchorVisibility>(host).copied(),
        Some(AnchorVisibility {
            in_editor: false,
            at_runtime: true
        })
    );
}

/// px ↔ m fecha o círculo: o que o artista escreve é o que ele volta a ler.
#[test]
fn the_position_round_trips_through_pixels() {
    let mut a = NamedAnchor::socket("m");
    apply_field(&mut a, &AnchorFieldEdit::Pos(0, 0, 28.0), 100.0);
    assert!((a.transform.translation.x - 0.28).abs() < 1e-6);
    // O caminho de volta é o do snapshot.
    assert!((a.transform.translation.x * 100.0 - 28.0).abs() < 1e-4);
}

/// **O snapshot conta quem monta em cada âncora, e o pai que o objeto tem.**
///
/// ⚠️ As duas metades da §12 saem do mesmo `build_anchor_info`, e este gate mede-as juntas
/// porque é assim que elas se contradizem: um snapshot que contasse os passageiros de todos os
/// filhos (em vez dos que montam NAQUELA âncora) passaria num teste com uma âncora só.
#[test]
fn the_snapshot_counts_the_riders_of_each_anchor_and_finds_the_parent() {
    use ph2d_ecs::{AnchorMount, ChildOf, Transform};

    let mut w = ph2d_ecs::World::new();
    // O AVÔ, com uma âncora — é o que o `parent_anchors` do boneco tem de encontrar.
    let mut gl = NamedAnchorList::new();
    gl.insert(NamedAnchor::socket("world_slot")).unwrap();
    let grandparent = w.spawn((Transform::IDENTITY, gl)).id();

    // O BONECO, com duas âncoras, montado no avô.
    let mut list = NamedAnchorList::new();
    list.insert(NamedAnchor::socket("hand_r")).unwrap();
    list.insert(NamedAnchor::socket("head")).unwrap();
    let body = w
        .spawn((
            Transform::IDENTITY,
            list,
            ChildOf(grandparent),
            AnchorMount::new("world_slot"),
        ))
        .id();

    // Dois na mão, nenhum na cabeça, e um filho que não monta em nada.
    for m in ["hand_r", "hand_r"] {
        w.spawn((Transform::IDENTITY, ChildOf(body), AnchorMount::new(m)));
    }
    w.spawn((Transform::IDENTITY, ChildOf(body)));
    // E um que monta num nome que este boneco não tem — não pode contar para ninguém.
    w.spawn((
        Transform::IDENTITY,
        ChildOf(body),
        AnchorMount::new("ghost"),
    ));

    let info = build_anchor_info(&w, body.to_bits(), &[body.to_bits()], 1, 100.0)
        .expect("o boneco e' inspecionavel");
    assert_eq!(info.rows[0].name, "hand_r");
    assert_eq!(info.rows[0].riders, 2, "os dois na mao");
    assert_eq!(info.rows[1].riders, 0, "ninguem na cabeca");
    assert_eq!(
        info.parent_anchors,
        vec!["world_slot".to_string()],
        "o seletor tem de oferecer as ancoras do AVO, que e' o pai deste boneco"
    );
    assert_eq!(info.mount.as_deref(), Some("world_slot"));
    assert!(!info.mount_dangling());
}

/// A rotação viaja em radianos e é autorada em graus.
#[test]
fn the_rotation_is_authored_in_degrees_and_stored_in_radians() {
    let mut a = NamedAnchor::socket("m");
    apply_field(&mut a, &AnchorFieldEdit::Rot(0, 90.0), 100.0);
    assert!((a.transform.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
}

/// ⚠️ Ligar a área semeia um retângulo **visível**. Um de área zero seria indistinguível de
/// não ter ligado nada, e o artista ligaria a caixa e não veria mudança nenhuma.
#[test]
fn switching_bounds_on_seeds_a_visible_rect() {
    let mut a = NamedAnchor::socket("m");
    apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, true), 100.0);
    let b = a.bounds.expect("a area tinha de nascer");
    assert!(b[2] > 0.0 && b[3] > 0.0, "area nula: invisivel");
    assert_eq!(a.kind(), ph2d_ecs::AnchorKind::Slice);
}

/// Editar um campo da área preserva os outros três.
#[test]
fn editing_one_bounds_field_preserves_its_siblings() {
    let mut a = NamedAnchor::socket("m");
    a.set_bounds(Some([1.0, 2.0, 3.0, 4.0]));
    apply_field(&mut a, &AnchorFieldEdit::Bounds(0, 2, 9.0), 100.0);
    assert_eq!(a.bounds, Some([1.0, 2.0, 9.0, 4.0]));
}

/// Desligar a área leva o miolo — a mesma lei do componente.
#[test]
fn switching_bounds_off_takes_the_centre_with_it() {
    let mut a = NamedAnchor::socket("m");
    apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, true), 100.0);
    apply_field(&mut a, &AnchorFieldEdit::CenterOn(0, true), 100.0);
    assert_eq!(a.kind(), ph2d_ecs::AnchorKind::NineSliceRegion);
    apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, false), 100.0);
    assert_eq!(a.center, None);
    assert_eq!(a.kind(), ph2d_ecs::AnchorKind::Socket);
}
