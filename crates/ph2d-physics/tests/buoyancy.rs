//! Empuxo — a área sabe QUANTO do corpo está dentro dela (ADR-0131 W-Buoyancy).
//!
//! O oráculo destes gates é sempre o que um artista veria, nunca a fórmula: o corpo
//! leve **para na linha d'água** (em vez de ser arremessado, que é o que a força
//! constante do W-Area fazia), o denso **afunda**, e o barco tombado **se endireita**.

use ph2d_physics::{AreaEffect, BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
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
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// Uma piscina: sensor estático de 8 × 4 cuja superfície fica em y = 0.
fn pool(w: &mut PhysicsWorld, fluid_density: f32) {
    w.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 1.5,
            density: fluid_density,
            form_drag: 0.0,
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            -2.0,
            ShapeDesc::Cuboid {
                half_x: 4.0,
                half_y: 2.0,
            },
        )
    });
}

/// Uma caixa de meio metro de lado, de densidade `d`, largada em `(x, y)`.
fn crate_at(w: &mut PhysicsWorld, x: f32, y: f32, d: f32) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        density: d,
        ..desc(
            RigidBodyType::Dynamic,
            x,
            y,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        )
    })
}

fn y_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("body alive").translation.y
}

#[test]
fn a_light_body_settles_at_the_waterline_instead_of_being_thrown_out() {
    // ⚠️ **A razão de existir da wave.** Com a `Force Y` constante do W-Area, um corpo
    // leve o bastante para subir NUNCA para: a força não sabe onde a superfície está,
    // então ele sai voando da piscina. Arquimedes se auto-nivela porque a força é
    // proporcional à área SUBMERSA — quando o corpo emerge, ela cai.
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    // Densidade 1 contra fluido 4: submerge ~1/4 da altura, ou seja o centro fica
    // ~0.125 m ABAIXO da superfície (meia altura 0.25).
    let c = crate_at(&mut w, 0.0, -1.0, 1.0);
    for _ in 0..900 {
        w.step();
    }
    let settled = y_of(&w, c);
    assert!(
        settled.abs() < 0.3,
        "a caixa leve tem de PARAR na linha d'água (y ≈ 0), e ficou em {settled}"
    );
    // E FICA lá — o teste que a força constante do W-Area reprova. ⚠️ O sistema é
    // amortecido, então "parou" é a deriva ao longo dos últimos 300 passos, não uma
    // igualdade: medir cedo demais (a 1ª versão media a 400) reprova um corpo que
    // ainda está assentando, o que é o produto funcionando.
    for _ in 0..300 {
        w.step();
    }
    assert!(
        (y_of(&w, c) - settled).abs() < 0.03,
        "a linha d'água tem de ser um EQUILÍBRIO, não uma passagem ({settled} -> {})",
        y_of(&w, c)
    );
}

#[test]
fn density_decides_who_floats_and_it_is_not_mass() {
    // *Madeira boia, pedra afunda* — e as duas podem ter a mesma massa. Três caixas do
    // MESMO tamanho (logo a mesma área submersa disponível) e três densidades: a que é
    // menos densa que o fluido boia, a mais densa afunda. Uma `Force Y` constante não
    // consegue expressar isso: ela é vencida pelo PESO, então dois corpos de mesma massa
    // e densidades diferentes se comportariam igual.
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    let light = crate_at(&mut w, -2.0, -1.0, 1.0);
    let neutral = crate_at(&mut w, 0.0, -1.0, 4.0);
    let heavy = crate_at(&mut w, 2.0, -1.0, 12.0);
    for _ in 0..400 {
        w.step();
    }
    let (l, n, h) = (y_of(&w, light), y_of(&w, neutral), y_of(&w, heavy));
    assert!(
        l > n && n > h,
        "menos denso boia mais alto: leve {l} > neutro {n} > pesado {h}"
    );
    assert!(
        l > -0.3,
        "a caixa 4x menos densa que o fluido tem de vir à superfície ({l})"
    );
    assert!(
        h < -1.5,
        "a caixa 3x mais densa que o fluido tem de afundar ({h})"
    );
}

