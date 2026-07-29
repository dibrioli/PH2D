//! Os gates da geometria da rota (W-Pulley W1).
//!
//! Todos aqui são **puros** — nenhum corpo, nenhum solver: a rota é função dos
//! dois pontos de amarração e da lista de rodas, e é onde toda a matemática nova
//! da wave mora. O que o kernel de impulso faz com ela é gateado à parte.

use super::*;

fn near(a: f32, b: f32, tol: f32, what: &str) {
    assert!((a - b).abs() < tol, "{what}: {a} vs {b}");
}

/// **A tangente TANGENCIA** — e isso é o teste inteiro da fórmula.
///
/// Duas afirmações, e nenhuma delas é a fórmula de volta: o ponto está *no aro*
/// (distância exatamente `r` do centro) e o trecho *encosta* (o raio até ele é
/// perpendicular à corda). Uma fórmula errada acerta uma das duas por acidente e
/// nunca as duas.
#[test]
fn a_tangent_touches_the_rim_at_a_right_angle() {
    for (r1, s1, r2, s2) in [
        (0.0, 1, 1.0, 1),
        (0.0, 1, 1.0, -1),
        (0.5, 1, 1.5, 1),
        (0.5, -1, 1.5, 1),
        (0.5, 1, 1.5, -1),
        (0.5, -1, 1.5, -1),
    ] {
        let c1 = [-3.0, 0.5];
        let c2 = [2.0, 2.0];
        let t = tangent(c1, r1, s1, c2, r2, s2).expect("existe");
        near(
            (t.dir[0] * t.dir[0] + t.dir[1] * t.dir[1]).sqrt(),
            1.0,
            1.0e-5,
            "unitário",
        );
        for (p, c, r) in [(t.from, c1, r1), (t.to, c2, r2)] {
            let d = [p[0] - c[0], p[1] - c[1]];
            near((d[0] * d[0] + d[1] * d[1]).sqrt(), r, 1.0e-4, "no aro");
            near(
                d[0] * t.dir[0] + d[1] * t.dir[1],
                0.0,
                1.0e-4,
                "perpendicular",
            );
        }
        near(
            ((t.to[0] - t.from[0]).powi(2) + (t.to[1] - t.from[1]).powi(2)).sqrt(),
            t.len,
            1.0e-4,
            "comprimento",
        );
    }
}

/// **`side = +1` põe o centro à ESQUERDA da corda** — a convenção inteira, num
/// número.
///
/// Sem isto pinado, um sinal trocado lá dentro deixa a corda passando do lado
/// errado de toda roldana e ainda assim TANGENTE: o gate acima ficaria verde.
#[test]
fn the_side_says_which_hand_the_wheel_is_on() {
    for s in [1_i8, -1] {
        let t = tangent([0.0, 0.0], 0.0, 1, [0.0, 2.0], 1.0, s).expect("existe");
        // Do ponto de tangência para o centro.
        let to_centre = [-t.to[0], 2.0 - t.to[1]];
        // `perp(dir)` é o giro de +90°, ou seja a ESQUERDA de quem anda em `dir`.
        let left = [-t.dir[1], t.dir[0]];
        let side_of_centre = to_centre[0] * left[0] + to_centre[1] * left[1];
        assert!(
            (side_of_centre - f32::from(s)).abs() < 1.0e-4,
            "side {s}: o centro está a {side_of_centre} da esquerda da corda"
        );
    }
}

