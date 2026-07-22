//! Gates da correspondência (`tween_match`).
//!
//! O oráculo de cada um é uma propriedade da APARÊNCIA ou uma verdade independente da
//! implementação (a atribuição ótima é comparada com a **enumeração exaustiva**, não com
//! ela mesma) — a regra 10 do plano.

use super::*;
use crate::stroke::Point;
use ph2d_core::Vec2;

fn stroke(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_default(Vec2::new(x, y));
    }
    s
}

fn closed(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = stroke(pts);
    s.closed = true;
    s
}

fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

/// Um segmento reto de `a` a `b` com `n` pontos (densidade controlada).
fn dense_line(a: (f32, f32), b: (f32, f32), n: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        s.push_point(Point::at(Vec2::new(
            a.0 + (b.0 - a.0) * t,
            a.1 + (b.1 - a.1) * t,
        )));
    }
    s
}

/// Um polígono regular de `n` lados, raio `r`, centrado em `c` — o círculo dos fixtures.
fn polygon(c: (f32, f32), r: f32, n: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        s.push_point(Point::at(Vec2::new(c.0 + r * a.cos(), c.1 + r * a.sin())));
    }
    s.closed = true;
    s
}

// ── as features ──────────────────────────────────────────────────────────────

/// **A lição que o `ph2d-vec-blend` pagou:** picar uma aresta em 20 pedaços não muda a
/// geometria, então não pode mudar a correspondência. Média de vértice falharia aqui
/// (o lado subdividido puxaria o centróide); a integral de arco não.
#[test]
fn features_are_a_fact_of_the_shape_not_of_the_point_density() {
    // Um "L": a perna horizontal densa num caso, esparsa no outro.
    let sparse = {
        let mut s = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        for p in dense_line((10.0, 0.0), (10.0, 10.0), 2)
            .positions()
            .iter()
            .skip(1)
        {
            s.push_point(Point::at(*p));
        }
        s
    };
    let dense = {
        let mut s = dense_line((0.0, 0.0), (10.0, 0.0), 21);
        for p in dense_line((10.0, 0.0), (10.0, 10.0), 2)
            .positions()
            .iter()
            .skip(1)
        {
            s.push_point(Point::at(*p));
        }
        s
    };
    let (fa, fb) = (features(&sparse), features(&dense));
    assert!(
        (fa.centroid - fb.centroid).length() < 1e-3,
        "centróide mudou com a densidade: {:?} vs {:?}",
        fa.centroid,
        fb.centroid
    );
    assert!(
        (fa.arclen - fb.arclen).abs() < 1e-3,
        "arco mudou: {} vs {}",
        fa.arclen,
        fb.arclen
    );
    let (ua, ub) = (fa.axis.expect("L tem eixo"), fb.axis.expect("L tem eixo"));
    assert!(
        1.0 - (ua.x * ub.x + ua.y * ub.y).abs() < 1e-3,
        "eixo girou com a densidade: {ua:?} vs {ub:?}"
    );
}

/// Um polígono quase-regular NÃO tem eixo (é isotrópico); uma reta tem, e é a reta dela.
#[test]
fn an_isotropic_shape_has_no_principal_axis_and_a_line_has_its_own() {
    assert_eq!(features(&polygon((0.0, 0.0), 10.0, 24)).axis, None);
    let u = features(&stroke(&[(0.0, 0.0), (10.0, 0.0)]))
        .axis
        .expect("uma reta tem eixo");
    assert!(u.y.abs() < 1e-4, "o eixo da horizontal é horizontal: {u:?}");
}

// ── o custo ──────────────────────────────────────────────────────────────────

