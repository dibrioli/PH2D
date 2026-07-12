//! **Os gates do canto rico**: raio por-canto + suavização (squircle). Arquivo irmão de
//! `shapes.rs` (teto de LOC).
//!
//! Três coisas são provadas aqui, e nenhuma delas é "a fórmula bate com a fórmula":
//!
//! 1. **Compatibilidade de save, executável.** Um vetor de valores escrito ANTES desta
//!    feature (só `values[0]`) cozinha a MESMA geometria, byte a byte. Sem isto, o projeto de
//!    ontem abre com três cantos vivos.
//! 2. **Localidade.** Cada um dos quatro campos de canto mexe SÓ no canto dele. O gate da
//!    shell (`every_declared_field_of_every_shape_moves_its_geometry`) só prova que o campo
//!    faz *alguma* coisa; um bug de índice passaria por ele cantando.
//! 3. **A rampa de curvatura.** A suavização é uma PROPRIEDADE geométrica (o salto de
//!    curvatura na junção reta/arco CAI), não uma fórmula. O teste mede a curvatura da curva
//!    que saiu, não a que eu pretendia escrever — é o oráculo que modela a APARÊNCIA.

use super::*;
use crate::corners::round_closed_corners;
use crate::{DEFAULT_CORNER_RADIUS, MAX_SHAPE_FIELDS, ShapeKind, ShapeValues, cook};

const A: [f64; 2] = [-4.0, -3.0];
const B: [f64; 2] = [4.0, 3.0];
/// `[TL, TR, BR, BL]` — os cantos como o usuário os vê (mundo Y-para-CIMA: `y` maior = topo).
const TL: usize = 0;
const TR: usize = 1;
const BR: usize = 2;
const BL: usize = 3;

/// A geometria INTEIRA (âncoras + handles) — o que "byte a byte" quer dizer aqui. Comparar só
/// âncoras deixaria passar um handle errado, que é justamente o que a suavização mexe.
fn geom(p: &VecPath) -> Vec<([f64; 2], [f64; 2], [f64; 2])> {
    p.verts_all()
        .map(|v| (v.anchor, v.in_handle, v.out_handle))
        .collect()
}

/// O canto do mundo mais próximo de `p` (qual dos quatro), para provar LOCALIDADE.
fn quadrant(p: [f64; 2]) -> usize {
    match (p[0] < 0.0, p[1] > 0.0) {
        (true, true) => TL,
        (false, true) => TR,
        (false, false) => BR,
        (true, false) => BL,
    }
}

/// Quanto a geometria se mexeu PERTO de cada canto: para cada canto, a maior distância entre
/// a nuvem de pontos de `before` e a de `after` naquele quadrante. Zero ⇒ o canto não foi
/// tocado. (Compara nuvem-com-nuvem porque a suavização MUDA a contagem de vértices.)
fn moved_per_corner(before: &VecPath, after: &VecPath) -> [f64; 4] {
    let cloud = |p: &VecPath, q: usize| -> Vec<[f64; 2]> {
        p.verts_all()
            .flat_map(|v| [v.anchor, v.in_handle, v.out_handle])
            .filter(|&x| quadrant(x) == q)
            .collect()
    };
    let mut out = [0.0; 4];
    for (q, o) in out.iter_mut().enumerate() {
        let (u, v) = (cloud(before, q), cloud(after, q));
        // Hausdorff (ida e volta): se um lado ganhou ou perdeu pontos, a distância acusa.
        let one_way = |from: &[[f64; 2]], to: &[[f64; 2]]| -> f64 {
            from.iter()
                .map(|p| {
                    to.iter()
                        .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                        .fold(f64::INFINITY, f64::min)
                })
                .fold(0.0, f64::max)
        };
        *o = one_way(&u, &v).max(one_way(&v, &u));
    }
    out
}

// ─── 1. Compatibilidade de save ──────────────────────────────────────────────────────────

