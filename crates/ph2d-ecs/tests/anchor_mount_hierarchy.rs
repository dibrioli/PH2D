//! **Montar numa âncora, medido pelas DUAS travessias** (ADR-0072 §2.6).
//!
//! O módulo [`ph2d_ecs::anchor_mount`] declara que o quadro de uma âncora entra na hierarquia por
//! uma lei só, chamada de dois sítios. Esta suíte é o que torna isso verdade em vez de escrito:
//! ela roda a propagação (o caminho do renderer, de cima para baixo) e a subida pela cadeia (o
//! caminho dos gizmos, do pick e da física) sobre a **mesma** árvore e exige o **mesmo** número.
//!
//! ⚠️ **É este o gate que impede o defeito caro deste desenho:** a espada desenhada na mão e
//! agarrada na origem do pai. Ele não é hipotético — o doc de `transform_inverse` regista a
//! família (`docs/Physics/BUGS_physics.md` #2, medida a um offset de pai inteiro), e a `line/Sprite`
//! já pagou a versão de overlay dela em 2026-08-22 (a cruz lia `GlobalTransform` do mundo errado e
//! ficava cravada na origem).

use ph2d_core::Vec2;
use ph2d_ecs::{
    AnchorMount, ChildOf, GlobalTransform, NamedAnchor, NamedAnchorList, PresentWorld, SimRef,
    SimWorld, Transform, TransformPropagationState, WorklistBuf, propagate_transforms_into_present,
    world_transform,
};

use bevy_ecs::entity::Entity;

/// A pose que a PROPAGAÇÃO publica para esta entidade.
fn propagated(sim: &mut SimWorld, e: Entity) -> GlobalTransform {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, &mut state, present_w, &mut worklist);
    });
    let mut q = present.world_mut().query::<(&SimRef, &GlobalTransform)>();
    let found = q
        .iter(present.world())
        .find(|(s, _)| s.0 == e)
        .map(|(_, g)| *g);
    found.expect("a entidade tem de ser visitada pela propagacao")
}

/// A pose que a SUBIDA pela cadeia devolve para esta entidade.
fn walked(sim: &SimWorld, e: Entity) -> GlobalTransform {
    GlobalTransform::from_transform(world_transform(sim.world(), e).expect("tem Transform"))
}

/// Um dono com uma âncora `muzzle` na pose dada.
fn owner(sim: &mut SimWorld, own: Transform, anchor: Transform) -> Entity {
    let mut list = NamedAnchorList::new();
    let mut a = NamedAnchor::socket("muzzle");
    a.transform = anchor;
    list.insert(a).unwrap();
    sim.world_mut().spawn((own, list)).id()
}

fn t(x: f32, y: f32) -> Transform {
    Transform {
        translation: Vec2::new(x, y),
        ..Transform::default()
    }
}

/// **O gate-mãe.** As duas travessias respondem o MESMO para um filho montado — e a árvore tem
/// rotação, escala e um neto, porque um caso só de translação passa mesmo quando a ordem da
/// composição está trocada.
#[test]
fn the_two_walks_agree_about_a_mounted_child() {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn(Transform {
            translation: Vec2::new(3.0, -1.0),
            rotation: 0.4,
            scale: Vec2::new(1.5, 1.5),
            ..Transform::default()
        })
        .id();
    let host = owner(
        &mut sim,
        Transform {
            translation: Vec2::new(0.0, 2.0),
            rotation: -0.2,
            scale: Vec2::new(2.0, 0.5),
            ..Transform::default()
        },
        Transform {
            translation: Vec2::new(1.0, 0.25),
            rotation: 0.9,
            scale: Vec2::new(0.5, 3.0),
            ..Transform::default()
        },
    );
    sim.world_mut().entity_mut(host).insert(ChildOf(root));
    let rider = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(0.7, -0.3),
                rotation: 0.15,
                ..Transform::default()
            },
            ChildOf(host),
            AnchorMount::new("muzzle"),
        ))
        .id();
    let grandchild = sim.world_mut().spawn((t(0.4, 0.4), ChildOf(rider))).id();

    for (label, e) in [("rider", rider), ("grandchild", grandchild)] {
        let a = propagated(&mut sim, e);
        let b = walked(&sim, e);
        assert_eq!(
            a.matrix, b.matrix,
            "{label}: a propagacao e a subida discordam — a espada desenha num sitio e o \
             ponteiro agarra-a noutro"
        );
    }
}

/// **Presença e ausência.** Sem o vínculo o filho fica no pai; com ele, na âncora. Um gate que
/// só medisse o caso montado passaria com o vínculo ignorado por completo.
#[test]
fn the_mount_is_what_moves_the_rider_and_nothing_else_does() {
    let mut sim = SimWorld::new();
    let host = owner(&mut sim, t(10.0, 0.0), t(0.0, 4.0));
    let free = sim.world_mut().spawn((t(0.0, 0.0), ChildOf(host))).id();
    let bound = sim
        .world_mut()
        .spawn((t(0.0, 0.0), ChildOf(host), AnchorMount::new("muzzle")))
        .id();

    assert_eq!(walked(&sim, free).translation(), Vec2::new(10.0, 0.0));
    assert_eq!(
        walked(&sim, bound).translation(),
        Vec2::new(10.0, 4.0),
        "o filho montado tem de partir da ANCORA, nao da origem do pai"
    );
    // E a propagação diz o mesmo.
    assert_eq!(
        propagated(&mut sim, bound).translation(),
        Vec2::new(10.0, 4.0)
    );
}

