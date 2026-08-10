//! **O EMPURRÃO DE UMA ZONA É UM FATO DO CORPO, E A CONSULTA DIZ O MESMO** (W-ZoneForce).
//!
//! Dois invariantes, e eles não são redundantes:
//!
//! 1. **uma vez por CORPO**, nunca uma vez por forma — a metade que a W-CompoundZone
//!    curou no empuxo e deixou por curar na força, no torque e no arrasto;
//! 2. **a consulta e o solver respondem o MESMO empurrão** — o que faz de
//!    `effector::zone_push_at` uma porta em vez de duas cópias.
//!
//! ⚠️ **O oráculo do (2) não olha para dentro:** ele compara o que a `fluid_at` DIZ
//! contra o que o corpo dinâmico ao lado de facto FAZ (`Δv` num passo). Comparar a porta
//! contra ela mesma seria verde por construção.

use ph2d_physics::{AreaEffect, BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

const DT: f32 = 1.0 / 60.0;

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

/// O efeito neutro — a zona que não faz nada.
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

/// Uma zona-sensor larga, girada por `rotation`, com o efeito dado.
fn zone(w: &mut PhysicsWorld, effect: AreaEffect, rotation: f32) {
    let mut d = desc(
        ShapeDesc::Cuboid {
            half_x: 40.0,
            half_y: 40.0,
        },
        0.0,
        0.0,
        RigidBodyType::Fixed,
    );
    d.rotation = rotation;
    d.is_sensor = true;
    d.effector = Some(effect);
    w.spawn_body(d);
}

/// Um corpo de área total `1.0`, inteiro ou partido em `parts` peças empilhadas — a
/// massa fica FIXA, senão a comparação mede outra coisa (uma aceleração é `F/m`).
fn body_of(w: &mut PhysicsWorld, parts: u32, spin: bool) -> RigidBodyHandle {
    #[allow(clippy::cast_precision_loss)]
    let half_y = 0.5 / parts as f32;
    let shape = ShapeDesc::Cuboid {
        half_x: 0.5,
        half_y,
    };
    let mut d = desc(shape, 0.0, 0.0, RigidBodyType::Dynamic);
    d.lock_rotation = !spin;
    let body = w.spawn_body(d);
    for i in 1..parts {
        #[allow(clippy::cast_precision_loss)]
        let y = 2.0 * half_y * i as f32;
        let mut part = desc(shape, 0.0, 0.0, RigidBodyType::Dynamic);
        part.lock_rotation = !spin;
        w.attach_part(body, &part, [0.0, y, 0.0]);
    }
    body
}

fn linvel(w: &PhysicsWorld, h: RigidBodyHandle) -> [f32; 2] {
    let v = w.bodies().get(h).expect("corpo vivo").linvel();
    [v.x, v.y]
}

fn angvel(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("corpo vivo").angvel()
}

/// Dois segundos numa zona, sem gravidade. Devolve `(x, |v|, ω)`.
fn run(parts: u32, effect: AreaEffect, spin: bool, v0: [f32; 2]) -> (f32, f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    zone(&mut w, effect, 0.0);
    let body = body_of(&mut w, parts, spin);
    if v0 != [0.0, 0.0] {
        w.bodies_mut()
            .get_mut(body)
            .expect("corpo vivo")
            .set_linvel(rapier_vec(v0), true);
    }
    for _ in 0..120 {
        w.step();
    }
    let x = w.bodies().get(body).expect("corpo vivo").translation().x;
    let v = linvel(&w, body);
    (x, v[0].hypot(v[1]), angvel(&w, body))
}

fn rapier_vec(v: [f32; 2]) -> rapier2d::na::Vector2<f32> {
    rapier2d::na::Vector2::new(v[0], v[1])
}

#[test]
fn a_zone_pushes_a_body_once_per_body_never_once_per_shape() {
    // ⚠️ Este gate nasceu VERMELHO, e os números do defeito eram estes: com a área
    // partida em 1/2/4 peças de massa total idêntica, o mesmo vento levava o corpo
    // `7,98 · 15,97 · 31,93` m — exatamente `1× · 2× · 4×`, uma aplicação por FORMA.
    let force = AreaEffect {
        force: [4.0, 0.0],
        ..inert()
    };
    let base = run(1, force, false, [0.0, 0.0]).0;
    for parts in [2u32, 4] {
        let x = run(parts, force, false, [0.0, 0.0]).0;
        let ratio = x / base;
        assert!(
            (ratio - 1.0).abs() < 1e-3,
            "a FORCA tem de ser aplicada uma vez por corpo: {parts} pecas andaram {x:.4} m contra {base:.4} (razao {ratio:.4})"
        );
    }
}

#[test]
fn a_zone_drags_a_body_once_per_body_never_once_per_shape() {
    // O mesmo defeito pelo outro lado: o arrasto compunha (`v·kᴺ`), então o corpo
    // partido parava mais depressa — 4,95 · 2,53 · 1,28 m era a assinatura.
    let drag = AreaEffect {
        drag: 2.0,
        ..inert()
    };
    let base = run(1, drag, false, [10.0, 0.0]).0;
    for parts in [2u32, 4] {
        let x = run(parts, drag, false, [10.0, 0.0]).0;
        let ratio = x / base;
        assert!(
            (ratio - 1.0).abs() < 1e-3,
            "o ARRASTO tem de ser aplicado uma vez por corpo: {parts} pecas andaram {x:.4} m contra {base:.4} (razao {ratio:.4})"
        );
    }
}

#[test]
fn a_zone_spins_a_body_once_per_body_never_once_per_shape() {
    // A terceira grandeza do mesmo laço. Sem `lock_rotation`, senão não há o que medir.
    let torque = AreaEffect {
        torque: 0.5,
        ..inert()
    };
    let base = run(1, torque, true, [0.0, 0.0]).2;
    assert!(base.abs() > 0.1, "o controle tem de girar: {base:.4} rad/s");
    for parts in [2u32, 4] {
        let w = run(parts, torque, true, [0.0, 0.0]).2;
        let ratio = w / base;
        assert!(
            (ratio - 1.0).abs() < 1e-2,
            "o TORQUE tem de ser aplicado uma vez por corpo: {parts} pecas giram a {w:.4} rad/s contra {base:.4} (razao {ratio:.4})"
        );
    }
}

/// O que a consulta DIZ contra o que o solver FAZ, num passo, sem mais nada agindo.
fn said_versus_done(effect: AreaEffect, rotation: f32, at: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    zone(&mut w, effect, rotation);
    let mut d = desc(
        ShapeDesc::Ball { radius: 0.5 },
        at[0],
        at[1],
        RigidBodyType::Dynamic,
    );
    d.lock_rotation = true;
    let body = w.spawn_body(d);
    // Um passo em vazio para o grafo de interseção existir antes de perguntarmos.
    w.step();
    let before = linvel(&w, body);
    let said = w.fluid_at(body).push;
    w.step();
    let after = linvel(&w, body);
    (
        [said[0] * DT, said[1] * DT],
        [after[0] - before[0], after[1] - before[1]],
    )
}

#[test]
fn the_query_says_the_push_the_solver_applies() {
    // ⚠️ O oráculo é o `Δv` de um corpo DINÂMICO — a consulta afirma uma aceleração, e
    // um passo tem de a entregar. Uma zona GIRADA está na fixture de propósito: é ela
    // que prova que o frame (W-AreaFrame) atravessa a porta em vez de ser re-derivado.
    let force = AreaEffect {
        force: [4.0, 0.0],
        ..inert()
    };
    for rotation in [0.0f32, std::f32::consts::FRAC_PI_3] {
        let (said, done) = said_versus_done(force, rotation, [0.0, 0.0]);
        for i in 0..2 {
            assert!(
                (said[i] - done[i]).abs() < 1e-6,
                "rot {rotation:.3}, eixo {i}: a consulta disse {:.8} e o solver fez {:.8}",
                said[i],
                done[i]
            );
        }
    }
}

#[test]
fn the_query_carries_the_falloff_the_solver_carries() {
    // A terceira porta que a composição herda. ⚠️ A banda é maior de propósito e o
    // motivo é físico: o solver reavalia o falloff a cada SUB-PASSO, na posição nova
    // do corpo, e a consulta responde pela posição do INÍCIO do tique. Longe do centro
    // as duas divergem por quanto o corpo andou dentro do tique.
    let fade = AreaEffect {
        force: [8.0, 0.0],
        falloff: 1.0,
        ..inert()
    };
    let (said, done) = said_versus_done(fade, 0.0, [0.0, 20.0]);
    assert!(
        said[0] > 0.0,
        "a margem ainda recebe empurrao: {:.8}",
        said[0]
    );
    assert!(
        (said[0] - done[0]).abs() < 1e-4,
        "com falloff a consulta disse {:.8} e o solver fez {:.8}",
        said[0],
        done[0]
    );
}

#[test]
fn the_query_answers_once_per_body_just_as_the_solver_does() {
    // ⚠️ A consulta anda pelas FORMAS do corpo (é assim que ela é barata: o custo é o
    // número de sobreposições dele, não o de zonas do mundo), então ela tem a MESMA
    // armadilha do solver — sem a dedup ela diria `N ×` o empurrão para um corpo
    // composto, e a lei cinemática arrancaria com o dobro do vento que o dinâmico ao
    // lado sente. A massa fica fixa: o que varia é só em quantas formas ela está.
    let force = AreaEffect {
        force: [4.0, 0.0],
        ..inert()
    };
    let mut said = Vec::new();
    for parts in [1u32, 2, 4] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        zone(&mut w, force, 0.0);
        let body = body_of(&mut w, parts, false);
        w.step();
        said.push(w.fluid_at(body).push[0]);
    }
    assert!(said[0] > 0.0, "o controle recebe empurrao: {:.6}", said[0]);
    for (parts, got) in [2u32, 4].iter().zip(&said[1..]) {
        let ratio = got / said[0];
        assert!(
            (ratio - 1.0).abs() < 1e-5,
            "a consulta tem de dizer o mesmo para {parts} pecas: {got:.6} contra {:.6} (razao {ratio:.4})",
            said[0]
        );
    }
}

#[test]
fn a_dry_body_is_told_nothing() {
    // O CONTROLE: sem zona a consulta devolve o neutro exato, e é ele que faz da lei
    // cinemática byte-idêntica ao mundo de antes desta wave.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let body = w.spawn_body(desc(
        ShapeDesc::Ball { radius: 0.5 },
        0.0,
        0.0,
        RigidBodyType::Dynamic,
    ));
    w.step();
    assert_eq!(w.fluid_at(body).push, [0.0, 0.0]);
}