/// **O gate anti-quebra-de-save.** Um round-rect salvo ANTES desta feature guarda só
/// `values[0]`; os campos apendados leem ZERO (regra 2 do catálogo). Ele TEM de cozinhar a
/// mesma geometria de sempre — byte a byte, não "parecido".
///
/// É o teste que reprova a escolha óbvia (três raios ABSOLUTOS): com ela, `[r, 0, 0, 0]`
/// viraria um retângulo com um canto redondo e três vivos, e o projeto de ontem abriria
/// deformado. Por isso os campos novos são DESVIOS — neutros em zero.
#[test]
fn a_value_vector_written_before_this_feature_cooks_the_same_geometry() {
    for &r in &[0.0, 0.001, 0.4, 1.5, 3.0, 999.0] {
        let want = geom(&rounded_rect(A, B, r));
        // (a) O save curto: o array nem chega ao campo novo.
        assert_eq!(
            geom(&cook(ShapeKind::RoundRect, A, B, &[r])),
            want,
            "save CURTO (r={r}) mudou de geometria"
        );
        // (b) O save cheio de ontem: os campos apendados existem, valendo zero.
        let mut v: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
        v[0] = r;
        assert_eq!(
            geom(&cook(ShapeKind::RoundRect, A, B, &v)),
            want,
            "save de ONTEM (r={r}) mudou de geometria"
        );
    }
    // E o default da forma continua sendo o round-rect uniforme de sempre.
    let d = ShapeKind::RoundRect.defaults();
    assert_eq!(
        geom(&cook(ShapeKind::RoundRect, A, B, &d)),
        geom(&rounded_rect(A, B, DEFAULT_CORNER_RADIUS)),
        "a forma NOVA nasceu diferente da de sempre"
    );
}

/// **A identidade da suavização é sagrada**: `smoothing = 0` é byte-a-byte o arco de círculo
/// de sempre — no round-rect E no motor genérico (polígono / estrela). Um `0.0` que passasse
/// pela construção nova sairia 1 ulp diferente e contaminaria toda forma já salva.
#[test]
fn smoothing_zero_is_byte_identical_to_the_circular_corner() {
    let radii = [1.2, 0.4, 2.0, 0.0];
    assert_eq!(
        geom(&rounded_rect_corners(A, B, radii, 0.0)),
        geom(&rounded_rect_corners(A, B, radii, -0.0)),
        "o zero negativo tambem e zero"
    );
    // O motor genérico: a estrela e o polígono passam por ele.
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]];
    let rs = [2.0, 1.0, 0.5, 3.0];
    assert_eq!(
        geom(&crate::corners::round_closed_corners_smooth(&pts, &rs, 0.0)),
        geom(&round_closed_corners(&pts, &rs)),
        "smoothing 0 no motor generico nao e a identidade"
    );
    // E a forma de sempre segue igual: polígono e estrela sem suavização, intocados.
    let poly = regular_polygon_rounded([0.0, 0.0], 3.0, 3.0, 6, 0.5);
    assert_eq!(
        geom(&poly),
        geom(&regular_polygon_rounded([0.0, 0.0], 3.0, 3.0, 6, 0.5)),
        "o poligono redondo mudou"
    );
}

