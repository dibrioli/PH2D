//! **W2a do Outline (plano f2): o motor aceita uma forma de N peças convexas.**
//!
//! `register_outline` guarda as peças (já em metros locais) por chave de CONTEÚDO; um
//! `ShapeDesc::Outline` com essa chave vira UM collider compound de `convex_hull` por peça.
//! Chave ausente ou sem peça válida → a caixa `half_*`, nunca pânico.

use ph2d_physics::{BodyDesc, OutlineKey, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

fn desc(shape: ShapeDesc, x: f32, y: f32, body_type: RigidBodyType, density: f32) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
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
        shape_off: false,
        offset: [0.0, 0.0],
    }
}

/// Um retângulo `[x0, x1] × [y0, y1]` como peça convexa.
fn caixa(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<[f32; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

/// Um U: fundo `y ∈ [-0,75, -0,55]` e dois braços até `y = 0,75`, `|x| ≤ 1`; vão `|x| < 0,8`.
fn pecas_u() -> Vec<Vec<[f32; 2]>> {
    vec![
        caixa(-1.0, -0.75, 1.0, -0.55),
        caixa(-1.0, -0.55, -0.8, 0.75),
        caixa(0.8, -0.55, 1.0, 0.75),
    ]
}

/// Um Π de duas pernas (base em `y = -0,5`) dentro de uma imagem maior (`half = 0,6 × 0,8`):
/// a caixa da imagem pousaria 0,3 m mais alto que as peças.
fn pecas_pi() -> Vec<Vec<[f32; 2]>> {
    vec![
        caixa(-0.6, -0.5, -0.4, 0.3),
        caixa(0.4, -0.5, 0.6, 0.3),
        caixa(-0.6, 0.3, 0.6, 0.5),
    ]
}
const PI_HALF: (f32, f32) = (0.6, 0.8);
/// Área do Π: duas pernas 0,2 × 0,8 e a barra 1,2 × 0,2.
const PI_AREA: f32 = 2.0 * 0.2 * 0.8 + 1.2 * 0.2;

fn outline(half: (f32, f32), key: OutlineKey) -> ShapeDesc {
    ShapeDesc::Outline {
        half_x: half.0,
        half_y: half.1,
        key,
    }
}

/// Chão estático com o topo em `y = 0`.
fn chao(w: &mut PhysicsWorld) {
    w.spawn_body(desc(
        ShapeDesc::Cuboid {
            half_x: 10.0,
            half_y: 0.5,
        },
        0.0,
        -0.5,
        RigidBodyType::Fixed,
        1.0,
    ));
}

fn pose(w: &PhysicsWorld, h: RigidBodyHandle) -> [f32; 3] {
    let b = w.bodies().get(h).expect("corpo vivo");
    [b.translation().x, b.translation().y, b.rotation().angle()]
}

fn bits(p: [f32; 3]) -> [u32; 3] {
    p.map(f32::to_bits)
}

fn corre(w: &mut PhysicsWorld, n: u32) {
    for _ in 0..n {
        w.step();
    }
}

#[test]
fn um_u_de_pecas_convexas_e_concavo_de_verdade() {
    let mut w = PhysicsWorld::new();
    let key = w.register_outline(pecas_u());
    w.spawn_body(desc(
        outline((1.0, 0.75), key),
        0.0,
        0.0,
        RigidBodyType::Fixed,
        1.0,
    ));
    let r = 0.2;
    let bola = w.spawn_body(desc(
        ShapeDesc::Ball { radius: r },
        0.0,
        2.0,
        RigidBodyType::Dynamic,
        1.0,
    ));
    corre(&mut w, 240);
    let [x, y, _] = pose(&w, bola);
    eprintln!(
        "bola no U: x {x:.4} y {y:.4} (fundo -0,55 + r = {:.2}; caixa daria {:.2})",
        -0.55 + r,
        0.75 + r
    );
    assert!(x.abs() < 0.8, "a bola saiu do vão: x {x}");
    assert!(
        y < 0.75 - r,
        "a bola ficou no topo dos braços: y {y} — o U virou caixa"
    );
    assert!(y > -0.55, "a bola atravessou o fundo do U: y {y}");
    assert!(
        (y - (-0.55 + r)).abs() < 0.01,
        "a bola não pousa no fundo: y {y}"
    );
}

#[test]
fn um_corpo_de_pecas_pousa_com_a_base_das_pecas_no_chao() {
    let mut w = PhysicsWorld::new();
    chao(&mut w);
    let key = w.register_outline(pecas_pi());
    let h = w.spawn_body(desc(
        outline(PI_HALF, key),
        0.0,
        0.6,
        RigidBodyType::Dynamic,
        1.0,
    ));
    corre(&mut w, 180);
    let [x, y, a] = pose(&w, h);
    let base = y - 0.5;
    eprintln!(
        "Π pousado: x {x:.4} y {y:.4} ângulo {a:.5} → base das peças {base:.4} (caixa da imagem daria {:.2})",
        PI_HALF.1
    );
    assert!(
        base.abs() < 0.01,
        "a base das peças não está no chão: {base}"
    );
    assert!(a.abs() < 0.01, "o Π tombou: {a}");
}

#[test]
fn a_massa_e_densidade_vezes_a_area_das_pecas() {
    let mut w = PhysicsWorld::new();
    let key = w.register_outline(pecas_pi());
    let densidade = 2.0;
    let h = w.spawn_body(desc(
        outline(PI_HALF, key),
        0.0,
        0.0,
        RigidBodyType::Dynamic,
        densidade,
    ));
    let m = w.body_mass(h);
    let esperado = densidade * PI_AREA;
    eprintln!(
        "massa do Π: {m:.5} kg, esperado {esperado:.5} (caixa da imagem: {:.5})",
        densidade * 4.0 * PI_HALF.0 * PI_HALF.1
    );
    assert!((m - esperado).abs() < 1e-3, "massa {m} ≠ {esperado}");
}

#[test]
fn o_rewind_repete_a_pose_e_a_chave_nao_sai() {
    let mut w = PhysicsWorld::new();
    chao(&mut w);
    let key = w.register_outline(pecas_pi());
    let mut d = desc(outline(PI_HALF, key), 0.0, 1.0, RigidBodyType::Dynamic, 2.0);
    d.angvel = 3.0;
    let h = w.spawn_body(d);
    let cp = w.checkpoint();
    corre(&mut w, 60);
    let primeira = pose(&w, h);
    w.restore(&cp);
    corre(&mut w, 60);
    let segunda = pose(&w, h);
    eprintln!("rewind: 1.ª {primeira:?} · 2.ª {segunda:?}");
    assert_eq!(bits(primeira), bits(segunda), "o rewind não repete a pose");

    // Outras formas registadas depois não tiram a chave: o re-spawn pelo mesmo `BodyDesc` acha as peças.
    w.register_outline(vec![caixa(0.0, 0.0, 1.0, 1.0)]);
    w.register_outline(vec![caixa(-1.0, -1.0, 0.0, 0.0), caixa(0.0, 0.0, 2.0, 0.5)]);
    w.remove_body(h);
    let mut d2 = d;
    d2.x = 5.0;
    let h2 = w.spawn_body(d2);
    let m = w.body_mass(h2);
    eprintln!("re-spawn: massa {m:.5}, esperado {:.5}", 2.0 * PI_AREA);
    assert!(
        (m - 2.0 * PI_AREA).abs() < 1e-3,
        "o re-spawn não achou as peças: massa {m}"
    );
}

/// Pousa um corpo de `shape` num chão e devolve (pose, massa).
fn pousa(shape: ShapeDesc, registos: &[Vec<Vec<[f32; 2]>>]) -> ([f32; 3], f32) {
    let mut w = PhysicsWorld::new();
    for r in registos {
        w.register_outline(r.clone());
    }
    chao(&mut w);
    let h = w.spawn_body(desc(shape, 0.0, 0.7, RigidBodyType::Dynamic, 1.0));
    corre(&mut w, 120);
    (pose(&w, h), w.body_mass(h))
}

#[test]
fn chave_ausente_ou_sem_peca_valida_e_a_caixa() {
    let half = (0.5, 0.4);
    let (caixa_pose, caixa_massa) = pousa(
        ShapeDesc::Cuboid {
            half_x: half.0,
            half_y: half.1,
        },
        &[],
    );
    let (ausente_pose, ausente_massa) = pousa(outline(half, OutlineKey(12345)), &[]);
    // Peças que nenhum hull aceita: pontos coincidentes, dois pontos, colineares, não finitos, vazia.
    let degeneradas = vec![
        vec![[0.1, 0.1]; 3],
        vec![[0.0, 0.0], [1.0, 0.0]],
        vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]],
        vec![[0.0, 0.0], [f32::NAN, 0.0], [1.0, 1.0]],
        vec![],
    ];
    let mut w = PhysicsWorld::new();
    let key_deg = w.register_outline(degeneradas.clone());
    let (deg_pose, deg_massa) = pousa(outline(half, key_deg), &[degeneradas]);
    eprintln!(
        "caixa {caixa_pose:?} {caixa_massa:.4} · ausente {ausente_pose:?} {ausente_massa:.4} · degenerada {deg_pose:?} {deg_massa:.4}"
    );
    assert_eq!(
        bits(ausente_pose),
        bits(caixa_pose),
        "chave ausente ≠ caixa"
    );
    assert_eq!(ausente_massa.to_bits(), caixa_massa.to_bits());
    assert_eq!(bits(deg_pose), bits(caixa_pose), "sem peça válida ≠ caixa");
    assert_eq!(deg_massa.to_bits(), caixa_massa.to_bits());
}

