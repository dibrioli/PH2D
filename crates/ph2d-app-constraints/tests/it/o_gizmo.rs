//! ⭐ **O gizmo no palco** (plano §4): arrastar a ponta do arco ou a pega do raio escreve o campo do
//! cartão, e o campo do cartão move a peça — uma lei, dois gestos.

use ph2d_app_constraints::gizmo::{Gesto, Parte, Peca, Segurando, gesto, pecas};
use ph2d_app_constraints::inspector::{apply_all, info};
use ph2d_constraints::{Distance, Kind, Limit};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::LIMITE_MAX;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

fn vista() -> (Camera2d, WindowSize) {
    (Camera2d::new([0.0, 0.0], 10.0), WindowSize::new(1000, 1000))
}

fn em(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

fn arco(sim: &SimWorld, sel: u64) -> ([f32; 2], [[f32; 2]; 2]) {
    let (cam, win) = vista();
    pecas(sim.world(), Some(sel), &cam, win)
        .into_iter()
        .find_map(|p| match p {
            Peca::Arco { centro, pontas, .. } => Some((centro, [pontas[0].1, pontas[1].1])),
            _ => None,
        })
        .expect("o Limit Rotation não desenhou o arco")
}

/// O ângulo de TELA de `p` visto de `c`, comparável entre pontos da mesma vista.
fn rumo(c: [f32; 2], p: [f32; 2]) -> f32 {
    (p[1] - c[1]).atan2(p[0] - c[0])
}

/// ⭐⭐⭐ Premir a ponta do Max e arrastá-la para cima põe o Max a 90°; o campo a 30° volta a pôr a
/// ponta a 30°. Largar solta; um clique longe das pegas não é do gizmo.
#[test]
fn dragging_an_arc_edge_writes_the_card_field_and_the_field_moves_the_arc() {
    let (cam, win) = vista();
    let mut sim = SimWorld::new();
    let mut c = Constraints::default();
    let q = std::f64::consts::FRAC_PI_4;
    let id = c.push(
        Kind::Limit(Limit {
            rot: Some([-q, q]),
            ..Limit::default()
        }),
        None,
    );
    let e = sim.world_mut().spawn((StableId(1), em(0.0, 0.0), c)).id();
    let b = e.to_bits();
    let (centro, [_, max]) = arco(&sim, b);
    assert!(
        !gesto(
            &mut sim,
            Gesto::Premir,
            Some(b),
            (centro[0] + 300.0, centro[1]),
            None,
            &cam,
            win
        ),
        "longe das pegas"
    );
    assert!(gesto(
        &mut sim,
        Gesto::Premir,
        Some(b),
        (max[0], max[1]),
        None,
        &cam,
        win
    ));
    let seg = sim.world().resource::<Segurando>().0.unwrap();
    assert_eq!((seg.item, seg.parte), (id, Parte::Max));
    assert!(gesto(
        &mut sim,
        Gesto::Mover,
        Some(b),
        (0.0, 0.0),
        Some([0.0, 2.0]),
        &cam,
        win
    ));
    let i = info(&sim, b, 1).unwrap();
    assert!(
        (i.rows[0].numeros[1].valor - 90.0).abs() < 1e-3,
        "o Max ficou {}",
        i.rows[0].numeros[1].valor
    );
    assert!(
        (i.rows[0].numeros[0].valor + 45.0).abs() < 1e-3,
        "o Min mexeu"
    );
    assert!(gesto(
        &mut sim,
        Gesto::Largar,
        Some(b),
        (0.0, 0.0),
        None,
        &cam,
        win
    ));
    assert!(
        !gesto(
            &mut sim,
            Gesto::Largar,
            Some(b),
            (0.0, 0.0),
            None,
            &cam,
            win
        ),
        "largar duas vezes"
    );
    assert!(
        !gesto(
            &mut sim,
            Gesto::Mover,
            Some(b),
            (0.0, 0.0),
            Some([2.0, 0.0]),
            &cam,
            win
        ),
        "solto, mover não é do gizmo"
    );

    assert!(apply_all(&mut sim, &[(b, E::Campo(id, LIMITE_MAX, 30.0))]));
    let (centro, [_, max]) = arco(&sim, b);
    let alvo = cam.world_to_screen([30f32.to_radians().cos(), 30f32.to_radians().sin()], win);
    let esperado = rumo(centro, [alvo.0, alvo.1]);
    assert!(
        (rumo(centro, max) - esperado).abs() < 1e-3,
        "o campo não moveu a ponta"
    );
}

/// ⭐⭐ A pega do raio do Limit Distance senta no círculo, do lado OPOSTO ao dono (no limite o dono
/// está no círculo e a pega roubaria o clique dele); arrastá-la põe a
/// distância ao alvo.
/// ⭐⭐ Arrastar a ponta do Max de 170° para −170° ATRAVESSA ±180° pelo arco curto: o Max anda 20°
/// (a 190°), nunca uma volta para trás (−170°, que o Min ainda aparava a 100°).
///
/// **Mutação que deve sangrar:** tirar o `wrap_pi` do ângulo arrastado.
#[test]
fn dragging_an_arc_edge_across_180_takes_the_short_way() {
    let (cam, win) = vista();
    let mut sim = SimWorld::new();
    let mut c = Constraints::default();
    let id = c.push(
        Kind::Limit(Limit {
            rot: Some([100f64.to_radians(), 170f64.to_radians()]),
            ..Limit::default()
        }),
        None,
    );
    let e = sim.world_mut().spawn((StableId(1), em(0.0, 0.0), c)).id();
    let b = e.to_bits();
    let (_, [_, max]) = arco(&sim, b);
    assert!(gesto(
        &mut sim,
        Gesto::Premir,
        Some(b),
        (max[0], max[1]),
        None,
        &cam,
        win
    ));
    assert_eq!(
        sim.world()
            .resource::<Segurando>()
            .0
            .map(|s| (s.item, s.parte)),
        Some((id, Parte::Max))
    );
    let a = (-170f64).to_radians();
    assert!(gesto(
        &mut sim,
        Gesto::Mover,
        Some(b),
        (0.0, 0.0),
        Some([2.0 * a.cos(), 2.0 * a.sin()]),
        &cam,
        win
    ));
    let i = info(&sim, b, 1).unwrap();
    assert!(
        (i.rows[0].numeros[1].valor - 190.0).abs() < 1e-3,
        "o Max ficou {}",
        i.rows[0].numeros[1].valor
    );
    assert!(
        (i.rows[0].numeros[0].valor - 100.0).abs() < 1e-3,
        "o Min mexeu"
    );
}

#[test]
fn dragging_the_radius_handle_writes_the_distance() {
    let (cam, win) = vista();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((StableId(1), em(0.0, 0.0)));
    let mut c = Constraints::default();
    c.push(
        Kind::Distance(Distance {
            distance: 2.0,
            ..Distance::default()
        }),
        Some(StableId(1)),
    );
    let e = sim.world_mut().spawn((StableId(2), em(5.0, 0.0), c)).id();
    let b = e.to_bits();
    let pega = pecas(sim.world(), Some(b), &cam, win)
        .into_iter()
        .find_map(|p| match p {
            Peca::Circulo { pega, .. } => Some(pega.1),
            _ => None,
        })
        .expect("o círculo não foi desenhado");
    let esperado = cam.world_to_screen([-2.0, 0.0], win);
    assert!(
        (pega[0] - esperado.0).hypot(pega[1] - esperado.1) < 1e-2,
        "a pega não está no raio"
    );
    assert!(gesto(
        &mut sim,
        Gesto::Premir,
        Some(b),
        (pega[0], pega[1]),
        None,
        &cam,
        win
    ));
    gesto(
        &mut sim,
        Gesto::Mover,
        Some(b),
        (0.0, 0.0),
        Some([0.0, 3.0]),
        &cam,
        win,
    );
    let i = info(&sim, b, 1).unwrap();
    assert!((i.rows[0].numeros[0].valor - 3.0).abs() < 1e-4);
    assert!(
        pecas(sim.world(), None, &cam, win).is_empty(),
        "sem escolha, sem gizmo"
    );
}