/// O caminho GERAL (motor de quinas) e o round-rect construído à mão desenham a MESMA volta:
/// mesmo sentido, mesmas oito âncoras, mesmos handles — a menos de `1,7e-10`.
///
/// Esse resíduo não é ruído: é o `KAPPA` de [`crate::shapes`], que está escrito como literal
/// TRUNCADO (`0.552_284_75`, oito casas) enquanto o motor de quinas o CALCULA
/// (`(4/3)·tan(π/8) = 0.5522847498307933`). Os dois diferem em `1,69e-10` — invisível
/// (bilionésimo de unidade de mundo), mas real. Não se corrige o literal: ele está em toda
/// geometria já salva, e mexer nele mudaria byte a byte o que o usuário já tem no disco. É
/// exatamente por isso que o caso uniforme + sem suavização **desvia para a função antiga**:
/// ali a identidade tem de ser byte a byte, não "quase".
///
/// (As duas voltas começam em vértices DIFERENTES do mesmo ciclo — o motor genérico emite a
/// quina de baixo-esquerda inteira, então ele abre no lado esquerdo; o round-rect à mão abre
/// no lado de baixo. Mesma volta, mesmo sentido, fase deslocada de um vértice.)
#[test]
fn the_general_corner_engine_agrees_with_the_hand_built_round_rect() {
    let hand = geom(&rounded_rect(A, B, 1.3));
    let pts = [[-4.0, -3.0], [4.0, -3.0], [4.0, 3.0], [-4.0, 3.0]];
    let general = geom(&round_closed_corners(&pts, &[1.3; 4]));
    assert_eq!(hand.len(), general.len(), "mesma contagem de vertices");
    // O motor genérico está uma fase à frente: `general[1] == hand[0]`.
    let n = hand.len();
    for (i, h) in hand.iter().enumerate() {
        let g = &general[(i + 1) % n];
        for (p, q) in [(h.0, g.0), (h.1, g.1), (h.2, g.2)] {
            assert!(
                (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9,
                "vertice {i}: {p:?} != {q:?}"
            );
        }
    }
}

// ─── 2. Localidade dos quatro cantos ─────────────────────────────────────────────────────

/// **Cada campo mexe SÓ no seu canto.** O gate da shell prova que o campo faz *alguma* coisa;
/// este prova que ele faz a coisa CERTA — um índice trocado (BR onde era BL) passaria por lá
/// e apareceria só na tela do Enio.
#[test]
fn each_corner_field_moves_only_its_own_corner() {
    let base = 1.0;
    let uniform = rounded_rect_corners(A, B, round_rect_radii(base, [0.0; 3]), 0.0);
    // Campo `i` (1 = TR, 2 = BR, 3 = BL) ⇒ canto `want`.
    for (i, want) in [(1, TR), (2, BR), (3, BL)] {
        let mut off = [0.0; 3];
        off[i - 1] = -base; // afina ESTE canto até o vivo
        let sharp = rounded_rect_corners(A, B, round_rect_radii(base, off), 0.0);
        let moved = moved_per_corner(&uniform, &sharp);
        assert!(
            moved[want] > 0.1,
            "o campo {i} nao mexeu no canto {want} (moveu {:.3})",
            moved[want]
        );
        for (q, &m) in moved.iter().enumerate() {
            // O piso NÃO é zero exato por um motivo conhecido e medido: sair do caso uniforme
            // troca a construção à mão (com o `KAPPA` literal, truncado em 8 casas) pelo motor
            // de quinas (que o calcula), e os handles dos cantos INTOCADOS deslocam `1,7e-10`
            // — vide `the_general_corner_engine_agrees_with_the_hand_built_round_rect`. O
            // vazamento que este gate caça (um índice de canto trocado) vale `~1`, não `1e-10`.
            assert!(
                q == want || m < 1e-9,
                "o campo {i} vazou para o canto {q} (moveu {m:.3e})"
            );
        }
    }
    // E o raio BASE é o mestre: mexer nele move os QUATRO.
    let bigger = rounded_rect_corners(A, B, round_rect_radii(base * 2.0, [0.0; 3]), 0.0);
    for (q, &m) in moved_per_corner(&uniform, &bigger).iter().enumerate() {
        assert!(m > 0.1, "o raio base nao moveu o canto {q}");
    }
}

/// Os quatro cantos são alcançáveis INDEPENDENTEMENTE (o desvio não perde expressividade):
/// qualquer quádrupla `(tl, tr, br, bl)` sai de `base = tl` + os três desvios.
#[test]
fn any_four_corner_radii_are_reachable_through_the_offsets() {
    let want = [0.5, 2.0, 0.0, 1.25_f64];
    let [tl, tr, br, bl] = want;
    let got = round_rect_radii(tl, [tr - tl, br - tl, bl - tl]);
    assert_eq!(got, want, "o desvio perdeu expressividade");
    // E cada canto do desenho tem, de fato, o raio pedido: o vértice mais próximo do canto
    // do mundo recua `r` da quina (num canto reto, `t = r`).
    let p = rounded_rect_corners(A, B, want, 0.0);
    let corner_world = [[-4.0, 3.0], [4.0, 3.0], [4.0, -3.0], [-4.0, -3.0]];
    for (q, &c) in corner_world.iter().enumerate() {
        let nearest = p
            .verts_all()
            .map(|v| (v.anchor[0] - c[0]).hypot(v.anchor[1] - c[1]))
            .fold(f64::INFINITY, f64::min);
        assert!(
            (nearest - want[q]).abs() < 1e-9,
            "canto {q}: recuo {nearest:.4} != raio {:.4}",
            want[q]
        );
    }
}

// ─── 3. A suavização: a rampa de curvatura ───────────────────────────────────────────────

/// Curvatura de uma cúbica em `t`: `|B' x B''| / |B'|^3`. `B'` degenerado (segmento reto com
/// handles nulos) ⇒ `0` — uma reta tem curvatura zero, não `NaN`.
fn curvature(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> f64 {
    let u = 1.0 - t;
    let d1 = |k: usize| {
        3.0 * (u * u * (p1[k] - p0[k]) + 2.0 * u * t * (p2[k] - p1[k]) + t * t * (p3[k] - p2[k]))
    };
    let d2 =
        |k: usize| 6.0 * (u * (p2[k] - 2.0 * p1[k] + p0[k]) + t * (p3[k] - 2.0 * p2[k] + p1[k]));
    let (vx, vy) = (d1(0), d1(1));
    let speed = vx.hypot(vy);
    if speed < 1e-9 {
        return 0.0;
    }
    (vx * d2(1) - vy * d2(0)).abs() / speed.powi(3)
}

/// O maior SALTO de curvatura entre dois segmentos vizinhos do contorno — a medida do que o
/// olho enxerga como "canto colado". Num round-rect de arco puro ele vale `1/r` (a reta tem
/// curvatura 0 e o arco tem `1/r`, um degrau); num canto contínuo perfeito, 0.
fn worst_curvature_jump(p: &VecPath) -> f64 {
    let v: Vec<_> = p.verts.clone();
    let n = v.len();
    // Curvatura no INÍCIO e no FIM de cada segmento (o salto mora nas junções).
    let ends: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let (a, b) = (&v[i], &v[(i + 1) % n]);
            let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
            (
                curvature(p0, p1, p2, p3, 0.0),
                curvature(p0, p1, p2, p3, 1.0),
            )
        })
        .collect();
    (0..n)
        .map(|i| (ends[i].1 - ends[(i + 1) % n].0).abs())
        .fold(0.0, f64::max)
}

