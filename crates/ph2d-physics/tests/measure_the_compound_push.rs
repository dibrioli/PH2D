//! **UMA FORMA OU UMA POR PEÇA?** — a sonda do empurrão num corpo COMPOSTO.
//!
//! A `W-CompoundZone` (2026-08-02) achou que as zonas liam `rb.colliders().first()`
//! em cinco sítios e que uma jangada composta capotava. Ela curou o EMPUXO — que
//! passou a somar sobre as formas, com um `dedup` por CORPO para não somar duas
//! vezes. Esta sonda pergunta a metade que ela não perguntou: **a FORÇA, o TORQUE
//! e o ARRASTO de uma zona também são aplicados uma vez por PEÇA?**
//!
//! O laço do `effector::apply` anda sobre SOBREPOSIÇÕES, e um corpo composto
//! sobrepõe a zona com cada uma das formas dele — a mesma frase que o comentário
//! do `to_float` já carrega. O que o `to_float` faz por dedup, o `apply_impulse`
//! logo acima dele faz **N vezes**.
//!
//! ⚠️ **O oráculo tem de manter a MASSA fixa**, senão a comparação mede outra
//! coisa: uma peça acrescenta massa, e uma aceleração é `F/m`. As duas metades do
//! composto medem, juntas, exatamente a área da simples.
//!
//! Rodar: `cargo test -p ph2d-physics --release --test measure_the_compound_push -- --ignored --nocapture`

use ph2d_physics::{AreaEffect, BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

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
        lock_rotation: true,
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

/// A zona: uma correnteza larga em +X, sem gravidade nenhuma no mundo.
fn current(w: &mut PhysicsWorld, effect: AreaEffect) {
    let mut d = desc(
        ShapeDesc::Cuboid {
            half_x: 40.0,
            half_y: 20.0,
        },
        0.0,
        0.0,
        RigidBodyType::Fixed,
    );
    d.is_sensor = true;
    d.effector = Some(effect);
    w.spawn_body(d);
}

/// A zona neutra — o efeito que nao faz nada.
fn inert() -> AreaEffect {
    AreaEffect {
        force: [0.0, 0.0],
        drag: 0.0,
        density: 0.0,
        form_drag: 0.0,
        torque: 0.0,
        world_axes: false,
        falloff: 0.0,
        mirror: AreaEffect::UNMIRRORED,
    }
}

fn push_effect() -> AreaEffect {
    AreaEffect {
        force: [4.0, 0.0],
        ..inert()
    }
}

/// Um corpo de área total `1.0` — inteiro, ou partido em `parts` peças iguais
/// empilhadas. Devolve `(x depois de 2 s, massa)`.
fn carried(parts: u32, effect: AreaEffect) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    current(&mut w, effect);
    #[allow(clippy::cast_precision_loss)]
    let half_y = 0.5 / parts as f32;
    let shape = ShapeDesc::Cuboid {
        half_x: 0.5,
        half_y,
    };
    let body = w.spawn_body(desc(shape, 0.0, 0.0, RigidBodyType::Dynamic));
    for i in 1..parts {
        #[allow(clippy::cast_precision_loss)]
        let y = 2.0 * half_y * i as f32;
        w.attach_part(
            body,
            &desc(shape, 0.0, 0.0, RigidBodyType::Dynamic),
            [0.0, y, 0.0],
        );
    }
    for _ in 0..120 {
        w.step();
    }
    (x_of(&w, body), mass(&w, body))
}

fn x_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("corpo vivo").translation().x
}

fn mass(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("corpo vivo").mass()
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_compound_push() {
    println!("\n=== O EMPURRAO NUM CORPO COMPOSTO (2 s, forca 4 N, area total 1.0) ===\n");
    println!("| pecas | massa  | x depois de 2 s | razao vs 1 peca |");
    println!("|-------|--------|-----------------|-----------------|");
    let base = carried(1, push_effect()).0;
    for parts in [1u32, 2, 4] {
        let (x, m) = carried(parts, push_effect());
        println!("| {parts:5} | {m:6.4} | {x:15.4} | {:15.4} |", x / base);
    }

    println!("\n=== E o ARRASTO da mesma zona (velocidade inicial 10 m/s, sem forca) ===\n");
    let drag = AreaEffect {
        drag: 2.0,
        ..inert()
    };
    println!("| pecas | x depois de 2 s | razao vs 1 peca |");
    println!("|-------|-----------------|-----------------|");
    let base = travelled(1, drag);
    for parts in [1u32, 2, 4] {
        let x = travelled(parts, drag);
        println!("| {parts:5} | {x:15.4} | {:15.4} |", x / base);
    }
    println!();
}

/// O mesmo corpo, largado a 10 m/s e freado só pelo arrasto da zona.
fn travelled(parts: u32, effect: AreaEffect) -> f32 {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    current(&mut w, effect);
    #[allow(clippy::cast_precision_loss)]
    let half_y = 0.5 / parts as f32;
    let shape = ShapeDesc::Cuboid {
        half_x: 0.5,
        half_y,
    };
    let mut d = desc(shape, 0.0, 0.0, RigidBodyType::Dynamic);
    d.linvel = [10.0, 0.0];
    let body = w.spawn_body(d);
    for i in 1..parts {
        #[allow(clippy::cast_precision_loss)]
        let y = 2.0 * half_y * i as f32;
        w.attach_part(
            body,
            &desc(shape, 0.0, 0.0, RigidBodyType::Dynamic),
            [0.0, y, 0.0],
        );
    }
    for _ in 0..120 {
        w.step();
    }
    x_of(&w, body)
}
