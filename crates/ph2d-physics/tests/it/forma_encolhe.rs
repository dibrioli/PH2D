//! **Trocar a forma de um corpo dinâmico POUSADO não lhe mete velocidade** (smoke do dono 1 da cena
//! 26 do Pixel, 09/11: «pulinhos»). A forma encolhe por cima com o fundo no MESMO sítio — a key da
//! slice do herói que agacha. O `retune_body` troca a forma no lugar; o corpo tem de ficar onde está.
//! Ver `docs/PixelArt/P14_plano.md` «Smoke do dono 1 — pulinhos».

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// 1 px = 1 cm (a régua de 100 px/m da cena).
const PX: f32 = 0.01;

fn caixa(meia_h_px: f32, off_y_px: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 2.0 * PX,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 4.0 * PX,
            half_y: meia_h_px * PX,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: true,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        shape_pose: [0.0; 3],
        collide: [0; 2],
        offset: [0.0, off_y_px * PX],
    }
}

fn chao() -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Fixed,
        y: -0.05,
        shape: ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 0.05,
        },
        lock_rotation: false,
        offset: [0.0, 0.0],
        ..caixa(7.0, 6.0)
    }
}

/// Pousa a caixa 8×14 px (fundo 1 px abaixo da origem), troca-a por `nova` e devolve o pior desvio
/// da altura (px) e a pior |vy| (m/s) nos 60 passos seguintes; `y0` é a altura pousada (px).
fn troca(nova: BodyDesc) -> (f32, f32, f32) {
    let mut w = PhysicsWorld::new();
    w.spawn_body(chao());
    let h = w.spawn_body(caixa(7.0, 6.0));
    for _ in 0..180 {
        w.step();
    }
    let y = |w: &PhysicsWorld| w.body_pose(h).expect("vivo").translation.y / PX;
    let y0 = y(&w);
    assert!(w.retune_body(h, &nova), "reafina no lugar");
    let (mut desvio, mut vy) = (0.0_f32, 0.0_f32);
    for _ in 0..60 {
        w.step();
        desvio = desvio.max((y(&w) - y0).abs());
        vy = vy.max(w.body_velocity(h).expect("vivo")[1].abs());
    }
    (y0, desvio, vy)
}

#[test]
fn trocar_a_forma_de_um_corpo_pousado_nao_o_faz_pular() {
    // Encolhe por cima (14 → 10 px), o fundo no mesmo sítio (6 − 7 = 4 − 5 = −1 px).
    let (y0, desvio, vy) = troca(caixa(5.0, 4.0));
    eprintln!("encolhe: pousada em {y0:.3} px · pior desvio {desvio:.3} px · |vy| máx {vy:.4} m/s");
    assert!((y0 - 1.0).abs() < 0.05, "pousa pelo fundo: {y0:.3} px");
    assert!(
        desvio < 0.05,
        "a forma muda no lugar, sem pulo: {desvio:.3} px"
    );
    assert!(vy < 0.01, "sem velocidade injetada: {vy:.4} m/s");
    // E cresce de volta (10 → 14 px), o fundo no mesmo sítio.
    let mut w = PhysicsWorld::new();
    w.spawn_body(chao());
    let h = w.spawn_body(caixa(5.0, 4.0));
    for _ in 0..180 {
        w.step();
    }
    let y0 = w.body_pose(h).unwrap().translation.y / PX;
    assert!(w.retune_body(h, &caixa(7.0, 6.0)));
    let mut pior = 0.0_f32;
    for _ in 0..60 {
        w.step();
        pior = pior.max((w.body_pose(h).unwrap().translation.y / PX - y0).abs());
    }
    eprintln!("cresce: pior desvio {pior:.3} px");
    assert!(pior < 0.05, "crescer também no lugar: {pior:.3} px");
}