/// **Raio ZERO reduz EXATAMENTE ao modelo de ponto** — a âncora de regressão da
/// wave.
///
/// A polia que shipou trata cada roldana como um ponto, e as direções que ela
/// entrega ao kernel são `(âncora − roldana)` normalizado. Com `radius: 0.0` a
/// rota nova tem de devolver **os mesmos versores**, senão o geral quebrou o
/// particular.
///
/// ⚠️ **O COMPRIMENTO difere, e é correto:** a rota nova conta o trecho ENTRE as
/// duas roldanas, que o modelo de ponto ignorava — corda real, que ali existe. É
/// uma constante, absorvida pelo `L0` semeado da mesma rota, então o
/// comportamento não muda; o gate afirma exatamente essa diferença em vez de
/// escondê-la.
#[test]
fn a_zero_radius_wheel_is_the_point_model_to_the_versor() {
    let (a, b) = ([-2.0_f32, 0.0], [2.0_f32, 0.5]);
    let (wa, wb) = ([-2.0_f32, 3.0], [2.0_f32, 2.5]);
    let wheels = [
        RopeWheel {
            centre: wa,
            radius: 0.0,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: wb,
            radius: 0.0,
            side: -1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    let r = route(a, b, &wheels, &mut scratch).expect("rota");

    let unit = |from: [f32; 2], to: [f32; 2]| {
        let d = [to[0] - from[0], to[1] - from[1]];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
        [d[0] / l, d[1] / l]
    };
    let old_a = unit(wa, a);
    let old_b = unit(wb, b);
    near(r.dir_a[0], old_a[0], 1.0e-6, "dir A x");
    near(r.dir_a[1], old_a[1], 1.0e-6, "dir A y");
    near(r.dir_b[0], old_b[0], 1.0e-6, "dir B x");
    near(r.dir_b[1], old_b[1], 1.0e-6, "dir B y");

    let branch = |p: [f32; 2], w: [f32; 2]| ((p[0] - w[0]).powi(2) + (p[1] - w[1]).powi(2)).sqrt();
    let span = branch(wa, wb);
    near(
        r.length,
        branch(a, wa) + branch(b, wb) + span,
        1.0e-4,
        "o comprimento é o do modelo de ponto MAIS o vão entre as roldanas",
    );
}

/// **Uma roda que a corda abraça meia volta acrescenta `π·r` de corda.**
///
/// O caso canônico da polia: os dois corpos pendurados do MESMO lado, a corda
/// subindo de um, contornando o topo e descendo até o outro. É o número que
/// distingue *"a corda passa pela superfície"* de *"a corda passa pelo centro"*,
/// e ele é `π·r` — não uma fração que dependa de onde os corpos estão.
#[test]
fn a_half_wrap_adds_pi_r_of_rope() {
    let r = 0.5_f32;
    // Âncoras EM BAIXO, à esquerda e à direita do centro, alinhadas com os pontos
    // de tangência: a corda sobe reto, dá meia volta e desce reto.
    let c = [0.0_f32, 4.0];
    let (a, b) = ([-r, 0.0], [r, 0.0]);
    // ⚠️ `side: -1`, e eu escrevi `+1` na primeira versão deste gate: subindo
    // pela esquerda e descendo pela direita, a corda contorna o topo virando
    // para a DIREITA. O gate nasceu vermelho com 9,82 contra 9,57 — a rota tinha
    // pegado a tangente CRUZADA, que é mais longa e é a resposta certa para o
    // lado que eu havia pedido. É por isto que `resolve_sides` existe, e o
    // `assert_eq` abaixo confere que ele acha o mesmo lado sozinho.
    let wheels = [RopeWheel {
        centre: c,
        radius: r,
        side: -1,
        id: 0,
        break_force: f32::INFINITY,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    let mut guessed = wheels;
    resolve_sides(a, b, &mut guessed, &mut scratch);
    assert_eq!(guessed[0].side, -1, "o lado tinha de ser descoberto");
    let out = route(a, b, &wheels, &mut scratch).expect("rota");
    let straight = 2.0 * (c[1] - a[1]);
    near(
        out.length,
        straight + std::f32::consts::PI * r,
        1.0e-3,
        "os dois trechos retos mais meia circunferência",
    );
    // E as duas pontas puxam para BAIXO, cada corpo para o seu lado da roda.
    assert!(out.dir_a[1] < -0.99 && out.dir_b[1] < -0.99, "{out:?}");
}

/// **Uma roda que a corda mal desvia quase não acrescenta arco** — o contra-teste
/// do de cima.
///
/// Sem ele, um bug que somasse `π·r` em TODA roda passaria pelo irmão acima.
#[test]
fn a_wheel_the_rope_barely_grazes_adds_almost_nothing() {
    let r = 0.5_f32;
    // O centro fica ACIMA da corda que anda para leste, e acima é a ESQUERDA de
    // quem anda para leste ⇒ `side: +1`. (Mesma armadilha do gate irmão: a
    // primeira versão pedia `-1` e a rota devolvia a tangente cruzada, 20,10.)
    let wheels = [RopeWheel {
        centre: [0.0, r],
        radius: r,
        side: 1,
        id: 0,
        break_force: f32::INFINITY,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    let out = route([-10.0, 0.0], [10.0, 0.0], &wheels, &mut scratch).expect("rota");
    near(out.length, 20.0, 0.01, "quase a reta inteira");
}

/// **A rota RECUSA o degenerado** em vez de devolver `NaN`.
///
/// Uma âncora DENTRO da roldana não tem tangente — e o `NaN` que a fórmula
/// produziria envenenaria a pose e o `physics_ecs_c9`. A recusa é a mesma do
/// modelo de ponto (pular a corda inteira), pelo mesmo motivo: meia rota é pior
/// que nenhuma.
#[test]
fn a_route_through_an_impossible_tangent_is_refused() {
    let wheels = [RopeWheel {
        centre: [0.0, 0.0],
        radius: 1.0,
        side: 1,
        id: 0,
        break_force: f32::INFINITY,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    // A âncora está DENTRO da roda.
    assert!(route([0.2, 0.0], [5.0, 0.0], &wheels, &mut scratch).is_none());
    // E duas rodas que se sobrepõem não têm tangente cruzada.
    let pair = [
        RopeWheel {
            centre: [0.0, 0.0],
            radius: 1.0,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.5, 0.0],
            radius: 1.0,
            side: -1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
    ];
    assert!(route([-5.0, 0.0], [5.0, 0.0], &pair, &mut scratch).is_none());
}

/// **O lado é DESCOBERTO da posição das roldanas** — o (7) do pedido.
///
/// Uma roda ACIMA da linha que liga as duas âncoras faz a corda virar para um
/// lado; a mesma roda ABAIXO, para o outro. O algoritmo não pode precisar que o
/// artista diga qual.
#[test]
fn the_side_is_discovered_from_where_the_wheel_sits() {
    let mut scratch = Vec::new();
    for (y, expected) in [(3.0_f32, -1_i8), (-3.0, 1)] {
        let mut wheels = [RopeWheel {
            centre: [0.0, y],
            radius: 0.4,
            // Chute deliberadamente ERRADO nos dois casos, para o gate medir a
            // descoberta e não o valor que entrou.
            side: if expected > 0 { -1 } else { 1 },
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        }];
        resolve_sides([-4.0, 0.0], [4.0, 0.0], &mut wheels, &mut scratch);
        assert_eq!(
            wheels[0].side, expected,
            "roda em y={y} tinha de resolver para {expected}"
        );
    }
}

/// **E o lado descoberto é o que a rota de fato faz** — o fecho do ponto fixo.
///
/// O gate acima compara com um número que eu escrevi. Este não: ele pergunta à
/// GEOMETRIA — depois de resolver, o sentido de giro que os dois trechos fazem em
/// cada roda tem de bater com o `side` guardado. É o que torna o resultado um
/// ponto fixo em vez de um chute que ninguém reconferiu.
#[test]
fn the_resolved_sides_agree_with_the_route_they_produce() {
    let mut scratch = Vec::new();
    let mut wheels = [
        RopeWheel {
            centre: [-2.0, 3.0],
            radius: 0.5,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.0, 4.5],
            radius: 0.3,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [2.5, 2.0],
            radius: 0.7,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
    ];
    let (a, b) = ([-4.0_f32, 0.0], [4.0_f32, 0.0]);
    resolve_sides(a, b, &mut wheels, &mut scratch);
    route(a, b, &wheels, &mut scratch).expect("rota");
    for (i, w) in wheels.iter().enumerate() {
        let (u_in, u_out) = (scratch[i].dir, scratch[i + 1].dir);
        let cross = u_in[0] * u_out[1] - u_in[1] * u_out[0];
        assert!(
            cross.abs() < 1.0e-6 || (cross > 0.0) == (w.side > 0),
            "roda {i}: side {} contra giro {cross}",
            w.side
        );
    }
}

/// **O arco desdobra além de meia volta.**
///
/// `atan2` sozinho devolve `(−π, π]`, então um enlace de 270° mediria −90° e a
/// corda encurtaria em vez de crescer. O `side` é quem diz qual dos dois sentidos
/// é o real.
#[test]
fn a_wrap_past_half_a_turn_measures_the_long_way_round() {
    // Entra para o leste, sai para o sul: giro de −90° pela direita, ou +270°
    // pela esquerda. O `side` decide.
    let (u_in, u_out) = ([1.0_f32, 0.0], [0.0_f32, -1.0]);
    near(
        turn_angle(u_in, u_out, -1),
        -std::f32::consts::FRAC_PI_2,
        1.0e-5,
        "pela direita",
    );
    near(
        turn_angle(u_in, u_out, 1),
        3.0 * std::f32::consts::FRAC_PI_2,
        1.0e-5,
        "pela esquerda",
    );
}