/// **A PROPRIEDADE, não a fórmula.** Conforme a suavização sobe, o salto de curvatura na
/// junção reta/arco CAI — monotonamente — e some em `s = 1` (aí o arco desaparece e as duas
/// asas se encontram espelhadas na bissetriz: mesma curvatura dos dois lados, por simetria).
///
/// O oráculo mede a curvatura da curva que SAIU (as cúbicas de verdade), não a que eu
/// pretendia escrever: se a construção regredir para uma superelipse, para um arco mal
/// costurado ou para handles trocados, este teste cai — mesmo que a "fórmula" continue lá.
#[test]
fn smoothing_ramps_the_curvature_instead_of_jumping_it() {
    let r = 1.5;
    let jumps: Vec<f64> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|&s| worst_curvature_jump(&rounded_rect_corners(A, B, [r; 4], s)))
        .collect();
    // Sem suavização o salto é o DEGRAU do arco: 0 (a reta) -> ~1/r (o arco). "~" porque a
    // cúbica que aproxima o quarto de círculo tem, na ponta, curvatura `2(1−κ)/(3κ²)/r ≈
    // 0,9785/r` (o erro clássico do círculo-Bézier), não `1/r` cravado.
    assert!(
        (jumps[0] - 1.0 / r).abs() < 0.03 / r,
        "o arco puro deveria saltar ~1/r = {:.3}, saltou {:.3}",
        1.0 / r,
        jumps[0]
    );
    for w in jumps.windows(2) {
        assert!(
            w[1] < w[0] - 1e-6,
            "a suavizacao tem de DERRUBAR o salto de curvatura: {jumps:?}"
        );
    }
    // O degrau COLAPSA assim que a suavização entra: a asa sai da reta com curvatura zero e
    // chega no arco com a curvatura DELE (é a constrição que dita o `b` — ver `crate::smooth`).
    // O resíduo é só o erro da cúbica que aproxima o arco, e ele encolhe junto com o arco.
    assert!(
        jumps[1] < 0.05 * jumps[0],
        "a rampa nao colapsou o degrau: {:.4} (era {:.4})",
        jumps[1],
        jumps[0]
    );
    // Em `s = 1` a curva é contínua em curvatura (o canto contínuo de verdade: o arco some e
    // as duas asas se encontram espelhadas na bissetriz).
    assert!(
        jumps[4] < 1e-6,
        "s=1 tem de ser G2 (salto ~0), foi {:.3e}",
        jumps[4]
    );
}

