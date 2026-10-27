//! Dois motores sobre a pose da MESMA entidade (o osso inteligente e a pilha de constraints): a
//! escrita de um não é «outra mão» para o outro, e o documento é o autorado de baixo.

use ph2d_ecs::{SimWorld, Transform};

use crate::{Driven, Driver, PreviewDrive};

fn em(x: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t
}

fn escreve(sim: &mut SimWorld, e: ph2d_ecs::Entity, t: Transform) {
    *sim.world_mut().get_mut::<Transform>(e).unwrap() = t;
}

fn agora(sim: &SimWorld, e: ph2d_ecs::Entity) -> Transform {
    *sim.world().get::<Transform>(e).unwrap()
}

/// Um quadro: o de baixo leva o autorado a `baixo`, o de cima leva isso a `cima`.
fn quadro(sim: &mut SimWorld, pd: &mut PreviewDrive, e: ph2d_ecs::Entity, baixo: f32, cima: f32) {
    let antes = agora(sim, e);
    if antes != em(baixo) {
        escreve(sim, e, em(baixo));
        pd.driven(e, Driven::SolverPose(antes), Driven::SolverPose(em(baixo)));
    } else {
        pd.still_driving(e, Driver::SolverPose);
    }
    let meio = agora(sim, e);
    escreve(sim, e, em(cima));
    pd.driven(
        e,
        Driven::ConstraintPose(meio),
        Driven::ConstraintPose(em(cima)),
    );
    pd.settle();
}

#[test]
fn two_motors_on_one_entity_keep_the_authored_pose() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(em(1.0)).id();
    let mut pd = PreviewDrive::default();
    for _ in 0..3 {
        quadro(&mut sim, &mut pd, e, 2.0, 3.0);
    }
    // O de baixo muda de valor: a camada de cima segue-o.
    quadro(&mut sim, &mut pd, e, 5.0, 6.0);
    assert_eq!(
        pd.authored(e.to_bits(), Driver::SolverPose),
        Some(Driven::SolverPose(em(1.0)))
    );
    assert_eq!(
        pd.authored(e.to_bits(), Driver::ConstraintPose),
        Some(Driven::ConstraintPose(em(5.0)))
    );
    let vivo = pd.substitute(&mut sim);
    assert_eq!(
        agora(&sim, e),
        em(1.0),
        "gravar a meio do play grava o AUTORADO"
    );
    PreviewDrive::restore(&mut sim, &vivo);
    assert_eq!(agora(&sim, e), em(6.0));
}

#[test]
fn an_artists_edit_under_two_motors_is_still_the_document() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(em(1.0)).id();
    let mut pd = PreviewDrive::default();
    quadro(&mut sim, &mut pd, e, 2.0, 3.0);
    escreve(&mut sim, e, em(9.0));
    quadro(&mut sim, &mut pd, e, 2.0, 3.0);
    assert_eq!(
        pd.authored(e.to_bits(), Driver::SolverPose),
        Some(Driven::SolverPose(em(9.0)))
    );
}

/// A consulta por entidade começa no `Driver::PRIMEIRO`: se ele não for o menor, os motores abaixo
/// dele somem das respostas. O `match` exaustivo obriga quem acrescenta um motor a passar por aqui.
#[test]
fn the_first_driver_is_the_least_and_per_entity_queries_see_every_driver() {
    use Driver as D;
    let todos = [
        D::SpriteAnim,
        D::SolverPose,
        D::PrefabStage,
        D::SpriteAlpha,
        D::MorphT,
        D::MorphPair,
        D::JointParams,
        D::SignalVisibility,
        D::ScriptPose,
        D::CanvasPose,
        D::ParallaxPose,
        D::TweenPose,
        D::TweenTint,
        D::SpriteCor,
        D::PontaNoMundo,
        D::Roldana,
        D::ConstraintPose,
        D::ColaPose,
    ];
    let exaustivo = |d: Driver| match d {
        D::SpriteAnim
        | D::SolverPose
        | D::PrefabStage
        | D::SpriteAlpha
        | D::MorphT
        | D::MorphPair
        | D::JointParams
        | D::SignalVisibility
        | D::ScriptPose
        | D::CanvasPose
        | D::ParallaxPose
        | D::TweenPose
        | D::TweenTint
        | D::SpriteCor
        | D::PontaNoMundo
        | D::Roldana
        | D::ConstraintPose
        | D::ColaPose => (),
    };
    todos.iter().copied().for_each(exaustivo);
    assert!(todos.iter().all(|&d| Driver::PRIMEIRO <= d));

    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(em(0.0)).id();
    let vizinha = sim.world_mut().spawn(em(0.0)).id();
    let mut pd = PreviewDrive::default();
    pd.driven(
        vizinha,
        Driven::SolverPose(em(0.0)),
        Driven::SolverPose(em(1.0)),
    );
    assert!(!pd.drives(e.to_bits()));
    pd.driven(e, Driven::SolverPose(em(0.0)), Driven::SolverPose(em(1.0)));
    assert!(pd.drives(e.to_bits()));
    assert!(pd.drives_other_than(e.to_bits(), Driver::ColaPose));
    assert!(!pd.drives_other_than(e.to_bits(), Driver::SolverPose));
    pd.driven(e, Driven::ColaPose(em(1.0)), Driven::ColaPose(em(2.0)));
    assert!(pd.drives_other_than(e.to_bits(), Driver::SolverPose));
    assert_eq!(
        pd.drivers_of(e.to_bits()),
        vec![Driver::SolverPose, Driver::ColaPose]
    );
}
