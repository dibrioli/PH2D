//! Gates do raio de quina vivo ([`crate::corner_live`]).
//!
//! O que estes testes protegem, em ordem de importância:
//!
//! 1. **A identidade é sagrada.** Sem raio, a saída é a fonte — byte a byte. É o que
//!    permite ter ligado o cozimento em TODO consumidor de geometria do módulo (render,
//!    hit-test, bbox, booleana) sem mexer numa vírgula do comportamento de hoje.
//! 2. **O caso reto continua sendo o arco exato de sempre.** A generalização para curvas
//!    não pode ter mexido no que o `crate::corners` já fazia.
//! 3. **A alça não mente.** O que o gizmo mostra é onde o cozimento arredonda — a mesma
//!    função nas duas direções. É o gate mais importante daqui, e o único que fica
//!    vermelho quando as duas contas divergem.
//! 4. **A saturação não estoura, e nada vira `NaN`.** Um pânico morre com stack trace; um
//!    `NaN` contamina a bbox em silêncio, o gizmo some, e o usuário reporta "a forma
//!    sumiu" três telas depois da causa (Bug #1, `docs/Vector Module/BUGS_vector.md`).
//!
//! **Sobre o que cada gate NÃO prova** — porque eu escrevi o gate errado primeiro e vale
//! deixar registrado: a varredura combinada de quinas vizinhas prova finitude e
//! contenção, mas **não morde** o guard de ordem nem o clamp de meia-corda (confirmei
//! removendo os dois, um de cada vez: ela seguiu verde). O estrago que eles evitam é
//! sub-ulp ou satura em outro lugar — invisível para uma asserção de caixa. Cada um tem o
//! gate próprio, e os três foram **provados por mutação**, não por alegação.

use super::*;
use crate::{VecPath, VecVertex, VertexKind};