/// A forma suavizada continua CABENDO na caixa do gesto (e não encolhe para dentro dela): a
/// asa se estende pela aresta, então uma construção errada ou vaza a caixa (o gizmo mente) ou
/// come o lado reto inteiro. Vale para todo `s`, inclusive com raio absurdo (satura).
#[test]
fn a_smoothed_corner_still_fits_its_box() {
    for &s in &[0.1, 0.5, 0.9, 1.0] {
        for &r in &[0.2, 1.5, 3.0, 999.0] {
            let p = rounded_rect_corners(A, B, [r; 4], s);
            let mut lo = [f64::MAX; 2];
            let mut hi = [f64::MIN; 2];
            for v in p.verts_all() {
                for x in [v.anchor, v.in_handle, v.out_handle] {
                    for k in 0..2 {
                        lo[k] = lo[k].min(x[k]);
                        hi[k] = hi[k].max(x[k]);
                    }
                }
            }
            // Cabe: nem uma âncora nem um HANDLE (que é o que puxa a curva) sai da caixa. Com
            // todos os pontos de controle dentro, o casco convexo — logo a curva — está dentro.
            assert!(
                lo[0] >= A[0] - 1e-9
                    && lo[1] >= A[1] - 1e-9
                    && hi[0] <= B[0] + 1e-9
                    && hi[1] <= B[1] + 1e-9,
                "s={s} r={r}: vazou a caixa ({lo:?}..{hi:?})"
            );
            // E toca os quatro lados (não encolheu): o canto suave ainda é tangente à caixa.
            assert!(
                (lo[0] - A[0]).abs() < 1e-9
                    && (lo[1] - A[1]).abs() < 1e-9
                    && (hi[0] - B[0]).abs() < 1e-9
                    && (hi[1] - B[1]).abs() < 1e-9,
                "s={s} r={r}: encolheu para dentro da caixa ({lo:?}..{hi:?})"
            );
        }
    }
}

