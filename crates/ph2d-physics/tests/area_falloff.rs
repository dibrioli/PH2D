//! **O falloff de uma zona de força** — o empurrão que enfraquece do centro para a borda
//! (ADR-0131 W-AreaFalloff), o último item aberto da família das zonas.
//!
//! Duas metades, e a primeira é a que decide o resto: a RÉGUA
//! ([`ShapeDesc::radial_fraction`]) tem de valer exatamente `1` sobre a fronteira em toda
//! direção, senão "desvanece até a borda" é uma frase sem número. Ela é testada contra
//! pontos de fronteira construídos da DEFINIÇÃO de cada forma — o retângulo pelo seu
//! perímetro, a cápsula pelos seus flancos e calotas — nunca pela fórmula sob teste, que
//! seria o oráculo sempre-verde ([[reference_topic_oracle_discipline]]).
//!
//! A segunda metade é o comportamento: o corpo perto do centro anda mais que o da borda, o
//! giro obedece ao mesmo fator, e o MEIO (arrasto, empuxo, arrasto de forma) não obedece —
//! essa última é a fronteira do escopo, e é gate para ninguém "completar" a wave.

use ph2d_physics::{
    AreaEffect, BodyDesc, LayerMatrix, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc,
};

/// Um corpo com tudo neutro — as fixtures sobrescrevem só o que estão testando.
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

/// Uma zona-sensor centrada na origem, com o efeito que o gate está medindo.
fn zone(half_x: f32, half_y: f32, effect: AreaEffect) -> BodyDesc {
    BodyDesc {
        is_sensor: true,
        effector: Some(effect),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Cuboid { half_x, half_y },
        )
    }
}

/// O efeito neutro; cada gate liga o campo de que precisa.
fn effect() -> AreaEffect {
    AreaEffect {
        force: [0.0, 0.0],
        drag: 0.0,
        density: 0.0,
        form_drag: 0.0,
        torque: 0.0,
        world_axes: false,
        falloff: 0.0,
        mirror: [1.0, 1.0],
    }
}

/// Um corpo à deriva. ⚠️ O raio é `0.5` — 0.785 kg — e isso é fixture, não estética: com
/// um corpo minúsculo o mesmo vento acelera a 127 m/s² e ele ATRAVESSA a zona antes de o
/// gate medir, que foi exatamente como a 1ª versão destes gates perdeu o próprio controle.
fn drifter(x: f32, y: f32) -> BodyDesc {
    desc(
        RigidBodyType::Dynamic,
        x,
        y,
        ShapeDesc::Ball { radius: 0.5 },
    )
}

/// Sem gravidade: a zona é a única coisa agindo, então o que o corpo faz É o que ela fez.
fn world() -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.set_layer_matrix(LayerMatrix::all());
    w
}

fn x_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("body alive").translation.x
}

// ─────────────────────────── a RÉGUA ───────────────────────────

/// **A fronteira vale exatamente 1, em toda direção e em toda forma.**
///
/// É a afirmação de que o falloff "chega a zero na borda" — sem ela a frase não tem
/// número, e um corpo saindo da zona ainda daria o degrau que a wave existe para tirar.
///
/// Os pontos de fronteira vêm da DEFINIÇÃO de cada forma (o perímetro do retângulo, a
/// parametrização da elipse, os flancos e calotas do estádio), nunca de
/// `radial_fraction` — um oráculo que chama a função sob teste é verde por construção.
#[test]
fn the_measure_is_one_on_every_boundary_direction() {
    let n = 64;
    // Elipse (e o círculo, que é o caso rx == ry): (rx cosθ, ry sinθ) está na borda.
    for (shape, rx, ry) in [
        (ShapeDesc::Ball { radius: 2.0 }, 2.0_f32, 2.0_f32),
        (ShapeDesc::Ellipse { rx: 3.0, ry: 1.0 }, 3.0, 1.0),
    ] {
        for i in 0..n {
            let a = (i as f32) * std::f32::consts::TAU / (n as f32);
            let (s, c) = a.sin_cos();
            let t = shape.radial_fraction([rx * c, ry * s]);
            assert!(
                (t - 1.0).abs() < 1e-4,
                "{shape:?}: o ponto de borda em {a} rad mediu {t}, não 1"
            );
        }
    }
    // Retângulo: o perímetro, quinas incluídas — o max-norm é 1 no lado E na quina.
    let (hx, hy) = (2.5_f32, 0.75_f32);
    let cuboid = ShapeDesc::Cuboid {
        half_x: hx,
        half_y: hy,
    };
    for i in 0..=n {
        let u = -1.0 + 2.0 * (i as f32) / (n as f32);
        for p in [[u * hx, hy], [u * hx, -hy], [hx, u * hy], [-hx, u * hy]] {
            let t = cuboid.radial_fraction(p);
            assert!(
                (t - 1.0).abs() < 1e-5,
                "cuboid: o ponto de borda {p:?} mediu {t}, não 1"
            );
        }
    }
    // Estádio (e a cápsula exata, rx == ry): flancos retos em |x| = rx com |y| <= h, e
    // as duas calotas de meia-elipse em torno de (0, ±h).
    for (shape, h, rx, ry) in [
        (
            ShapeDesc::Capsule {
                half_height: 1.0,
                radius: 0.5,
            },
            1.0_f32,
            0.5_f32,
            0.5_f32,
        ),
        (
            ShapeDesc::Stadium {
                half_height: 0.8,
                rx: 1.5,
                ry: 0.4,
            },
            0.8,
            1.5,
            0.4,
        ),
    ] {
        for i in 0..=n {
            let u = -1.0 + 2.0 * (i as f32) / (n as f32);
            let t = shape.radial_fraction([rx, u * h]);
            assert!(
                (t - 1.0).abs() < 1e-4,
                "{shape:?}: o flanco em y = {} mediu {t}, não 1",
                u * h
            );
            let a = (i as f32) * std::f32::consts::PI / (n as f32);
            let (s, c) = a.sin_cos();
            let t = shape.radial_fraction([rx * c, h + ry * s]);
            assert!(
                (t - 1.0).abs() < 1e-4,
                "{shape:?}: a calota em {a} rad mediu {t}, não 1"
            );
        }
    }
}