/// Quadrado de lado 10, quinas retas, sem raio.
fn square() -> Vec<VecVertex> {
    [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec()
}

/// Todo ponto de controle de todo vértice é finito?
fn all_finite(verts: &[VecVertex]) -> bool {
    verts.iter().all(|v| {
        [v.anchor, v.in_handle, v.out_handle]
            .iter()
            .all(|p| p[0].is_finite() && p[1].is_finite())
    })
}

// ---------------------------------------------------------------- 1. a identidade

/// **Sem raio nenhum, o cozimento NÃO ACONTECE.** Não é "produz a mesma coisa" — é
/// `Cow::Borrowed`, o mesmo ponteiro. Sem esta propriedade, ligar o `cooked()` em todo
/// consumidor teria feito a geometria de todo path do projeto passar por uma reconstrução
/// em ponto flutuante, e os goldens do catálogo inteiro teriam derivado.
#[test]
fn without_a_radius_the_cook_is_not_merely_equal_but_the_very_same_path() {
    let path = VecPath {
        verts: square(),
        closed: true,
        ..VecPath::default()
    };
    let cooked = path.cooked();
    assert!(
        matches!(cooked, std::borrow::Cow::Borrowed(_)),
        "sem raio o cozimento tem de ser um EMPRÉSTIMO — zero alocação, zero aritmética"
    );
    assert!(std::ptr::eq(&*cooked, &path), "e o mesmo ponteiro");
    assert!(round_authored_corners(&square(), true).is_none());
}

/// Um vértice `Smooth`/`Symmetric` tem handles colineares por definição — não é quina, e
/// o cozimento o pula **sozinho**, sem consultar o `kind`. É o que faz o raio morder só
/// onde há quina de verdade: a geometria decide, não uma flag.
#[test]
fn a_smooth_vertex_is_not_a_corner_so_its_radius_is_inert() {
    // Losango com o vértice do topo SUAVE (handles colineares horizontais).
    let mut verts = vec![
        VecVertex::corner([0.0, -10.0]),
        VecVertex::corner([10.0, 0.0]),
        VecVertex::smooth([0.0, 10.0], [4.0, 10.0], [-4.0, 10.0]),
        VecVertex::corner([-10.0, 0.0]),
    ];
    verts[2].corner_radius = 3.0; // pedido, mas não há quina onde aplicar
    let out = round_authored_corners(&verts, true).expect("outra quina existe? não");
    // Nenhuma quina foi dividida (o raio do suave é inerte, os demais são zero).
    assert_eq!(out.len(), 4, "o vértice suave NÃO virou dois");
    assert_eq!(out[2].anchor, [0.0, 10.0], "e ficou onde estava");
}

// ------------------------------------------- 2. o caso reto = o arco exato de sempre

/// **A generalização para curvas não mexeu no caso reto.** O raio numa quina entre duas
/// RETAS tem de produzir exatamente o mesmo arco que o `crate::corners` já produzia — o
/// recuo `t = r/tan(θ/2)`, o handle a `KAPPA·r` do ponto de recuo, apontando para a quina
/// original. Se isto quebrar, polígono, estrela e round-rect mudam de forma.
#[test]
fn a_right_angle_between_two_straight_edges_is_still_the_canonical_quarter_circle() {
    let mut verts = square();
    verts[0].corner_radius = 2.0;
    let out = round_authored_corners(&verts, true).unwrap();
    assert_eq!(out.len(), 5, "só a quina 0 virou dois vértices");
    // A quina (0,0): chega vindo de (0,10) → recua em +Y; sai para (10,0) → +X.
    let (v_in, v_out) = (&out[0], &out[1]);
    assert!((v_in.anchor[0]).abs() < 1e-9 && (v_in.anchor[1] - 2.0).abs() < 1e-9);
    assert!((v_out.anchor[0] - 2.0).abs() < 1e-9 && (v_out.anchor[1]).abs() < 1e-9);
    // O handle do arco: a KAPPA·r da âncora, apontando para a quina original (0,0).
    let want = 2.0 - 2.0 * crate::corners::KAPPA;
    assert!(
        (v_in.out_handle[1] - want).abs() < 1e-6,
        "handle do arco: {:?}, queria y≈{want}",
        v_in.out_handle
    );
    assert!((v_out.in_handle[0] - want).abs() < 1e-6);
    // E o cozido não carrega raio: o arredondamento já aconteceu.
    assert!(out.iter().all(|v| v.corner_radius == 0.0));
}

/// O cozimento concorda com o **outro motor** (`corners::round_closed_corners`, o gerador
/// das Live Shapes) num contorno de arestas retas: as duas construções são a MESMA conta,
/// e um dia que divirjam é um dia em que o polígono do catálogo e um polígono desenhado
/// à mão arredondam diferente — com o mesmo número na caixa.
#[test]
fn the_straight_case_agrees_with_the_live_shape_engine_that_already_existed() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    let radii = [2.0, 1.0, 3.0, 0.5];
    let theirs = crate::corners::round_closed_corners(&pts, &radii);

    let mut verts: Vec<VecVertex> = pts.map(VecVertex::corner).to_vec();
    for (v, r) in verts.iter_mut().zip(radii) {
        v.corner_radius = r;
    }
    let mine = round_authored_corners(&verts, true).unwrap();

    assert_eq!(mine.len(), theirs.verts.len(), "mesma contagem de vértices");
    for (i, (a, b)) in mine.iter().zip(&theirs.verts).enumerate() {
        for (p, q, what) in [
            (a.anchor, b.anchor, "âncora"),
            (a.in_handle, b.in_handle, "in"),
            (a.out_handle, b.out_handle, "out"),
        ] {
            assert!(
                (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9,
                "vértice {i} ({what}): {p:?} != {q:?} — os dois motores divergiram"
            );
        }
    }
}

/// O raio 0 numa quina **não a divide**, mesmo com as vizinhas arredondando. (Se dividisse,
/// a contagem de vértices do cozido mudaria à toa e a booleana veria pontos de controle
/// que ninguém pediu — foi exatamente esse o bug que o `to_bez` da booleana já contorna.)
#[test]
fn a_zero_radius_corner_is_not_split_even_when_its_neighbours_round() {
    let mut verts = square();
    verts[0].corner_radius = 2.0;
    verts[2].corner_radius = 2.0;
    let out = round_authored_corners(&verts, true).unwrap();
    assert_eq!(out.len(), 6, "2 quinas × 2 + 2 quinas cruas");
    assert_eq!(out[2].anchor, [10.0, 0.0], "a quina 1 ficou crua, no lugar");
    assert_eq!(out[5].anchor, [0.0, 10.0], "e a quina 3 também");
}

// ------------------------------------------------------------ 3. a quina entre CURVAS

/// **O gap que o handoff marcou como aberto: arredondar entre duas CURVAS.** Numa quina
/// onde os dois lados são cúbicos de verdade, o arredondamento tem de (a) acontecer,
/// (b) recuar POR CIMA da curva (não da corda), e (c) sair TANGENTE aos dois lados — que
/// é o que separa um filete de um corte.
#[test]
fn a_corner_between_two_real_curves_rounds_and_leaves_tangent_to_both_sides() {
    // "Folha": um bico afiado no topo (90°, abrindo para BAIXO), com os dois lados
    // curvos de verdade. Os handles do bico ficam ABAIXO da âncora — é o que faz dele
    // uma quina, e não um vértice suave com barriga.
    let verts = vec![
        VecVertex {
            // O BICO: chega curvando pela esquerda (de baixo), sai curvando pela direita.
            anchor: [0.0, 10.0],
            in_handle: [-3.0, 7.0],
            out_handle: [3.0, 7.0],
            kind: VertexKind::Corner,
            corner_radius: 1.5,
        },
        VecVertex {
            anchor: [5.0, 0.0],
            in_handle: [5.0, 5.0],
            out_handle: [3.0, -3.0],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [-5.0, 0.0],
            in_handle: [-3.0, -3.0],
            out_handle: [-5.0, 5.0],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
    ];
    let out = round_authored_corners(&verts, true).expect("o bico é curva-curva");
    assert!(all_finite(&out));
    assert_eq!(out.len(), 4, "o bico virou dois vértices");

    // (a) A quina afiada SUMIU: nenhum vértice ficou no ápice original.
    assert!(
        out.iter().all(|v| v.anchor != [0.0, 10.0]),
        "o ápice afiado devia ter sido recuado"
    );
    // (b) Os dois pontos de recuo caíram ABAIXO do ápice, um de cada lado. (O bico é o
    // vértice 0, então os dois que o substituem são `out[0]` e `out[1]`.)
    let (a, b) = (&out[0], &out[1]);
    assert!(
        a.anchor[1] < 10.0 && b.anchor[1] < 10.0,
        "recuaram pra dentro"
    );
    assert!(a.anchor[0] < 0.0 && b.anchor[0] > 0.0, "um de cada lado");
    // (c) TANGENTE: o handle de saída de `a` e o de entrada de `b` são os do arco, e o
    // arco tem de sair de `a` na direção em que a curva CHEGAVA nele. O handle de entrada
    // de `a` (que veio do pedaço da curva original) e o de saída dele (o arco) têm de ser
    // colineares com a âncora — senão o filete faz uma dobra visível no ponto de emenda.
    let d_in = [a.anchor[0] - a.in_handle[0], a.anchor[1] - a.in_handle[1]];
    let d_out = [a.out_handle[0] - a.anchor[0], a.out_handle[1] - a.anchor[1]];
    let cross = d_in[0] * d_out[1] - d_in[1] * d_out[0];
    let norm = d_in[0].hypot(d_in[1]) * d_out[0].hypot(d_out[1]);
    assert!(
        (cross / norm).abs() < 1e-6,
        "a emenda curva→arco tem de ser TANGENTE (sen do ângulo = {})",
        cross / norm
    );
}

/// O recuo numa curva é medido **sobre a curva**, não sobre a corda: um lado muito
/// encurvado recua o mesmo TANTO que um lado reto, e por isso o ponto de recuo dele fica
/// mais perto da quina em linha reta. Sem isso, arredondar uma quina de uma forma
/// orgânica comeria muito mais de um lado que do outro.
#[test]
fn the_setback_is_measured_along_the_curve_not_across_the_chord() {
    // Quina em (0,0): à esquerda uma curva que faz uma BARRIGA (arco longo, corda curta);
    // à direita uma reta. Os dois lados recuam a MESMA distância.
    let verts = vec![
        VecVertex {
            anchor: [-10.0, 0.0],
            in_handle: [-10.0, 0.0],
            out_handle: [-8.0, 9.0], // barriga pra cima
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, 0.0],
            in_handle: [-2.0, 9.0], // chega descendo
            out_handle: [0.0, 0.0],
            kind: VertexKind::Corner,
            corner_radius: 2.0,
        },
        VecVertex::corner([10.0, 0.0]),
    ];
    let out = round_authored_corners(&verts, false).unwrap();
    assert!(all_finite(&out));
    let (a, b) = (&out[1], &out[2]); // os dois pontos de recuo da quina
    let d_curve = (a.anchor[0].powi(2) + a.anchor[1].powi(2)).sqrt();
    let d_line = (b.anchor[0].powi(2) + b.anchor[1].powi(2)).sqrt();
    // O lado CURVO recuou a mesma distância AO LONGO da curva, então em linha reta ele
    // fica MAIS PERTO da quina que o lado reto (a curva "gasta" caminho encurvando).
    assert!(
        d_curve < d_line,
        "recuo medido pela CORDA, não pela curva: curvo={d_curve:.4}, reto={d_line:.4}"
    );
    assert!(d_curve > 0.0 && d_line > 0.0);
}

/// Num caminho ABERTO as pontas não têm quina (falta um dos dois lados) — o raio delas é
/// ignorado, e elas saem intactas. Sem esta guarda o primeiro vértice indexaria o segmento
/// `-1`.
#[test]
fn the_endpoints_of_an_open_path_have_no_corner_to_round() {
    let mut verts = vec![
        VecVertex::corner([0.0, 0.0]),
        VecVertex::corner([10.0, 0.0]),
        VecVertex::corner([10.0, 10.0]),
    ];
    verts[0].corner_radius = 3.0; // ponta: ignorado
    verts[2].corner_radius = 3.0; // ponta: ignorado
    verts[1].corner_radius = 2.0; // miolo: arredonda
    let out = round_authored_corners(&verts, false).unwrap();
    assert_eq!(out.len(), 4, "só o vértice do MEIO virou dois");
    assert_eq!(out[0].anchor, [0.0, 0.0], "a ponta ficou onde estava");
    assert_eq!(out[3].anchor, [10.0, 10.0]);
}

// ------------------------------------------------- 4. a saturação (a lição do Bug #1)

/// **A ORDEM dos dois cortes de um segmento, e é este o gate que morde o guard.**
///
/// Duas quinas vizinhas dividem a aresta entre elas e as duas recuam nela. Se o corte do
/// começo passar o do fim, o pedaço entre eles corre **para trás** — a forma dobra sobre
/// si. `resolve_trims` promete que isso não acontece: cruzou, os dois viram UM número.
///
/// Ser um teste de UNIDADE aqui não é preguiça, é necessidade — e vale registrar por quê,
/// porque eu escrevi primeiro o gate errado. A varredura combinada logo abaixo **não
/// morde este guard**: o clamp de meia-corda já faz os dois recuos pararem no meio da
/// aresta em vez de se atropelarem, então o cruzamento que sobra é de **1 ulp**, e o
/// estrago dele — um segmento de 1e-15 correndo ao contrário — é invisível para qualquer
/// asserção de finitude, de caixa ou de área. Um gate que não pode ficar vermelho é um
/// placebo, e eu confirmei que aquele era: removi o guard inteiro e a varredura seguiu
/// verde. **Este** teste fica vermelho, porque testa a pós-condição diretamente.
#[test]
fn the_two_trims_of_a_segment_never_cross_they_collapse() {
    // O caso que a aritmética produz de verdade: os dois recuos se encontram no meio e um
    // deles cai 1 ulp do lado errado.
    let (a, b) = resolve_trims(0.5 + f64::EPSILON, 0.5);
    assert_eq!(a, b, "cruzou por 1 ulp ⇒ colapsa num número só");
    // E o caso grosseiro (que só existe se o clamp de meia-corda for embora um dia).
    let (a, b) = resolve_trims(0.6, 0.4);
    assert!(a <= b, "o corte do começo NUNCA passa o do fim: {a} > {b}");
    assert_eq!(a, b);
    assert!(
        (0.0..=1.0).contains(&a),
        "e o resultado é um parâmetro válido"
    );
    // O caso normal passa intocado — a rede não pode custar precisão a quem não cruzou.
    assert_eq!(resolve_trims(0.2, 0.8), (0.2, 0.8));
}

/// A varredura **COMBINADA**: os dois raios de quinas vizinhas crescem juntos e vão muito
/// além do que a aresta comporta. O que ela prova é **finitude e não-inversão** sob
/// saturação — nenhum `NaN`, nenhum pânico, nada fora da caixa, em 1681 pares de raios.
///
/// Ser combinada é o ponto: uma varredura de um parâmetro por vez nunca faz duas quinas
/// disputarem a mesma aresta, e foi assim que o Bug #1 passou por todos os gates do
/// módulo. (O que ela **não** prova é o guard de ordem — esse tem o teste acima, e a
/// razão está escrita lá.)
///
/// A asserção é **finitude**, não só ausência de pânico: um `NaN` não derruba o teste —
/// ele vira uma bbox `NaN`, o gizmo some, e a forma "desaparece" três telas adiante.
#[test]
fn two_neighbouring_corners_devouring_the_same_edge_never_panic_and_never_produce_nan() {
    let mut n = 0;
    for i in 0..=40 {
        for j in 0..=40 {
            // Os dois raios varrem MUITO além do que a aresta de 10 comporta.
            let (r0, r1) = (i as f64 * 0.5, j as f64 * 0.5);
            let mut verts = square();
            verts[0].corner_radius = r0;
            verts[1].corner_radius = r1; // vizinha imediata — dividem a aresta de baixo
            // `None` = nada a arredondar (os dois raios são zero) — é a identidade, e a
            // fonte É a saída. Só o (0,0) da varredura cai aqui.
            let out = round_authored_corners(&verts, true).unwrap_or(verts);
            assert!(
                all_finite(&out),
                "NaN/inf com r0={r0}, r1={r1} — o gizmo sumiria e ninguém saberia por quê"
            );
            // A saturação não INVERTE a forma: tudo continua dentro do quadrado original.
            for v in &out {
                assert!(
                    (-1e-9..=10.0 + 1e-9).contains(&v.anchor[0])
                        && (-1e-9..=10.0 + 1e-9).contains(&v.anchor[1]),
                    "r0={r0}, r1={r1}: âncora {:?} estourou a caixa — o raio inverteu a forma",
                    v.anchor
                );
            }
            n += 1;
        }
    }
    assert_eq!(n, 41 * 41, "a varredura combinada rodou inteira");
}

/// A mesma varredura, mas com **TODAS as quinas** pedindo raio absurdo ao mesmo tempo — o
/// caso em que cada aresta é disputada pelos dois lados e não sobra nada. A forma satura
/// (vira um círculo inscrito, no limite) e continua finita e dentro da caixa.
#[test]
fn every_corner_at_an_absurd_radius_saturates_instead_of_exploding() {
    for r in [5.0, 10.0, 50.0, 1e3, 1e6, 1e12] {
        let mut verts = square();
        for v in &mut verts {
            v.corner_radius = r;
        }
        let out = round_authored_corners(&verts, true).unwrap();
        assert!(all_finite(&out), "r={r} produziu não-finito");
        for v in &out {
            assert!(
                (-1e-9..=10.0 + 1e-9).contains(&v.anchor[0])
                    && (-1e-9..=10.0 + 1e-9).contains(&v.anchor[1]),
                "r={r}: {:?} saiu da caixa",
                v.anchor
            );
        }
    }
}

/// Uma quina cujo raio é grande **e** cujos lados são curvos e curtos — a combinação que
/// nenhuma varredura de um parâmetro só alcança, e onde o recuo por bisseção pode saturar
/// nas duas pontas do mesmo segmento ao mesmo tempo.
#[test]
fn absurd_radii_on_a_curved_path_stay_finite() {
    for r in [0.5, 2.0, 10.0, 1e6] {
        for bulge in [0.0, 1.0, 5.0, 20.0, -20.0] {
            let verts = vec![
                VecVertex {
                    anchor: [0.0, 0.0],
                    in_handle: [-bulge, -bulge],
                    out_handle: [bulge, bulge],
                    kind: VertexKind::Corner,
                    corner_radius: r,
                },
                VecVertex {
                    anchor: [4.0, 0.0],
                    in_handle: [4.0 - bulge, bulge],
                    out_handle: [4.0 + bulge, -bulge],
                    kind: VertexKind::Corner,
                    corner_radius: r,
                },
                VecVertex {
                    anchor: [2.0, 3.0],
                    in_handle: [2.0 + bulge, 3.0],
                    out_handle: [2.0 - bulge, 3.0],
                    kind: VertexKind::Corner,
                    corner_radius: r,
                },
            ];
            let out = round_authored_corners(&verts, true)
                .unwrap_or_else(|| panic!("r={r}, bulge={bulge}: devia ter cozinhado"));
            assert!(all_finite(&out), "r={r}, bulge={bulge} produziu não-finito");
        }
    }
}

// ------------------------------------------------- 5. A ALÇA NÃO MENTE (seed = sample)

/// **A invariante mais importante do arquivo: o que a alça mostra é onde o cozimento
/// arredonda.**
///
/// A alça de raio corre pela bissetriz, e o quanto ela andou (`setback`) vira raio por
/// `r = setback · tan(θ/2)`. O cozimento faz o caminho inverso: `t = r / tan(θ/2)`. Se as
/// duas contas — ou os dois CLAMPS — divergirem, o usuário arrasta a alça e a forma não
/// acompanha: a alça continua andando com a quina parada. Este repo já pagou três vezes
/// por essa classe de bug (a memória `feedback_derived_coordinate_seed_must_match_sample`),
/// sempre por semear uma coordenada derivada com uma fórmula e lê-la com outra.
///
/// Aqui a asserção é direta: para um recuo qualquer, o raio que a alça produz, cozido,
/// recua **exatamente aquele tanto** na aresta. Inclusive além do teto — onde os dois
/// têm de saturar no MESMO ponto.
#[test]
fn the_handle_never_lies_the_cook_trims_exactly_where_the_handle_says() {
    // Retângulo 20×10: as duas arestas da quina 0 têm comprimentos DIFERENTES, então o
    // teto (metade da mais curta) é assimétrico — se o cozimento usasse a outra aresta,
    // ou nenhuma, isto pegaria.
    let base = [[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();

    // Os raios vêm por DOIS caminhos, e os dois têm de cair no mesmo lugar:
    //  (a) pela alça (`radius_for_setback`, que já clampa);
    //  (b) CRU, gravado direto — é o que um número digitado no painel, um arquivo salvo
    //      ou um weld produzem. O (b) é o que morde: ele alcança raios que a alça sozinha
    //      nunca geraria, e é exatamente onde o cozimento e o gizmo podem saturar em
    //      pontos diferentes. Testar só o (a) é testar o caminho fácil (e eu testei, e
    //      ficou verde com o cozimento clampando errado).
    let frame0 = corner_at(&base, true, 0).expect("a quina 0 existe");
    let by_handle = [0.5, 1.0, 3.0, 4.999, 5.0, 8.0, 50.0].map(|d| frame0.radius_for_setback(d));
    let raw = [0.5, 4.9, 5.0, 5.1, 12.0, 100.0, 1e9];

    for r in by_handle.into_iter().chain(raw) {
        let mut verts = base.clone();
        verts[0].corner_radius = r;

        // E este é o recuo que a alça vai DESENHAR de volta (o que o usuário vê).
        let drawn = corner_at(&verts, true, 0).unwrap();
        let shown = drawn.setback.min(drawn.max_setback);

        // O que o COZIMENTO de fato fez: a distância da quina (0,0) até o ponto de recuo
        // na aresta que chega (a vertical x=0, vinda de (0,10)).
        let out = round_authored_corners(&verts, true).unwrap();
        let trimmed = out[0].anchor; // o vértice do lado que CHEGA
        assert!(trimmed[0].abs() < 1e-9, "o recuo é sobre a aresta vertical");
        let cooked_setback = trimmed[1];

        assert!(
            (cooked_setback - shown).abs() < 1e-6,
            "a alça MENTIU: com raio {r}, ela desenha o recuo em {shown:.6} e o \
             cozimento recuou {cooked_setback:.6} — a forma não segue o cursor"
        );
    }
}

/// E o round-trip da alça: arrastar até `d` e reler tem de devolver `d` (clampado ao
/// teto). É o que faz a alça ficar EXATAMENTE sob o cursor em vez de escorregar dele.
#[test]
fn dragging_the_handle_to_a_distance_and_reading_it_back_gives_the_same_distance() {
    let base = [[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();
    let frame = corner_at(&base, true, 0).unwrap();
    assert!(
        (frame.max_setback - 5.0).abs() < 1e-9,
        "o teto é metade da aresta MAIS CURTA (10/2), não da mais longa: {}",
        frame.max_setback
    );
    for d in [0.0, 0.25, 1.0, 4.0, 5.0, 9.0, 1e6] {
        let mut verts = base.clone();
        verts[0].corner_radius = frame.radius_for_setback(d);
        let back = corner_at(&verts, true, 0).unwrap();
        let want = d.min(frame.max_setback);
        assert!(
            (back.setback - want).abs() < 1e-9,
            "arrastei até {d} (teto {}), reli {} — queria {want}",
            frame.max_setback,
            back.setback
        );
        // E a alça é desenhada onde o dedo está: sobre a bissetriz, ao recuo pedido.
        let h = back.handle_at(back.setback);
        let d_from_anchor = (h[0] - 0.0).hypot(h[1] - 0.0);
        assert!(
            (d_from_anchor - want).abs() < 1e-9,
            "a alça foi desenhada a {d_from_anchor} da quina, e devia estar a {want}"
        );
    }
}

// ----------------------------------------------------------- 6. cozinhar é idempotente

/// Cozinhar o cozido é o cozido. Os vértices que saem do forno têm `corner_radius = 0`,
/// então uma segunda passada é a identidade — e é isso que garante que ligar o `cooked()`
/// em dois consumidores encadeados (o hit-test do traço chama `nearest_point_on_path` num
/// path já cozido, por exemplo) não arredonde a mesma quina duas vezes.
#[test]
fn cooking_the_cooked_is_the_cooked() {
    let mut verts = square();
    verts[0].corner_radius = 2.0;
    verts[2].corner_radius = 1.0;
    let path = VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    };
    let once = path.cooked().into_owned();
    let twice = once.cooked();
    assert!(
        matches!(twice, std::borrow::Cow::Borrowed(_)),
        "o cozido não tem raio pendente — recozinhar é um empréstimo"
    );
    assert_eq!(once.verts, twice.verts);
}

/// O raio é um COMPRIMENTO no espaço local: escalar o path escala o raio junto. Sem isso,
/// dobrar o tamanho de uma forma manteria a quina com o raio antigo — metade do
/// arredondamento relativo, silenciosamente.
#[test]
fn scaling_a_path_scales_its_corner_radius_with_it() {
    let mut scene = crate::VecScene::default();
    let mut verts = square();
    verts[0].corner_radius = 2.0;
    let id = scene.push_path(VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    });
    assert!(scene.scale_path(id, 3.0, 3.0, [0.0, 0.0]));
    let r = scene.path_mut(id).unwrap().verts[0].corner_radius;
    assert!(
        (r - 6.0).abs() < 1e-9,
        "raio devia ter escalado 3× (2 → 6), veio {r}"
    );
}

/// "Afiar" (`sharpen_path`) promete uma forma ANGULAR — então ele zera o raio vivo junto
/// com os handles. Sem isso a forma continuaria redonda depois do comando que existe para
/// deixá-la pontuda, e pareceria um bug (porque seria um).
#[test]
fn sharpening_a_path_clears_the_live_corner_radius_too() {
    let mut scene = crate::VecScene::default();
    let mut verts = square();
    verts[0].corner_radius = 2.0;
    let id = scene.push_path(VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    });
    assert!(scene.sharpen_path(id));
    assert_eq!(scene.path_mut(id).unwrap().verts[0].corner_radius, 0.0);
}

// ---------------------------------------------------------------- 5. o CHANFRO (chamfer)

/// Cozinha um quadrado com o vértice 1 recuado por `radius` (o SINAL escolhe o estilo) e
/// devolve os dois vértices em que a quina se partiu (índices 1 e 2 na saída: os sharp são
/// 1-para-1, o recuado vira 2).
fn split_pair(radius: f64) -> (VecVertex, VecVertex) {
    let mut v = square();
    v[1].corner_radius = radius;
    let out = round_authored_corners(&v, true).expect("há recuo a cozinhar");
    (out[1], out[2])
}

/// A distância do MEIO do segmento de ligação até a corda reta entre os dois pontos de recuo.
/// Zero = reta (chanfro); positiva = arco (arredondado). Mede a APARÊNCIA, não a fórmula.
fn bulge(a: &VecVertex, b: &VecVertex) -> f64 {
    let mid = crate::cubic_at(a.anchor, a.out_handle, b.in_handle, b.anchor, 0.5);
    let (p0, p3) = (a.anchor, b.anchor);
    let chord = [p3[0] - p0[0], p3[1] - p0[1]];
    let len = chord[0].hypot(chord[1]);
    if len < 1e-12 {
        return 0.0;
    }
    ((mid[0] - p0[0]) * chord[1] - (mid[1] - p0[1]) * chord[0]).abs() / len
}

/// **O chanfro é uma RETA; o arredondado, um ARCO — nos MESMOS pontos de recuo.** É a feature
/// inteira (os dois modos do corner widget do Illustrator): o sinal do `corner_radius` escolhe
/// só o traçado da ligação, e nada mais — o recuo é a magnitude, idêntico para os dois.
#[test]
fn a_chamfer_is_a_straight_cut_a_round_is_an_arc_at_the_same_setback() {
    let (r_in, r_out) = split_pair(3.0); // arredondado
    let (c_in, c_out) = split_pair(-3.0); // chanfro, MESMA magnitude

    // Os pontos de recuo são IDÊNTICOS — o recuo é a magnitude, o estilo não o move.
    assert_eq!(r_in.anchor, c_in.anchor, "o chanfro recua igual ao arredondado (in)");
    assert_eq!(r_out.anchor, c_out.anchor, "idem (out)");

    // O chanfro é reto; o arredondado bojudo. O fosso é enorme, não ruído de ponto flutuante.
    let round_bulge = bulge(&r_in, &r_out);
    let chamfer_bulge = bulge(&c_in, &c_out);
    assert!(
        chamfer_bulge < 1e-9,
        "o chanfro tem de ser RETO (bojo medido {chamfer_bulge})"
    );
    assert!(
        round_bulge > 0.3,
        "o arredondado tem de BOJAR de verdade (bojo medido {round_bulge})"
    );
    assert!(all_finite(&[c_in, c_out, r_in, r_out]));
}

/// A convenção de sinal mora nas portas do `VecVertex`: magnitude e estilo são ORTOGONAIS. O
/// arrasto (tamanho) não converte estilo, e o toggle (estilo) não muda o tamanho.
#[test]
fn the_corner_sign_convention_keeps_magnitude_and_style_orthogonal() {
    let mut v = VecVertex::corner([0.0, 0.0]);
    v.corner_radius = 4.0;
    assert!(!v.is_chamfer());
    assert_eq!(v.corner_size(), 4.0);

    v.set_chamfer(true); // vira chanfro, tamanho intacto
    assert!(v.is_chamfer());
    assert_eq!(v.corner_size(), 4.0);
    assert_eq!(v.corner_radius, -4.0);

    v.set_corner_size(7.0); // redimensiona, estilo intacto
    assert!(v.is_chamfer(), "arrastar a alça não pode desfazer o chanfro");
    assert_eq!(v.corner_size(), 7.0);

    v.set_chamfer(false); // volta a arredondado, tamanho intacto
    assert!(!v.is_chamfer());
    assert_eq!(v.corner_radius, 7.0);
}