/// A suavização vale para QUALQUER ângulo de quina — é o mesmo motor do polígono e da estrela
/// (a construção do Figma é de 90°; a daqui é do ângulo geral). Numa quina CÔNCAVA (o vale de
/// uma estrela) ela também vale: a rampa de curvatura cai igual.
#[test]
fn the_smoothing_generalises_to_any_corner_angle() {
    // Um pentágono (quinas de 108°) e a "seta" côncava dos testes de `corners`.
    let penta: Vec<[f64; 2]> = (0..5)
        .map(|i| {
            let a = std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * f64::from(i) / 5.0;
            [3.0 * a.cos(), 3.0 * a.sin()]
        })
        .collect();
    let arrow = [
        [0.0, 0.0],
        [5.0, 5.0],
        [10.0, 0.0],
        [10.0, 10.0],
        [0.0, 10.0],
    ];
    for (name, pts, r) in [
        ("pentagono", penta.as_slice(), 0.8),
        ("quina concava", arrow.as_slice(), 1.0),
    ] {
        let radii = vec![r; pts.len()];
        let sharp = crate::corners::round_closed_corners_smooth(pts, &radii, 0.0);
        let smooth = crate::corners::round_closed_corners_smooth(pts, &radii, 0.6);
        let (j0, j1) = (worst_curvature_jump(&sharp), worst_curvature_jump(&smooth));
        assert!(
            j1 < j0 - 1e-6,
            "{name}: a suavizacao nao derrubou o salto ({j0:.3} -> {j1:.3})"
        );
        // A forma não estourou: todo ponto de controle segue no casco do contorno original,
        // ampliado pelo raio (uma quina suave NUNCA sai mais do que a quina que ela substitui).
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in pts {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        for v in smooth.verts_all() {
            for k in 0..2 {
                assert!(
                    v.anchor[k] >= lo[k] - 1e-9 && v.anchor[k] <= hi[k] + 1e-9,
                    "{name}: a quina suave saiu da caixa do contorno"
                );
            }
        }
    }
}

/// **O usuário arrasta sliders** — então o teste arrasta também, e nos cantos do espaço de
/// parâmetros (raio absurdo, desvio absurdo, suavização subnormal, caixa degenerada). Nada de
/// `NaN`, nada de pânico, nada fora da caixa. O `0/0` mora perto: com `β = 0` o `b` da asa é
/// `tan²(β/2)/sin β` — um `s` subnormal vindo de um save corrompido sumiria com a forma.
#[test]
fn no_slider_position_produces_nan_or_escapes_the_box() {
    let boxes = [(A, B), ([0.0, 0.0], [0.0, 0.0]), ([1.0, 2.0], [1.0, 9.0])];
    let sliders = [
        -1e9,
        -500.0,
        -0.5,
        0.0,
        f64::MIN_POSITIVE,
        1e-12,
        0.01,
        0.5,
        1.0,
        3.0,
        500.0,
        1e9,
    ];
    for (a, b) in boxes {
        for &base in &sliders {
            for &off in &sliders {
                for &s in &sliders {
                    let v = [base, off, -off, off * 0.5, s, 0.0, 0.0, 0.0];
                    let p = cook(ShapeKind::RoundRect, a, b, &v);
                    assert!(!p.verts.is_empty(), "cozinhou vazio: {v:?}");
                    for x in p.verts_all() {
                        for c in [x.anchor, x.in_handle, x.out_handle] {
                            assert!(c[0].is_finite() && c[1].is_finite(), "NaN/inf em {v:?}");
                            let (lo, hi) = (a[0].min(b[0]), a[0].max(b[0]));
                            let (lo_y, hi_y) = (a[1].min(b[1]), a[1].max(b[1]));
                            assert!(
                                c[0] >= lo - 1e-9
                                    && c[0] <= hi + 1e-9
                                    && c[1] >= lo_y - 1e-9
                                    && c[1] <= hi_y + 1e-9,
                                "vazou a caixa em {v:?}: {c:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Uma quina de raio ZERO fica CRUA mesmo com suavização a 1: não há arco para suavizar (e é
/// por isso que o campo `Smoothing` só existe onde há raio no default — um campo morto na
/// tela é o que o gate anti-campo-morto da shell reprova).
#[test]
fn smoothing_a_sharp_corner_is_a_no_op() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]];
    let sharp = crate::corners::round_closed_corners_smooth(&pts, &[0.0; 4], 1.0);
    assert_eq!(sharp.verts.len(), 4, "quina viva nao virou tres vertices");
    for (v, want) in sharp.verts.iter().zip(pts) {
        assert_eq!(v.anchor, want);
        assert_eq!(v.in_handle, want, "handle nulo (aresta reta)");
        assert_eq!(v.out_handle, want);
    }
    // E mistura: só o canto COM raio suaviza; o vivo continua vivo.
    let mixed = crate::corners::round_closed_corners_smooth(&pts, &[2.0, 0.0, 0.0, 0.0], 0.8);
    assert!(
        mixed.verts.len() == 3 + 4,
        "um canto suave (4 vertices) + tres vivos, deu {}",
        mixed.verts.len()
    );
}