#[test]
fn a_capsized_body_rights_itself() {
    // ⚠️ O gate do `apply_impulse_at_point`. O empuxo age no centroide da parte
    // SUBMERSA; quando o corpo inclina, esse centroide sai de cima do centro de massa e
    // o braço de alavanca gera um torque restaurador. Uma força no centro de massa
    // flutuaria exatamente igual e deixaria o barco tombado para sempre — e é por isso
    // que este é o único gate que distingue as duas.
    let mut w = PhysicsWorld::new();
    pool(&mut w, 6.0);
    // Um "barco": caixa larga e baixa, começando inclinada quase 60°.
    //
    // ⚠️ O oráculo mede `sin` do ângulo, não o ângulo: um retângulo **não tem quilha**,
    // então 0 e π são a MESMA pose flutuante e ambas estão endireitadas. A 1ª versão
    // exigia ângulo pequeno e ficou vermelha sobre um barco perfeitamente nivelado que
    // por acaso tinha girado 180° (3,141 rad) — o produto estava certo, a pergunta é
    // que estava errada.
    let boat = w.spawn_body(BodyDesc {
        rotation: 1.0,
        density: 1.0,
        ..desc(
            RigidBodyType::Dynamic,
            0.0,
            -0.2,
            ShapeDesc::Cuboid {
                half_x: 0.8,
                half_y: 0.15,
            },
        )
    });
    for _ in 0..600 {
        w.step();
    }
    let tilt = w
        .body_pose(boat)
        .expect("boat")
        .rotation
        .angle()
        .sin()
        .abs();
    assert!(
        tilt < 0.35,
        "o barco largado a 1,0 rad (sin = 0,84) tem de ficar com o eixo longo na \
         horizontal (|sin| < 0,35), e ficou em {tilt}"
    );
}

#[test]
fn a_body_above_the_surface_is_untouched() {
    // Nenhuma parte submersa, nenhuma força — e nada de acordar o corpo à toa. Sem isto
    // o empuxo seria um campo que age à distância dentro da bounding box da poça.
    // ⚠️ Gravidade NORMAL, e o controle é quem não sente gravidade — a 1ª versão zerou
    // a gravidade do MUNDO e assim desligou o próprio empuxo que ela media (o controle
    // ficou parado por não haver empuxo nenhum: verde pelo motivo errado, e o gate
    // irmão `zero_gravity_means_no_buoyancy` já cobre esse caso).
    // ⚠️ E o controle fica numa COLUNA diferente do experimento. Na 1ª versão ele
    // estava logo acima da poça e o corpo de baixo — que a poça de densidade 100
    // arremessava como um foguete — o atingiu a 21,8 m de altura. É a terceira vez que
    // esta linha vê um controle atropelado pelo próprio experimento (o W-Area teve duas):
    // um controle tem de estar fora do CAMINHO, e o caminho aqui é uma coluna.
    let mut w = PhysicsWorld::new();
    pool(&mut w, 8.0);
    let above = w.spawn_body(BodyDesc {
        gravity_scale: 0.0,
        ..desc(
            RigidBodyType::Dynamic,
            2.5,
            3.0,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        )
    });
    let below = crate_at(&mut w, -2.5, -1.0, 1.0);
    for _ in 0..60 {
        w.step();
    }
    assert!(
        (y_of(&w, above) - 3.0).abs() < 1e-6,
        "um corpo fora da poça não pode ser tocado ({})",
        y_of(&w, above)
    );
    assert!(
        y_of(&w, below) > -0.9,
        "o controle dentro da poça tem de subir ({})",
        y_of(&w, below)
    );
}

#[test]
fn zero_gravity_means_no_buoyancy() {
    // Arquimedes é consequência do PESO do fluido. Sem gravidade não há empuxo — o caso
    // degenerado se resolve pela própria física, sem um `if` de caso especial que
    // alguém teria de lembrar de manter.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    pool(&mut w, 50.0);
    let c = crate_at(&mut w, 0.0, -1.0, 1.0);
    for _ in 0..120 {
        w.step();
    }
    assert!(
        (y_of(&w, c) + 1.0).abs() < 1e-6,
        "sem gravidade nada pode boiar ({})",
        y_of(&w, c)
    );
}