/// **O termo indisponível é OMITIDO, não zerado.**
///
/// ⚠️ A 1ª versão deste gate comparava retas × círculos DESENHADOS e ficou vermelha sobre
/// código correto: os dois fixtures diferiam também em arco e em bbox (logo em régua), então
/// ele não continha o fenômeno que media — media três coisas ao mesmo tempo. A pergunta é
/// sobre a FUNÇÃO DE CUSTO, então o fixture é feito na função de custo: dois pares idênticos
/// em tudo, um com eixo e outro sem.
///
/// Com o eixo presente e CONCORDANTE, o termo vale zero e **dilui** a média (o custo cai);
/// ausente, ele sai da conta e a média sobe. Zerar o ausente faria os dois custarem igual —
/// é a mutação que este gate mata.
#[test]
fn a_missing_term_is_omitted_from_the_average_never_counted_as_zero() {
    let base = |axis: Option<Vec2>, cx: f32| StrokeFeatures {
        centroid: Vec2::new(cx, 0.0),
        arclen: 20.0,
        axis,
        closed: false,
        lo: Vec2::new(cx - 10.0, -10.0),
        hi: Vec2::new(cx + 10.0, 10.0),
    };
    let x = Some(Vec2::new(1.0, 0.0));
    let with = TweenPlan::from_features(&[base(x, 0.0)], &[base(x, 6.0)]);
    let without = TweenPlan::from_features(&[base(None, 0.0)], &[base(None, 6.0)]);
    let (cw, co) = (
        with.cost_of_a(0).expect("par"),
        without.cost_of_a(0).expect("par"),
    );
    assert!(
        co > cw + 1e-4,
        "o par SEM eixo tinha de custar mais (o termo ausente não dilui a média): com \
         {cw:.4} × sem {co:.4}"
    );
}

/// Aberto × fechado é incompatibilidade dura: um contorno não vira uma linha, nem por
/// eliminação (é o que o custo `BLOCKED` + o limiar garantem juntos).
#[test]
fn an_open_stroke_never_pairs_with_a_closed_one() {
    let plan = TweenPlan::build(
        &drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]),
        &drawing(vec![closed(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)])]),
    );
    assert_eq!(plan.pair_of_a(0), None, "aberto casou com fechado");
    assert_eq!(plan.pairs(), 0);
}

// ── a correspondência ────────────────────────────────────────────────────────

/// **A razão de existir do Tween v2.** O artista redesenhou o mesmo personagem no quadro
/// B começando pela perna em vez do braço. O pareamento por ÍNDICE (o do GP) casaria braço
/// com perna e o inbetween seria um borrão atravessando o corpo; a correspondência
/// espacial casa forma com forma.
#[test]
fn strokes_drawn_in_a_different_order_still_pair_by_shape() {
    let arm = |y: f32| stroke(&[(0.0, y), (30.0, y)]);
    let leg = |y: f32| stroke(&[(0.0, y), (0.0, y - 40.0)]);
    let a = drawing(vec![arm(100.0), leg(0.0)]);
    // B: mesma pose, um passo adiante — e desenhado na ordem TROCADA.
    let b = drawing(vec![leg(2.0), arm(104.0)]);

    let plan = TweenPlan::build(&a, &b);
    assert_eq!(
        plan.pair_of_a(0),
        Some(1),
        "o braço tem de casar com o braço"
    );
    assert_eq!(plan.pair_of_a(1), Some(0), "a perna, com a perna");
}

/// O par ordinal é o DESEMPATE, não a resposta: com duas cópias indistinguíveis, a ordem
/// de desenho decide — e é isso que faz o v2 devolver o v1 quando não há informação nova.
#[test]
fn identical_shapes_fall_back_to_the_drawing_order() {
    let bar = |x: f32| stroke(&[(x, 0.0), (x + 10.0, 0.0)]);
    let plan = TweenPlan::build(
        &drawing(vec![bar(0.0), bar(100.0)]),
        &drawing(vec![bar(0.0), bar(100.0)]),
    );
    assert_eq!(plan.pair_of_a(0), Some(0));
    assert_eq!(plan.pair_of_a(1), Some(1));
}

/// **Sem par é ÓRFÃO, não par forçado.** O solver casa todo mundo que puder (é uma
/// atribuição); quem decide se o par SIGNIFICA algo é o limiar. Sem ele, o traço que
/// sobra de A é casado com o que sobra de B por eliminação — e o inbetween mostra um
/// braço virando um pé do outro lado da tela.
#[test]
fn a_stroke_with_no_counterpart_becomes_an_orphan_instead_of_a_forced_pair() {
    let a = drawing(vec![
        stroke(&[(0.0, 0.0), (30.0, 0.0)]),
        stroke(&[(0.0, 400.0), (5.0, 405.0)]), // some no quadro B
    ]);
    let b = drawing(vec![stroke(&[(2.0, 0.0), (32.0, 0.0)])]);
    let plan = TweenPlan::build(&a, &b);
    assert_eq!(plan.pair_of_a(0), Some(0));
    assert_eq!(
        plan.pair_of_a(1),
        None,
        "o traço que some virou par forçado"
    );
}