/// **O centro é 0 e o lado de fora passa de 1** — as duas outras âncoras da régua.
#[test]
fn the_centre_is_zero_and_the_outside_is_past_one() {
    for shape in [
        ShapeDesc::Ball { radius: 2.0 },
        ShapeDesc::Cuboid {
            half_x: 2.5,
            half_y: 0.75,
        },
        ShapeDesc::Ellipse { rx: 3.0, ry: 1.0 },
        ShapeDesc::Capsule {
            half_height: 1.0,
            radius: 0.5,
        },
        ShapeDesc::Stadium {
            half_height: 0.8,
            rx: 1.5,
            ry: 0.4,
        },
    ] {
        assert_eq!(
            shape.radial_fraction([0.0, 0.0]),
            0.0,
            "{shape:?}: o centro tem de ser exatamente 0"
        );
        let far = shape.radial_fraction([100.0, 100.0]);
        assert!(far > 1.0, "{shape:?}: um ponto muito fora mediu {far} <= 1");
    }
}

/// **A régua é invariante sob escala** — é ISSO que faz o anel do overlay ser a curva de
/// nível exata (a silhueta encolhida) e que faz o falloff acompanhar a escala do W6 de
/// graça. Escalar a forma e o ponto pelos MESMOS fatores não pode mover o número.
#[test]
fn the_measure_is_invariant_under_scale() {
    let (sx, sy) = (3.0_f32, 0.4_f32);
    let scaled = |s: ShapeDesc| match s {
        // Uma bola sob escala não-uniforme é genuinamente uma elipse — a mesma resolução
        // que o `scaled_shape` faz, escrita aqui à mão para não chamar a porta do produto.
        ShapeDesc::Ball { radius } => ShapeDesc::Ellipse {
            rx: radius * sx,
            ry: radius * sy,
        },
        ShapeDesc::Cuboid { half_x, half_y } => ShapeDesc::Cuboid {
            half_x: half_x * sx,
            half_y: half_y * sy,
        },
        ShapeDesc::Ellipse { rx, ry } => ShapeDesc::Ellipse {
            rx: rx * sx,
            ry: ry * sy,
        },
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => ShapeDesc::Stadium {
            half_height: half_height * sy,
            rx: radius * sx,
            ry: radius * sy,
        },
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => ShapeDesc::Stadium {
            half_height: half_height * sy,
            rx: rx * sx,
            ry: ry * sy,
        },
    };
    for shape in [
        ShapeDesc::Ball { radius: 2.0 },
        ShapeDesc::Cuboid {
            half_x: 2.5,
            half_y: 0.75,
        },
        ShapeDesc::Ellipse { rx: 3.0, ry: 1.0 },
        ShapeDesc::Capsule {
            half_height: 1.0,
            radius: 0.5,
        },
    ] {
        for p in [[0.3, 0.0], [0.0, 0.4], [0.7, 0.9], [-1.2, 0.35]] {
            let a = shape.radial_fraction(p);
            let b = scaled(shape).radial_fraction([p[0] * sx, p[1] * sy]);
            assert!(
                (a - b).abs() < 1e-4,
                "{shape:?} em {p:?}: {a} sem escala vs {b} com escala — a régua não é \
                 invariante, e então o anel de meio caminho do overlay descreveria outra curva"
            );
        }
    }
}

// ─────────────────────────── o COMPORTAMENTO ───────────────────────────