#[test]
fn the_surface_is_perpendicular_to_gravity_not_to_the_y_axis() {
    // ⚠️ Água tem superfície horizontal mesmo numa piscina torta, e o mesmo raciocínio
    // diz que com gravidade LATERAL a superfície é vertical. Uma implementação que
    // tomasse "o topo da AABB em Y" acertaria por acidente no caso comum e erraria aqui.
    // Gravidade para +x: o corpo tem de ser empurrado para -x.
    let mut w = PhysicsWorld::new();
    w.set_gravity(9.81, 0.0);
    pool(&mut w, 8.0);
    let c = crate_at(&mut w, 0.0, -1.0, 1.0);
    let x0 = w.body_pose(c).expect("crate").translation.x;
    for _ in 0..120 {
        w.step();
    }
    let x1 = w.body_pose(c).expect("crate").translation.x;
    assert!(
        x1 < x0 - 0.3,
        "com gravidade em +x o empuxo tem de empurrar em -x ({x0} -> {x1})"
    );
}

#[test]
fn a_tessellated_ball_carries_the_documented_polygon_bias() {
    // ⚠️ O viés que o módulo NOMEIA, medido em vez de acreditado. O rapier representa
    // uma bola exatamente, mas o empuxo a recorta como um polígono de 32 lados, cuja
    // área é `(N/2π)·sin(2π/N)` = 99,36% da do círculo. Totalmente submersa, a força
    // sai 0,64% menor que a de Arquimedes — e o gate afirma ESSE número, para que uma
    // troca de tesselação apareça como quantidade nomeada em vez de deriva silenciosa.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, -10.0);
    // Bola de raio 1 (área exata π), totalmente submersa numa poça funda.
    w.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 0.0,
            density: 1.0,
            form_drag: 0.0,
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            -10.0,
            ShapeDesc::Cuboid {
                half_x: 20.0,
                half_y: 20.0,
            },
        )
    });
    let ball = w.spawn_body(BodyDesc {
        gravity_scale: 0.0,
        ..desc(
            RigidBodyType::Dynamic,
            0.0,
            -10.0,
            ShapeDesc::Ball { radius: 1.0 },
        )
    });
    // ⚠️ Alguns ticks ANTES de medir. Uma zona só alcança corpos que o sub-passo
    // anterior reportou como sobrepostos (o lag de um sub-passo que o módulo documenta
    // desde o W-Area), então o PRIMEIRO tick aplica 3 dos 4 sub-passos e a razão sai
    // exatamente 3/4 do esperado — foi o que a 1ª versão deste gate mediu (0,745 contra
    // 0,994), e o número denunciou o mecanismo. Em regime, os 4 sub-passos aplicam.
    for _ in 0..10 {
        w.step();
    }
    let m = std::f32::consts::PI; // densidade 1 × área π
    let before = w.bodies().get(ball).expect("ball").linvel().y;
    w.step();
    let v = w.bodies().get(ball).expect("ball").linvel().y - before;
    // ρ·|g|·A·dt / m, somado sobre os sub-passos de um tick = ρ·|g|·A·dt_tick / m.
    let ideal = 1.0 * 10.0 * std::f32::consts::PI * w.dt() / m;
    let ratio = v / ideal;
    let n = f64::from(ph2d_physics::ELLIPSE_SEGS);
    let expected =
        (n / (2.0 * std::f64::consts::PI) * (2.0 * std::f64::consts::PI / n).sin()) as f32;
    assert!(
        (ratio - expected).abs() < 0.002,
        "a bola tesselada tem de render {expected} da força exata (o viés de 0,64% que o \
         módulo documenta), e rendeu {ratio}"
    );
}

