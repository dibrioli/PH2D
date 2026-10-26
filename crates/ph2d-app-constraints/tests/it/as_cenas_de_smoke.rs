//! ⭐ **A sonda headless das cenas** — os números que a mensagem ao dono promete, medidos antes dela.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::smoke::{self, FOLGA_DA_PUPILA, ROUTERS};
use ph2d_constraints_ecs::evaluate;
use ph2d_ecs::{Name, SimWorld, Transform, world_transform};
use ph2d_preview_drive::PreviewDrive;

fn cena(n: u32) -> (SimWorld, PreviewDrive) {
    let mut sim = SimWorld::new();
    assert!(
        smoke::encenar(sim.world_mut(), n),
        "a cena {n} não respondeu"
    );
    let mut drive = PreviewDrive::default();
    evaluate(&mut sim, &mut drive);
    drive.settle();
    (sim, drive)
}

fn chamada(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena não tem «{nome}»"))
}

fn mundo(sim: &SimWorld, e: Entity) -> Transform {
    world_transform(sim.world(), e).expect("pose mundial")
}

#[test]
fn every_level_up_to_the_router_ceiling_answers() {
    let teto = ROUTERS[0].max_level;
    for n in 1..=teto {
        let mut sim = SimWorld::new();
        assert!(
            smoke::encenar(sim.world_mut(), n),
            "a cena {n} está abaixo do tecto e é muda"
        );
    }
    assert!(!smoke::encenar(SimWorld::new().world_mut(), teto + 1));
}

/// Cena 2: arrastar a bola por um círculo largo à volta do olho — a pupila nunca sai da folga e a seta
/// aponta sempre para a bola.
#[test]
fn the_pupil_never_leaves_the_eye_and_the_arrow_aims_at_the_ball() {
    let (mut sim, mut drive) = cena(2);
    let olho = chamada(&mut sim, "Olho");
    let bola = chamada(&mut sim, "Bola");
    let pupila = chamada(&mut sim, "Pupila");
    let seta = chamada(&mut sim, "Seta");
    let mut maior = 0.0_f64;
    for k in 0..72 {
        let a = f64::from(k) * std::f64::consts::TAU / 72.0;
        let r = if k % 2 == 0 { 3.0 } else { 0.3 };
        #[expect(clippy::cast_possible_truncation, reason = "o Transform é f32")]
        {
            let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
            t.translation.x = (r * a.cos()) as f32;
            t.translation.y = (r * a.sin()) as f32;
        }
        evaluate(&mut sim, &mut drive);
        drive.settle();
        let (o, p, b, s) = (
            mundo(&sim, olho),
            mundo(&sim, pupila),
            mundo(&sim, bola),
            mundo(&sim, seta),
        );
        let d = f64::from(p.translation.x - o.translation.x)
            .hypot(f64::from(p.translation.y - o.translation.y));
        maior = maior.max(d);
        if r < FOLGA_DA_PUPILA {
            assert!(
                (p.translation - b.translation).length() < 1e-5,
                "dentro da folga a pupila É a bola"
            );
        }
        let alvo = f64::from(b.translation.y - s.translation.y)
            .atan2(f64::from(b.translation.x - s.translation.x));
        let erro = ph2d_constraints::wrap_pi(f64::from(s.rotation) - alvo).abs();
        assert!(
            erro < 1e-5,
            "a seta erra a bola por {erro} rad no passo {k}"
        );
    }
    assert!(
        maior <= FOLGA_DA_PUPILA + 1e-5,
        "a pupila saiu do olho: {maior}"
    );
    assert!(
        maior >= FOLGA_DA_PUPILA - 1e-5,
        "a pupila nunca chegou à borda: {maior}"
    );
}