/// Um vínculo pendurado — nome que o pai não tem — comporta-se **byte a byte** como não haver
/// vínculo. É o que impede a espada de saltar para a origem do mundo quando a âncora é renomeada.
#[test]
fn a_dangling_mount_is_byte_identical_to_no_mount() {
    let mut sim = SimWorld::new();
    let host = owner(&mut sim, t(10.0, 0.0), t(0.0, 4.0));
    let free = sim.world_mut().spawn((t(1.0, 1.0), ChildOf(host))).id();
    let lost = sim
        .world_mut()
        .spawn((t(1.0, 1.0), ChildOf(host), AnchorMount::new("hand_r")))
        .id();
    assert_eq!(walked(&sim, lost).matrix, walked(&sim, free).matrix);
    assert_eq!(
        propagated(&mut sim, lost).matrix,
        propagated(&mut sim, free).matrix
    );
}

/// **A rotação da âncora vira quem monta** — é o que faz a bala sair na direção do cano, e o que
/// uma API que só devolvesse posição não conseguiria exprimir.
#[test]
fn the_anchor_rotation_turns_the_rider() {
    let mut sim = SimWorld::new();
    let host = owner(
        &mut sim,
        Transform::default(),
        Transform {
            translation: Vec2::new(1.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2,
            ..Transform::default()
        },
    );
    // O filho anda 2 m no SEU +X; sob a âncora rodada 90°, isso é +Y do mundo.
    let rider = sim
        .world_mut()
        .spawn((t(2.0, 0.0), ChildOf(host), AnchorMount::new("muzzle")))
        .id();
    let p = walked(&sim, rider).translation();
    assert!((p.x - 1.0).abs() < 1e-5, "{p:?}");
    assert!((p.y - 2.0).abs() < 1e-5, "{p:?}");
}

/// **Montagem sobre montagem** — quem monta pode ele próprio ser um dono com âncoras. É o caso
/// em que a ORDEM da dobra na subida tem de estar certa: trocar as duas linhas compila e põe a
/// âncora no espaço errado.
///
/// ⚠️ **O corpo tem de estar RODADO, e a primeira versão deste teste não estava.** Com poses só
/// de translação, `pai ∘ âncora` e `âncora ∘ pai` dão o mesmo ponto — as translações comutam —, e
/// a mutação que trocava as duas linhas passava por aqui **verde**. Foi o gate irmão
/// (`the_two_walks_agree_about_a_mounted_child`, que tem rotação e escala) que a apanhou. *Uma
/// fixtura que não contém o fenómeno mede silêncio*, e este teste era exatamente isso.
#[test]
fn a_mount_whose_host_is_itself_mounted_composes_in_order() {
    let mut sim = SimWorld::new();
    // A mão do personagem, a 4 m acima da origem dele — e o corpo RODADO, senão a ordem da
    // composição não é observável (ver a nota acima).
    let body = owner(
        &mut sim,
        Transform {
            translation: Vec2::new(10.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2,
            ..Transform::default()
        },
        t(0.0, 4.0),
    );
    // A arma monta na mão e tem, ela própria, uma âncora de boca 3 m à frente.
    let gun = {
        let mut list = NamedAnchorList::new();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = Vec2::new(3.0, 0.0);
        list.insert(a).unwrap();
        sim.world_mut()
            .spawn((
                Transform::default(),
                list,
                ChildOf(body),
                AnchorMount::new("muzzle"),
            ))
            .id()
    };
    let flash = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(gun),
            AnchorMount::new("muzzle"),
        ))
        .id();

    // A mão está 4 m no +Y LOCAL do corpo, que rodado 90° aponta para −X do mundo: 10 − 4 = 6.
    let g = walked(&sim, gun).translation();
    assert!((g.x - 6.0).abs() < 1e-5 && g.y.abs() < 1e-5, "{g:?}");
    // A boca está 3 m no +X local da arma, que herdou os 90°: +3 em Y do mundo.
    let f = walked(&sim, flash).translation();
    assert!(
        (f.x - 6.0).abs() < 1e-5 && (f.y - 3.0).abs() < 1e-5,
        "os dois quadros de ancora tem de compor na ordem raiz->folha, deu {f:?}"
    );
    assert_eq!(
        propagated(&mut sim, flash).matrix,
        walked(&sim, flash).matrix
    );
}

/// A escala do dono **chega** a quem monta: uma espada num personagem 2× é 2×.
#[test]
fn the_owners_scale_reaches_the_rider() {
    let mut sim = SimWorld::new();
    let host = owner(
        &mut sim,
        Transform {
            scale: Vec2::new(2.0, 2.0),
            ..Transform::default()
        },
        t(1.0, 0.0),
    );
    let rider = sim
        .world_mut()
        .spawn((t(1.0, 0.0), ChildOf(host), AnchorMount::new("muzzle")))
        .id();
    let w = walked(&sim, rider);
    // 1 m ate' a ancora + 1 m do filho, tudo a 2× ⇒ 4 m.
    assert_eq!(w.translation(), Vec2::new(4.0, 0.0));
}