/// A meia-extensão da zona destes gates. Grande de propósito: o corpo da borda tem de
/// continuar DENTRO durante a corrida inteira, senão "andou menos" também é o que acontece
/// com quem simplesmente saiu — o controle atropelado pelo próprio experimento, que esta
/// linha já pagou três vezes ([[reference_topic_fixture_discipline]]).
const HALF: f32 = 12.0;

/// A 90% do caminho até a borda: `t = 0.9`, longe o bastante para o fator morder e dentro
/// o bastante para a fixture continuar contendo o fenômeno.
const NEAR_EDGE: f32 = 0.9 * HALF;

/// Solta dois corpos numa zona de vento — um no olho, outro perto da borda — e devolve a
/// VELOCIDADE de cada um. A velocidade é a integral direta do impulso, então ela mede o
/// empurrão sem depender de quanto o corpo viajou.
fn two_drifters(falloff: f32) -> (f32, f32) {
    let mut w = world();
    w.spawn_body(zone(
        HALF,
        HALF,
        AreaEffect {
            force: [4.0, 0.0],
            falloff,
            ..effect()
        },
    ));
    // Ambos na MESMA linha do vento (y = 0) para que só a distância ao centro os separe.
    let near = w.spawn_body(drifter(0.0, 0.0));
    let far = w.spawn_body(drifter(NEAR_EDGE, 0.0));
    for _ in 0..12 {
        w.step();
    }
    // A premissa, declarada: os dois ainda estão na zona. Sem isto o gate mede a saída.
    for h in [near, far] {
        let x = x_of(&w, h);
        assert!(
            x.abs() < HALF,
            "a fixture perdeu o fenômeno: o corpo saiu da zona (x = {x}, meia-extensão {HALF})"
        );
    }
    (
        w.bodies().get(near).expect("body alive").linvel().x,
        w.bodies().get(far).expect("body alive").linvel().x,
    )
}

/// **O empurrão desvanece do centro para a borda** — a wave inteira, numa asserção.
///
/// E o CONTROLE está no mesmo gate, porque sem ele "foram empurrados diferente" também é o
/// que acontece quando alguma outra coisa os separa: sem falloff os dois corpos, na mesma
/// linha do mesmo vento, têm de sair com a MESMA velocidade — e bit a bit, porque um campo
/// uniforme não tem de onde tirar uma diferença.
#[test]
fn the_push_fades_from_the_centre_to_the_edge() {
    let (flat_near, flat_far) = two_drifters(0.0);
    assert_eq!(
        flat_near, flat_far,
        "controle: sem falloff o campo é uniforme, então os dois corpos têm de sair \
         idênticos — qualquer diferença aqui invalida a medição abaixo"
    );
    let (near, far) = two_drifters(1.0);
    assert!(
        near > far * 3.0,
        "com falloff 1 o corpo do olho ({near}) tem de ser empurrado muito mais que o da \
         borda ({far}) — o fator não está chegando ao impulso"
    );
    assert!(
        far > 0.0,
        "a 90% do caminho ainda é DENTRO da zona: o corpo tem de ser empurrado um pouco \
         ({far})"
    );
}

/// **Sem falloff a FORMA da zona não é consultada** — a identidade byte a byte da cena
/// antiga, afirmada onde ela pode ser contradita.
///
/// A wave deu à zona uma silhueta e uma régua. Se o fator vazasse para o caminho neutro, o
/// mesmo vento sobre o mesmo corpo passaria a depender de a zona ser uma caixa ou um disco
/// — que é precisamente a diferença que não existia antes desta wave. Com falloff 0 as duas
/// trajetórias têm de ser **iguais ao bit**; com falloff 1 têm de divergir, senão o gate
/// estaria verde por a régua nunca ser consultada em lugar nenhum.
#[test]
fn a_zone_without_falloff_does_not_consult_its_shape() {
    let run = |shape: ShapeDesc, falloff: f32| {
        let mut w = world();
        w.spawn_body(BodyDesc {
            is_sensor: true,
            effector: Some(AreaEffect {
                force: [4.0, 0.0],
                torque: 1.0,
                falloff,
                ..effect()
            }),
            ..desc(RigidBodyType::Fixed, 0.0, 0.0, shape)
        });
        // ⚠️ FORA do eixo, e isso é a fixture conter o fenômeno: no eixo x uma caixa de
        // meia-largura 12 e um disco de raio 12 medem o MESMO `t`, então um corpo em
        // (5.4, 0) não distingue as duas formas e o `assert_ne!` abaixo falharia sobre um
        // produto correto. Na diagonal a caixa mede 0.45 e o disco 0.64.
        let b = w.spawn_body(drifter(NEAR_EDGE * 0.5, NEAR_EDGE * 0.5));
        for _ in 0..12 {
            w.step();
        }
        let body = w.bodies().get(b).expect("body alive");
        (body.linvel().x, body.angvel())
    };
    let boxy = ShapeDesc::Cuboid {
        half_x: HALF,
        half_y: HALF,
    };
    let round = ShapeDesc::Ball { radius: HALF };
    assert_eq!(
        run(boxy, 0.0),
        run(round, 0.0),
        "com falloff 0 a silhueta da zona não pode ser lida — o caminho neutro deixou de \
         ser o que a cena antiga percorria"
    );
    assert_ne!(
        run(boxy, 1.0),
        run(round, 1.0),
        "com falloff 1 a silhueta TEM de ser lida: na diagonal o quadrado mede t = 0.45 e \
         o disco 0.64, então as duas trajetórias divergem. Se não divergem, a régua não \
         está sendo consultada e o gate de identidade acima é verde por acidente"
    );
}