/// O caso mais simples que existe — UM traço que anda — tem de casar.
///
/// ⚠️ É o gate que pinou a régua: com a diagonal tirada dos CENTRÓIDES ela seria igual ao
/// próprio deslocamento, o termo saturaria em 1.0 para qualquer movimento e este par
/// seria recusado. A régua é o bbox dos PONTOS.
#[test]
fn one_stroke_that_moves_is_still_the_same_stroke() {
    for dx in [1.0, 5.0, 20.0, 60.0] {
        let plan = TweenPlan::build(
            &drawing(vec![stroke(&[(0.0, 0.0), (100.0, 0.0)])]),
            &drawing(vec![stroke(&[(dx, 0.0), (100.0 + dx, 0.0)])]),
        );
        assert_eq!(
            plan.pair_of_a(0),
            Some(0),
            "recusou um traço que só andou {dx}"
        );
    }
}

// ── o solver de atribuição ───────────────────────────────────────────────────

/// **A atribuição é ÓTIMA** — comparada com a enumeração EXAUSTIVA de permutações, que é
/// o único oráculo honesto para um solver de atribuição (um espelho da implementação
/// provaria só que ela é igual a si mesma).
///
/// Guloso falharia na 1ª matriz: ele agarra o mínimo global da tabela e fica preso com a
/// sobra cara.
#[test]
fn the_assignment_is_optimal_not_greedy() {
    // O guloso pega (0,0)=1 — o menor da tabela — e herda (1,1)=100: total 101. O ótimo
    // recusa esse mínimo local e paga 2+2 = 4.
    let costs = [
        1.0, 2.0, //
        2.0, 100.0,
    ];
    let got: f32 = assign(&costs, 2, 2)
        .iter()
        .map(|&(i, j)| costs[i * 2 + j])
        .sum();
    assert!(
        (got - 4.0).abs() < 1e-6,
        "total {got} != 4 (ótimo); guloso daria 101"
    );

    // E a varredura: para todo n ≤ 5, o total do solver bate o mínimo sobre TODAS as
    // permutações. A matriz vem de um LCG determinístico (HR-5: sem `thread_rng`).
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((seed >> 33) % 1000) as f32 / 100.0
    };
    for n in 1..=5usize {
        for _ in 0..20 {
            let costs: Vec<f32> = (0..n * n).map(|_| next()).collect();
            let got: f32 = assign(&costs, n, n)
                .iter()
                .map(|&(i, j)| costs[i * n + j])
                .sum();
            let best = brute_force(&costs, n);
            assert!(
                (got - best).abs() < 1e-4,
                "n={n}: solver {got:.4} × exaustivo {best:.4}"
            );
        }
    }
}

/// O mínimo sobre todas as permutações (Heap, iterativo) — o oráculo.
fn brute_force(costs: &[f32], n: usize) -> f32 {
    let mut perm: Vec<usize> = (0..n).collect();
    let mut c = vec![0usize; n];
    let total = |p: &[usize]| -> f32 { p.iter().enumerate().map(|(i, &j)| costs[i * n + j]).sum() };
    let mut best = total(&perm);
    let mut i = 0;
    while i < n {
        if c[i] < i {
            perm.swap(if i % 2 == 0 { 0 } else { c[i] }, i);
            best = best.min(total(&perm));
            c[i] += 1;
            i = 0;
        } else {
            c[i] = 0;
            i += 1;
        }
    }
    best
}

/// Retangular: com mais traços em B do que em A, cada traço de A recebe UM par e nenhum
/// é reusado — e o caminho transposto (mais em A) devolve os mesmos pares.
#[test]
fn a_rectangular_assignment_pairs_every_row_once_in_both_orientations() {
    let costs = [
        1.0, 5.0, 9.0, //
        4.0, 2.0, 8.0,
    ];
    let wide = assign(&costs, 2, 3);
    assert_eq!(wide, vec![(0, 0), (1, 1)]);

    // O transposto (3 linhas, 2 colunas) tem de casar as MESMAS duas células.
    let mut tall = vec![0.0f32; 6];
    for i in 0..2 {
        for j in 0..3 {
            tall[j * 2 + i] = costs[i * 3 + j];
        }
    }
    let got = assign(&tall, 3, 2);
    assert_eq!(got, vec![(0, 0), (1, 1)], "o caminho transposto divergiu");
}