#[test]
fn the_waterline_is_level_even_in_a_tilted_pool() {
    // ⚠️ O gate que só a linha d'água pode falhar: água é HORIZONTAL mesmo numa poça
    // torta, então a superfície não é a aresta de cima do collider. Uma poça girada 0,4
    // rad tem de devolver dois pontos na MESMA altura — e a aresta de cima dela, não.
    let mut w = PhysicsWorld::new();
    w.spawn_body(BodyDesc {
        rotation: 0.4,
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 0.0,
            density: 4.0,
            form_drag: 0.0,
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 3.0,
                half_y: 1.0,
            },
        )
    });
    let lines = w.waterlines();
    assert_eq!(lines.len(), 1, "uma poça com empuxo tem uma linha d'água");
    let (a, b) = lines[0];
    assert!(
        (a[1] - b[1]).abs() < 1e-5,
        "a linha tem de ser HORIZONTAL numa poça inclinada, e saiu {a:?} -> {b:?}"
    );
    // E ela passa pelo ponto mais alto do collider — é ali que a água acaba.
    let top = 3.0f32 * 0.4f32.sin() + 1.0 * 0.4f32.cos();
    assert!(
        (a[1] - top).abs() < 1e-3,
        "a superfície fica no extremo superior do collider ({}, esperado {top})",
        a[1]
    );

    // ⚠️ **E com gravidade LATERAL a linha é VERTICAL.** Sem esta metade o gate é verde
    // sobre a versão errada: com gravidade padrão `-g/|g|` É exatamente `(0, 1)`, então
    // uma implementação que tomasse "o topo em Y" passaria em tudo acima — foi
    // precisamente o que a mutação mostrou, e a fixture é que não continha o fenômeno.
    let mut side = PhysicsWorld::new();
    side.set_gravity(9.81, 0.0);
    side.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 0.0,
            density: 4.0,
            form_drag: 0.0,
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 3.0,
                half_y: 1.0,
            },
        )
    });
    let (c, d) = side.waterlines()[0];
    assert!(
        (c[0] - d[0]).abs() < 1e-5 && (c[1] - d[1]).abs() > 1.0,
        "com gravidade em +x a superfície é VERTICAL (x constante, y variando), e saiu \
         {c:?} -> {d:?}"
    );
    assert!(
        (c[0] + 3.0).abs() < 1e-4,
        "e ela fica no extremo −x do collider, que é o 'alto' desta gravidade ({})",
        c[0]
    );
}

#[test]
fn only_a_buoyant_zone_has_a_waterline() {
    // Uma zona que só empurra ou só resiste não tem superfície nenhuma para mostrar —
    // desenhar uma linha nela diria ao artista que ali há água, e não há.
    let mut w = PhysicsWorld::new();
    w.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 5.0],
            drag: 2.0,
            density: 0.0,
            form_drag: 0.0,
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 2.0,
                half_y: 2.0,
            },
        )
    });
    assert!(
        w.waterlines().is_empty(),
        "vento e xarope não têm linha d'água"
    );

    // E sem gravidade também não: não há superfície que signifique alguma coisa, a mesma
    // resposta que o empuxo dá.
    let mut w2 = PhysicsWorld::new();
    w2.set_gravity(0.0, 0.0);
    pool(&mut w2, 4.0);
    assert!(
        w2.waterlines().is_empty(),
        "sem gravidade não há superfície"
    );
}

#[test]
fn shape_drag_resists_by_section_where_uniform_drag_cannot() {
    // O fato do W-FormDrag no MUNDO, não no kernel: o MESMO tronco largado de través
    // desce mais devagar que de proa. Um `drag` uniforme dá exatamente a mesma queda nas
    // duas poses — é essa diferença que a resistência de FORMA compra.
    let fall = |rotation: f32, form: f32| {
        let mut w = PhysicsWorld::new();
        w.spawn_body(BodyDesc {
            is_sensor: true,
            effector: Some(AreaEffect {
                force: [0.0, 0.0],
                drag: 0.0,
                density: 0.0,
                form_drag: form,
            }),
            ..desc(
                RigidBodyType::Fixed,
                0.0,
                -5.0,
                ShapeDesc::Cuboid {
                    half_x: 6.0,
                    half_y: 6.0,
                },
            )
        });
        let b = w.spawn_body(BodyDesc {
            rotation,
            lock_rotation: true,
            ..desc(
                RigidBodyType::Dynamic,
                0.0,
                -2.0,
                ShapeDesc::Cuboid {
                    half_x: 1.0,
                    half_y: 0.25,
                },
            )
        });
        for _ in 0..90 {
            w.step();
        }
        -y_of(&w, b)
    };
    let (broadside, edge_on) = (fall(0.0, 3.0), fall(std::f32::consts::FRAC_PI_2, 3.0));
    assert!(
        edge_on > broadside * 1.4,
        "de proa o tronco tem de descer bem mais que de través ({edge_on} contra \
         {broadside}) — é a secção que resiste"
    );
    // E o controle: sem arrasto de forma, as duas poses são idênticas (a rotação está
    // travada, então nada mais as distingue). Sem esta metade o gate acima poderia estar
    // medindo qualquer assimetria da fixture.
    let (a, b) = (fall(0.0, 0.0), fall(std::f32::consts::FRAC_PI_2, 0.0));
    assert!(
        (a - b).abs() < 1e-4,
        "sem Shape Drag a orientação não pode mudar nada ({a} vs {b})"
    );
}
