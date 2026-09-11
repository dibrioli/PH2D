//! **Um corpo com MAIS DE UMA FORMA** (W-Compound) — o lado do wrapper.
//!
//! O oráculo é um **"L"**: um braço horizontal e uma perna vertical pendurada na
//! ponta dele. É a forma mais curta em que as três coisas que podem dar errado
//! são distinguíveis — a perna não existir (atravessa o chão), a perna ser outro
//! CORPO (as duas se separam), e a peça descansar na pose errada.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

/// A massa do corpo, pela mesma porta que o `mass_override` já usa.
fn mass(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("corpo vivo").mass()
}

fn desc(shape: ShapeDesc, x: f32, y: f32, body_type: RigidBodyType) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
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
        offset: [0.0, 0.0],
    }
}

fn arm_shape() -> ShapeDesc {
    ShapeDesc::Cuboid {
        half_x: 1.0,
        half_y: 0.2,
    }
}

fn leg_shape() -> ShapeDesc {
    ShapeDesc::Cuboid {
        half_x: 0.2,
        half_y: 1.0,
    }
}

/// Um "L" largado de `y = 5` sobre um chão. `with_leg` decide se a perna existe.
/// Devolve `(pose do corpo, mundo)` depois de 3 s.
fn drop_ell(with_leg: bool) -> (f32, f32, PhysicsWorld) {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 20.0, 0.5);
    let body = w.spawn_body(desc(arm_shape(), 0.0, 5.0, RigidBodyType::Dynamic));
    if with_leg {
        // A perna desce da ponta DIREITA do braço.
        w.attach_part(
            body,
            &desc(leg_shape(), 0.0, 0.0, RigidBodyType::Dynamic),
            [0.8, -1.0, 0.0],
        )
        .expect("peça pendurada");
    }
    for _ in 0..180 {
        w.step();
    }
    let p = w.body_pose(body).expect("corpo vivo");
    (p.translation.y, p.rotation.angle(), w)
}

/// **A peça EXISTE para o solver.** Sem ela o braço desce até a própria
/// meia-altura sobre o chão; com ela a perna toca primeiro e a peça inteira
/// descansa mais alto.
///
/// ⚠️ Nasceu VERMELHO: antes desta wave não havia como pendurar a segunda forma,
/// e a sonda `measure_compound` mediu o que o artista via — a perna desenhada
/// **atravessando o chão**, sem erro e sem warning.
#[test]
fn a_part_holds_the_body_up() {
    let (bare, _, _) = drop_ell(false);
    let (with, _, _) = drop_ell(true);
    assert!(
        with > bare + 0.5,
        "a peça não segurou nada: sem ela o corpo descansa em {bare:.3}, com ela em \
         {with:.3} — a perna tem de tocar o chão ANTES do braço"
    );
}

/// **E ela fica ONDE foi pendurada.** A pose local é autorada; um corpo composto
/// cuja peça deriva é duas formas empilhadas por acaso.
#[test]
fn a_part_stays_where_it_was_hung() {
    let mut w = PhysicsWorld::new();
    let body = w.spawn_body(desc(arm_shape(), 0.0, 5.0, RigidBodyType::Dynamic));
    let part = w
        .attach_part(
            body,
            &desc(leg_shape(), 0.0, 0.0, RigidBodyType::Dynamic),
            [0.8, -1.0, 0.5],
        )
        .expect("peça pendurada");
    for _ in 0..180 {
        w.step();
    }
    let local = w.part_local(part).expect("peça viva");
    assert!(
        (local[0] - 0.8).abs() < 1e-5 && (local[1] + 1.0).abs() < 1e-5,
        "a peça derivou para {local:?}, e foi pendurada em [0.8, -1.0, 0.5]"
    );
    assert!(
        (local[2] - 0.5).abs() < 1e-5,
        "a rotação local da peça virou {:.6}, autorada 0.5",
        local[2]
    );
}

/// **O OFFSET do collider da peça COMPÕE com a pose local**, não compete.
///
/// Sobrescrever a translação em vez de compor apagaria o offset em silêncio — a
/// peça pousaria na pose local crua e nada apontaria o número perdido.
#[test]
fn a_parts_offset_composes_with_where_it_hangs() {
    let mut w = PhysicsWorld::new();
    let body = w.spawn_body(desc(arm_shape(), 0.0, 5.0, RigidBodyType::Dynamic));
    let mut d = desc(leg_shape(), 0.0, 0.0, RigidBodyType::Dynamic);
    d.offset = [0.3, 0.0];
    // Pendurada em (1, 0) e girada 90°: o offset de +0,3 no frame DELA aponta
    // para +Y do mundo, então a peça tem de acabar em (1, 0.3).
    let part = w
        .attach_part(body, &d, [1.0, 0.0, std::f32::consts::FRAC_PI_2])
        .expect("peça pendurada");
    let local = w.part_local(part).expect("peça viva");
    assert!(
        (local[0] - 1.0).abs() < 1e-5 && (local[1] - 0.3).abs() < 1e-5,
        "o offset não compôs: a peça ficou em {local:?}, esperado ~[1.0, 0.3]"
    );
}

/// **A MASSA soma, e é isso que distingue um corpo composto de dois corpos.**
/// Duas formas ligadas por um Weld são duas massas que o solver pode separar;
/// aqui elas são uma peça, e a inércia do conjunto é a das duas.
#[test]
fn a_part_adds_its_mass_to_the_body() {
    let mut w = PhysicsWorld::new();
    let body = w.spawn_body(desc(arm_shape(), 0.0, 5.0, RigidBodyType::Dynamic));
    let before = mass(&w, body);
    w.attach_part(
        body,
        &desc(leg_shape(), 0.0, 0.0, RigidBodyType::Dynamic),
        [0.8, -1.0, 0.0],
    )
    .expect("peça pendurada");
    let after = mass(&w, body);
    // A perna é 0,4 × 2,0 a densidade 1 = 0,8 kg.
    assert!(
        (after - before - 0.8).abs() < 1e-4,
        "a massa foi de {before:.4} para {after:.4}; a peça pesa 0,8 kg"
    );
}

/// **Tirar a peça devolve o corpo ao que ele era** — a metade que torna o
/// reconcile possível (uma peça apagada na Hierarquia tem de sumir do solver).
#[test]
fn detaching_a_part_gives_the_body_back() {
    let mut w = PhysicsWorld::new();
    let body = w.spawn_body(desc(arm_shape(), 0.0, 5.0, RigidBodyType::Dynamic));
    let before = mass(&w, body);
    let part = w
        .attach_part(
            body,
            &desc(leg_shape(), 0.0, 0.0, RigidBodyType::Dynamic),
            [0.8, -1.0, 0.0],
        )
        .expect("peça pendurada");
    w.detach_part(part);
    let after = mass(&w, body);
    assert!(
        (after - before).abs() < 1e-4,
        "a massa ficou em {after:.4} depois do detach, e era {before:.4}"
    );
    assert!(
        w.part_local(part).is_none(),
        "o handle da peça sobreviveu ao detach"
    );
}