#[test]
fn mesmo_conteudo_mesma_chave_conteudo_diferente_outra() {
    let mut a = PhysicsWorld::new();
    let mut b = PhysicsWorld::new();
    let k1 = a.register_outline(pecas_u());
    let k2 = a.register_outline(pecas_u());
    let k3 = b.register_outline(pecas_u());
    // Um L (assimétrico) e o seu espelho em x.
    let l = vec![caixa(-0.5, -0.5, 0.5, -0.3), caixa(-0.5, -0.3, -0.3, 0.5)];
    let espelhado: Vec<Vec<[f32; 2]>> = l
        .iter()
        .map(|p| p.iter().map(|&[x, y]| [-x, y]).collect())
        .collect();
    let k4 = a.register_outline(l);
    let k5 = a.register_outline(espelhado);
    // As peças partidas noutro sítio (mesma união) também são outra forma.
    let k6 = a.register_outline(vec![caixa(-1.0, -0.75, 1.0, 0.75)]);
    let k7 = a.register_outline(vec![
        caixa(-1.0, -0.75, 1.0, 0.0),
        caixa(-1.0, 0.0, 1.0, 0.75),
    ]);
    eprintln!("chaves: {k1:?} {k2:?} {k3:?} · Π {k4:?} espelhado {k5:?} · {k6:?} {k7:?}");
    assert_eq!(k1, k2, "o mesmo conteúdo deu chaves diferentes");
    assert_eq!(k1, k3, "a chave depende do mundo");
    assert_ne!(k1, k4);
    assert_ne!(k4, k5, "o espelho deu a mesma chave");
    assert_ne!(k6, k7);
}