/// **O limiar, cercado dos DOIS lados pelos fixtures que a régua mediu.**
///
/// A régua imprime a tabela; este gate a pina. Baixar `PAIR_REJECT_COST` mata a 1ª
/// asserção (o braço que gira e encolhe vira dois órfãos, e o inbetween pisca); subir mata
/// a 2ª (o braço casa com a perna e o inbetween atravessa o corpo).
#[test]
fn the_threshold_accepts_the_worst_real_pose_change_and_refuses_another_limb() {
    let arm = |x: f32, y: f32| stroke(&[(x, y), (x + 60.0, y)]);
    let leg = |x: f32, y: f32| stroke(&[(x, y), (x, y - 80.0)]);
    let torso = |x: f32| stroke(&[(x, 0.0), (x, 100.0)]);
    let scene = |first: FlipStroke| drawing(vec![first, torso(0.0), leg(-40.0, 0.0)]);

    // gira 90° E encolhe 30% — o pior par LEGÍTIMO medido (0.3352).
    let hard = TweenPlan::build(
        &scene(arm(0.0, 100.0)),
        &scene(stroke(&[(0.0, 100.0), (0.0, 142.0)])),
    );
    assert_eq!(
        hard.pair_of_a(0),
        Some(0),
        "o braço que gira 90° e encolhe 30% virou órfão (custo {:?})",
        hard.cost_of_a(0)
    );

    // braço × perna — o melhor par claramente ALHEIO medido (0.4261).
    let bogus = TweenPlan::build(&scene(arm(0.0, 100.0)), &scene(leg(120.0, 100.0)));
    assert_ne!(
        bogus.pair_of_a(0),
        Some(0),
        "o braço casou com a perna do outro lado da cena"
    );
}

// ── as RÉGUAS (medem os números que viraram constante) ───────────────────────

