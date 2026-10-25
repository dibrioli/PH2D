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
    assert!((pupila_com(2.0, fora) - FOLGA_DA_PUPILA).abs() < 1e-5, "fora, iguais");
    let dentro = 0.2;
    assert!((pupila_com(0.0, dentro) - f64::from(dentro)).abs() < 1e-5, "Inside move-a");
    assert!((pupila_com(2.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5, "Surface à borda");
    assert!((pupila_com(1.0, dentro) - FOLGA_DA_PUPILA).abs() < 1e-5, "Outside à borda");
}