/// **O falloff pesa o TORQUE também** — a metade que a nota aberta do W-AreaTorque pedia.
#[test]
fn the_falloff_scales_the_torque_too() {
    let spin = |falloff: f32, x: f32| {
        let mut w = world();
        w.spawn_body(zone(
            4.0,
            4.0,
            AreaEffect {
                torque: 2.0,
                falloff,
                ..effect()
            },
        ));
        let b = w.spawn_body(drifter(x, 0.0));
        for _ in 0..30 {
            w.step();
        }
        w.bodies().get(b).expect("body alive").angvel().abs()
    };
    let (flat_near, flat_far) = (spin(0.0, 0.0), spin(0.0, 3.6));
    assert!(
        (flat_near - flat_far).abs() < 1e-4,
        "controle: sem falloff o giro é uniforme ({flat_near} vs {flat_far})"
    );
    let (near, far) = (spin(1.0, 0.0), spin(1.0, 3.6));
    assert!(
        near > far * 3.0,
        "o redemoinho tem de girar mais no olho ({near}) que na margem ({far})"
    );
}

/// **O falloff NÃO alcança o meio** — arrasto, empuxo e arrasto de forma ficam uniformes.
///
/// A fronteira do escopo, e o gate existe para ninguém "completar" a wave passando o fator
/// adiante: um arrasto é uma SUBSTÂNCIA, e uma substância não fica mais rala perto da
/// própria margem. Irmão do gate de invariância do torque que o W-AreaFrame escreveu.
#[test]
fn the_falloff_leaves_the_medium_alone() {
    // Um corpo lançado atravessa a zona; o quanto ele desacelera é o arrasto agindo.
    let braked = |falloff: f32, x: f32| {
        let mut w = world();
        w.spawn_body(zone(
            4.0,
            4.0,
            AreaEffect {
                drag: 3.0,
                falloff,
                ..effect()
            },
        ));
        let b = w.spawn_body(BodyDesc {
            linvel: [1.0, 0.0],
            ..drifter(x, 0.0)
        });
        for _ in 0..20 {
            w.step();
        }
        w.bodies().get(b).expect("body alive").linvel().x
    };
    for x in [0.0_f32, 3.6] {
        assert_eq!(
            braked(0.0, x),
            braked(1.0, x),
            "em x = {x} o arrasto mudou com o falloff — o fator vazou do empurrão para o \
             meio, e a água da beira da piscina passou a molhar menos"
        );
    }
}

/// **Um corpo cujo centro já saiu não é puxado para TRÁS** — o cap de `t <= 1`.
///
/// A sobreposição que registra o par é forma-contra-forma, então o centro de um corpo
/// grande pode estar do lado de fora enquanto ele ainda encosta. Sem o cap o fator fica
/// negativo e a zona inverte o sinal exatamente na borda onde deveria soltar — um bug que
/// nenhum ledger acusa, porque a soma continua fechando.
#[test]
fn a_body_past_the_edge_is_never_pushed_backwards() {
    let mut w = world();
    w.spawn_body(zone(
        2.0,
        2.0,
        AreaEffect {
            force: [5.0, 0.0],
            falloff: 1.0,
            ..effect()
        },
    ));
    // Raio grande de propósito: o centro em x = 2.4 está FORA da caixa (meia-largura 2.0)
    // e mesmo assim o corpo a sobrepõe — é a fixture que contém o fenômeno.
    let b = w.spawn_body(desc(
        RigidBodyType::Dynamic,
        2.4,
        0.0,
        ShapeDesc::Ball { radius: 0.8 },
    ));
    let x0 = x_of(&w, b);
    for _ in 0..40 {
        w.step();
    }
    let moved = x_of(&w, b) - x0;
    assert!(
        moved >= -1e-4,
        "o corpo com o centro fora da zona foi puxado para TRÁS em {moved} — o `t` não \
         está capado em 1 e o fator ficou negativo"
    );
}