/// **A régua do limiar** (`PAIR_REJECT_COST`) e do piso de anisotropia
/// (`AXIS_MIN_ANISOTROPY`): imprime o custo dos pares LEGÍTIMOS (o que um inbetween de
/// verdade parece) contra o dos ESPÚRIOS (o que o limiar tem de recusar). O número da
/// constante sai do VÃO entre as duas colunas — não do doc de referência.
///
/// `cargo test -p ph2d-flip --release the_cost_ruler -- --ignored --nocapture`
#[test]
#[ignore = "régua: imprime a tabela que justifica as constantes"]
fn the_cost_ruler() {
    // A cena de referência: um "personagem" de 3 traços num quadro de ~200 unidades.
    let arm = |x: f32, y: f32| stroke(&[(x, y), (x + 60.0, y)]);
    let leg = |x: f32, y: f32| stroke(&[(x, y), (x, y - 80.0)]);
    let torso = |x: f32| stroke(&[(x, 0.0), (x, 100.0)]);

    let cost1 = |a: FlipStroke, b: FlipStroke| -> f32 {
        // Os traços de contexto dão a ESCALA da cena (o denominador do termo de centróide).
        let da = drawing(vec![a, torso(0.0), leg(-40.0, 0.0)]);
        let db = drawing(vec![b, torso(0.0), leg(-40.0, 0.0)]);
        let fa: Vec<StrokeFeatures> = da.strokes.iter().map(features).collect();
        let fb: Vec<StrokeFeatures> = db.strokes.iter().map(features).collect();
        let ctx = CostCtx {
            diag: union_diag(&fa, &fb),
            order_span: (fa.len().max(fb.len()) - 1) as f32,
        };
        pair_cost(&fa[0], &fb[0], 0, 0, ctx)
    };

    println!("\n== PARES LEGÍTIMOS (o limiar tem de ACEITAR) ==");
    let mut worst_ok: f32 = 0.0;
    for (name, a, b) in [
        ("parado", arm(0.0, 100.0), arm(0.0, 100.0)),
        ("anda 5", arm(0.0, 100.0), arm(5.0, 100.0)),
        ("anda 20", arm(0.0, 100.0), arm(20.0, 100.0)),
        ("anda 60", arm(0.0, 100.0), arm(60.0, 100.0)),
        (
            "gira 45",
            arm(0.0, 100.0),
            stroke(&[(0.0, 100.0), (42.4, 142.4)]),
        ),
        (
            "gira 90",
            arm(0.0, 100.0),
            stroke(&[(0.0, 100.0), (0.0, 160.0)]),
        ),
        (
            "gira 90 + encolhe 30%",
            arm(0.0, 100.0),
            stroke(&[(0.0, 100.0), (0.0, 142.0)]),
        ),
        ("encolhe 50%", arm(0.0, 100.0), arm(0.0, 100.0).clone()),
    ] {
        let c = cost1(a, b);
        worst_ok = worst_ok.max(c);
        println!("  {name:24} {c:.4}");
    }

    println!("== PARES ESPÚRIOS (o limiar tem de RECUSAR) ==");
    let mut best_bad = f32::INFINITY;
    for (name, a, b) in [
        ("braço × perna", arm(0.0, 100.0), leg(120.0, 100.0)),
        (
            "braço × traço no canto",
            arm(0.0, 100.0),
            stroke(&[(180.0, -80.0), (200.0, -60.0)]),
        ),
        (
            "braço × cotoco",
            arm(0.0, 100.0),
            stroke(&[(0.0, 100.0), (6.0, 100.0)]),
        ),
    ] {
        let c = cost1(a, b);
        best_bad = best_bad.min(c);
        println!("  {name:24} {c:.4}");
    }
    println!(
        "\n  pior LEGÍTIMO {worst_ok:.4}  |  melhor ESPÚRIO {best_bad:.4}  |  \
         constante {PAIR_REJECT_COST}\n"
    );

    println!("== ANISOTROPIA (o piso do eixo principal) ==");
    let aniso = |s: &FlipStroke| -> f32 {
        // Reconstrói a razão que o `principal_axis` usa, para a tabela ser sobre o MESMO
        // número que a constante corta.
        let f = features(s);
        let (mut len, mut m1, mut mxx, mut mxy, mut myy) = (0.0f32, Vec2::ZERO, 0.0, 0.0, 0.0);
        for (_, p, q) in s.segments() {
            let d = q - p;
            let l = (d.x * d.x + d.y * d.y).sqrt();
            if l <= 0.0 {
                continue;
            }
            len += l;
            m1 += (p + q) * (0.5 * l);
            mxx += l * (p.x * p.x + p.x * d.x + d.x * d.x / 3.0);
            mxy += l * (p.x * p.y + 0.5 * (p.x * d.y + d.x * p.y) + d.x * d.y / 3.0);
            myy += l * (p.y * p.y + p.y * d.y + d.y * d.y / 3.0);
        }
        let c = m1 / len;
        let (cxx, cxy, cyy) = (
            mxx / len - c.x * c.x,
            mxy / len - c.x * c.y,
            myy / len - c.y * c.y,
        );
        let _ = f;
        let tr = cxx + cyy;
        (((cxx - cyy) * (cxx - cyy) + 4.0 * cxy * cxy).sqrt()) / tr
    };
    let ellipse = |rx: f32, ry: f32| -> FlipStroke {
        let mut s = FlipStroke::new();
        for i in 0..48 {
            let a = std::f32::consts::TAU * i as f32 / 48.0;
            s.push_point(Point::at(Vec2::new(rx * a.cos(), ry * a.sin())));
        }
        s.closed = true;
        s
    };
    let wobbly = {
        let mut s = FlipStroke::new();
        for i in 0..21 {
            let x = i as f32 * 5.0;
            s.push_point(Point::at(Vec2::new(x, (i % 3) as f32 - 1.0)));
        }
        s
    };
    for (name, s) in [
        ("círculo (48-gon)", ellipse(10.0, 10.0)),
        ("elipse 1.05:1", ellipse(10.5, 10.0)),
        ("elipse 1.1:1", ellipse(11.0, 10.0)),
        ("elipse 1.3:1", ellipse(13.0, 10.0)),
        ("elipse 2:1", ellipse(20.0, 10.0)),
        ("reta com tremor", wobbly),
    ] {
        println!("  {name:24} {:.4}", aniso(&s));
    }
    println!("\n  piso da constante: {AXIS_MIN_ANISOTROPY}\n");
}

/// Desenho vazio de um lado: zero pares, nenhum pânico.
#[test]
fn an_empty_drawing_yields_an_empty_plan() {
    let plan = TweenPlan::build(&drawing(vec![]), &drawing(vec![stroke(&[(0.0, 0.0)])]));
    assert_eq!(plan.pairs(), 0);
    assert_eq!(plan.pair_of_b(0), None);
}