/// Cena 3: os dois ponteiros autorados a 170°.
#[test]
fn the_limited_pointer_stops_at_45_and_the_half_copy_lands_at_180() {
    let (mut sim, _) = cena(3);
    let limitado = chamada(&mut sim, "Ponteiro com limite");
    let meio = chamada(&mut sim, "Ponteiro a meio");
    let graus = |e| f64::from(mundo(&sim, e).rotation).to_degrees();
    assert!(
        (graus(limitado) - 45.0).abs() < 1e-4,
        "limite: {}",
        graus(limitado)
    );
    assert!(
        (graus(meio).abs() - 180.0).abs() < 1e-4,
        "meia cópia: {}",
        graus(meio)
    );
}

/// Cena 2, report do dono 25/10 (*«Inside e Surface estão na mesma posição»*): com a bola FORA do
/// olho os dois põem a pupila na borda (é a mesma resposta); com a bola DENTRO da folga, o Inside
/// deixa-a na bola e o Surface empurra-a até à borda — e o Outside também.
#[test]
fn inside_and_surface_differ_only_with_the_ball_inside_the_eye() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_ecs::canal_constraint::MODO;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let pupila_com = |modo: f32, bola_x: f32| {
        let (mut sim, mut drive) = cena(2);
        let bola = chamada(&mut sim, "Bola");
        let pupila = chamada(&mut sim, "Pupila");
        let id = sim.world().get::<Constraints>(pupila).unwrap().items[1].id;
        apply_all(&mut sim, &[(pupila.to_bits(), E::Campo(id, MODO, modo))]);
        let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
        t.translation.x = bola_x;
        t.translation.y = 0.0;
        evaluate(&mut sim, &mut drive);
        drive.settle();
        f64::from(mundo(&sim, pupila).translation.x)
    };
    let fora = 3.0;
    assert!((pupila_com(0.0, fora) - FOLGA_DA_PUPILA).abs() < 1e-5);
    assert!(
        (pupila_com(2.0, fora) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "fora, iguais"
    );
    let dentro = 0.2;
    assert!(
        (pupila_com(0.0, dentro) - f64::from(dentro)).abs() < 1e-5,
        "Inside move-a"
    );
    assert!(
        (pupila_com(2.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "Surface à borda"
    );
    assert!(
        (pupila_com(1.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5,
        "Outside à borda"
    );
}

/// Cena 2, report do dono 25/10 (*«Outside: a pupila fica sempre em cima da bola»*): o Outside é
/// «nunca mais PERTO que a distância» — com a bola fora do círculo a pupila segue-a (nada a
/// corrigir); com a bola DENTRO, a pupila vai à borda, pelo raio, em qualquer raio e direcção; a
/// meia força fica a meio caminho entre a bola e a borda.
#[test]
fn outside_pushes_the_pupil_to_the_edge_whenever_the_ball_is_inside() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_ecs::canal_constraint::MODO;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let (mut sim, mut drive) = cena(2);
    let bola = chamada(&mut sim, "Bola");
    let pupila = chamada(&mut sim, "Pupila");
    let ids: Vec<u32> = sim
        .world()
        .get::<Constraints>(pupila)
        .unwrap()
        .items
        .iter()
        .map(|i| i.id)
        .collect();
    apply_all(&mut sim, &[(pupila.to_bits(), E::Campo(ids[1], MODO, 1.0))]);
    let em = |sim: &mut SimWorld, drive: &mut PreviewDrive, x: f32, y: f32| {
        let mut t = sim.world_mut().get_mut::<Transform>(bola).unwrap();
        t.translation.x = x;
        t.translation.y = y;
        for _ in 0..3 {
            ph2d_app_constraints::quadro(sim, &ph2d_vec_scene::VecScene::new(), drive);
            drive.settle();
        }
        let p = mundo(sim, pupila).translation;
        (f64::from(p.x), f64::from(p.y))
    };
    for (x, y) in [(0.05, 0.0), (0.3, 0.2), (-0.1, -0.4), (0.0, 0.59)] {
        let (px, py) = em(&mut sim, &mut drive, x, y);
        assert!(
            (px.hypot(py) - FOLGA_DA_PUPILA).abs() < 1e-4,
            "bola em ({x}, {y}) ⇒ pupila a {}",
            px.hypot(py)
        );
    }
    let (px, py) = em(&mut sim, &mut drive, 2.0, 1.0);
    assert!(
        (px - 2.0).abs() < 1e-5 && (py - 1.0).abs() < 1e-5,
        "fora, segue a bola"
    );
    apply_all(&mut sim, &[(pupila.to_bits(), E::Forca(ids[1], 0.5))]);
    let (px, _) = em(&mut sim, &mut drive, 0.2, 0.0);
    assert!(
        (px - (0.2 + FOLGA_DA_PUPILA) / 2.0).abs() < 1e-4,
        "meia força ⇒ {px}"
    );
}

/// Cena 4: o braço com a IK na mão e um Copy Rotation ABAIXO — a mão roda com o Rodador (no Blender o
/// Copy Rotation some); descer a IK na lista põe a ponta da mão na bola.
#[test]
fn the_arm_scene_turns_the_hand_below_the_ik() {
    use ph2d_app_constraints::inspector::apply_all;
    use ph2d_app_constraints::smoke::OSSO;
    use ph2d_constraints_ecs::Constraints;
    use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
    let (mut sim, mut drive) = cena(4);
    let ombro = chamada(&mut sim, "Ombro");
    let mao = chamada(&mut sim, "Antebraço");
    let bola = chamada(&mut sim, "Bola");
    let rodador = chamada(&mut sim, "Rodador");
    let passo = |sim: &mut SimWorld, drive: &mut PreviewDrive| {
        for _ in 0..3 {
            ph2d_app_constraints::quadro(sim, &ph2d_vec_scene::VecScene::new(), drive);
            drive.settle();
        }
    };
    let longe = |a: Transform, b: Transform| {
        f64::from((a.translation.x - b.translation.x).hypot(a.translation.y - b.translation.y))
    };
    let confere = |sim: &SimWorld| {
        let (m, r) = (mundo(sim, mao), mundo(sim, rodador));
        assert!(
            ph2d_constraints::wrap_pi(f64::from(m.rotation - r.rotation)).abs() < 1e-5,
            "a mão roda com o Rodador: {} ≠ {}",
            m.rotation.to_degrees(),
            r.rotation.to_degrees()
        );
        assert!((longe(m, mundo(sim, ombro)) - f64::from(OSSO)).abs() < 1e-4);
        assert!(
            (longe(m, mundo(sim, bola)) - f64::from(OSSO)).abs() < 1e-4,
            "o cotovelo é o da IK"
        );
    };
    passo(&mut sim, &mut drive);
    confere(&sim);
    sim.world_mut()
        .get_mut::<Transform>(bola)
        .unwrap()
        .translation
        .x = -0.5;
    sim.world_mut()
        .get_mut::<Transform>(rodador)
        .unwrap()
        .rotation = 30f32.to_radians();
    passo(&mut sim, &mut drive);
    confere(&sim);
    let antes = (mundo(&sim, ombro), mundo(&sim, mao));
    passo(&mut sim, &mut drive);
    assert_eq!(
        (mundo(&sim, ombro), mundo(&sim, mao)),
        antes,
        "parado, não salta"
    );
    // Descer a IK: agora ela vence, e a ponta da mão toca a bola.
    let ik = sim.world().get::<Constraints>(mao).unwrap().items[0].id;
    apply_all(&mut sim, &[(mao.to_bits(), E::Mover(ik, 1))]);
    passo(&mut sim, &mut drive);
    let m = mundo(&sim, mao);
    let (s, c) = m.rotation.sin_cos();
    let b = mundo(&sim, bola).translation;
    let ponta = (
        m.translation.x + c * OSSO * m.scale.x,
        m.translation.y + s * OSSO * m.scale.x,
    );
    assert!(
        (ponta.0 - b.x).abs() < 1e-4 && (ponta.1 - b.y).abs() < 1e-4,
        "{ponta:?} ≠ {b:?}"
    );
}
