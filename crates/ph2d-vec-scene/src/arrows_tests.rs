//! Testes de `arrows.rs` — arquivo irmao (teto de 700 LOC por arquivo, HR-18).
//!
//! Ligado por `#[path]` no modulo pai, entao `use super::*` continua valendo.

use super::*;

const A: [f64; 2] = [-2.0, -1.0];
const B: [f64; 2] = [2.0, 1.0];
/// Os defaults da seta circular (espelham `ShapeKind::defaults`).
const CURVED: (f64, f64, f64, f64, f64) = (270.0, 180.0, 0.125, 0.25, 0.12);

fn curved_default() -> VecPath {
    arrow_curved(A, B, CURVED.0, CURVED.1, CURVED.2, CURVED.3, CURVED.4)
}

/// Quantas amostras por cúbica a bbox do teste usa. O `fit` resolve a derivada da
/// cúbica e acha o extremo EXATO; o teste, de propósito, não copia essa conta — ele
/// varre a curva. O erro da varredura cai com `1/n²` (a 64 amostras erra 2e-5 do lado
/// de DENTRO, o que faria o teste acusar a forma de não encostar na borda).
const SAMPLES: u32 = 512;

/// A bbox VERDADEIRA (a CURVA, não as âncoras) de um contorno fechado. Medir só as
/// âncoras subestima a forma: a Bézier passeia fora do casco delas, e é justamente aí
/// que o arco da faixa toca a borda da caixa.
fn curve_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let n = p.verts.len();
    for i in 0..n {
        let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
        for s in 0..=SAMPLES {
            let q = crate::cubic_at(
                a.anchor,
                a.out_handle,
                b.in_handle,
                b.anchor,
                f64::from(s) / f64::from(SAMPLES),
            );
            for k in 0..2 {
                lo[k] = lo[k].min(q[k]);
                hi[k] = hi[k].max(q[k]);
            }
        }
    }
    (lo, hi)
}

/// O afim DIAGONAL que o [`fit`] aplicou, recuperado comparando a forma crua com a
/// publicada (`x' = ax·x + bx`). O `fit` é escala + translação por eixo — nunca
/// rotaciona —, então dois vértices bastam. É o que permite ao teste **desfazer** o
/// ajuste e medir o arco onde ele é um CÍRCULO de verdade, sem copiar uma linha da
/// implementação.
fn recover_fit(raw: &VecPath, out: &VecPath) -> [[f64; 2]; 2] {
    let axis = |k: usize| -> [f64; 2] {
        let r0 = raw.verts[0].anchor[k];
        let i = (0..raw.verts.len())
            .max_by(|&x, &y| {
                (raw.verts[x].anchor[k] - r0)
                    .abs()
                    .total_cmp(&(raw.verts[y].anchor[k] - r0).abs())
            })
            .expect("a seta tem vertices");
        let dr = raw.verts[i].anchor[k] - r0;
        assert!(
            dr.abs() > 1e-6,
            "eixo degenerado: nao da para recuperar o fit"
        );
        let scale = (out.verts[i].anchor[k] - out.verts[0].anchor[k]) / dr;
        [scale, out.verts[0].anchor[k] - scale * r0]
    };
    [axis(0), axis(1)]
}

/// Mundo (publicado) → espaço de autoria: desfaz o `fit` e depois o `Unit::p`.
fn to_unit(p: [f64; 2], m: [[f64; 2]; 2]) -> Uv {
    let (cx, cy) = ((A[0] + B[0]) * 0.5, (A[1] + B[1]) * 0.5);
    let (hw, hh) = ((B[0] - A[0]).abs() * 0.5, (B[1] - A[1]).abs() * 0.5);
    let x = (p[0] - m[0][1]) / m[0][0];
    let y = (p[1] - m[1][1]) / m[1][0];
    (((x - cx) / hw + 1.0) * 0.5, (1.0 - (y - cy) / hh) * 0.5)
}

/// **O teste da foto do Enio.** A faixa da seta circular é um ARCO, não um polígono.
///
/// O contorno emitido é reamostrado ENTRE as âncoras (é lá que a corda mente: as
/// âncoras de uma polilinha inscrita ficam *sobre* o círculo, e só o miolo do vão
/// desaba) e comparado com o círculo de verdade, no espaço onde ele é um círculo.
///
/// A escala do defeito, medida: **corda reta de 15° = `8,6e-3·r`** (8,6 px num raio de
/// 1000 — o que ele fotografou) · cúbica de 90° = `2,7e-4·r` · **cúbica de 45° =
/// `4,2e-6·r`**. O teto de `2e-5` abaixo passa 430× longe da corda e 5× perto da
/// cúbica: discrimina os dois regimes sem ser frágil.
#[test]
fn the_curved_arrow_band_is_a_true_arc_not_a_polygon() {
    let c = Curved::resolve(CURVED.0, CURVED.1, CURVED.2, CURVED.3, CURVED.4);
    let raw = c.build(&Unit::of(A, B));
    let out = curved_default();
    assert_eq!(
        raw.verts.len(),
        out.verts.len(),
        "o fit nao mexe na contagem"
    );
    let m = recover_fit(&raw, &out);

    // O arco EXTERNO abre o contorno; o INTERNO o fecha. Entre duas âncoras vizinhas
    // de cada um há exatamente UMA cúbica.
    let n_out = arc_segments(c.span_out);
    let n_in = arc_segments(c.span_in);
    let total = out.verts.len();
    let arcs = [(0_usize, n_out, c.r_out), (total - 1 - n_in, n_in, c.r_in)];

    // O desvio radial máximo dos dois arcos, medido ENTRE as âncoras. `chords` refaz a
    // mesma medida sobre uma cópia POLIGONIZADA (handles colados na âncora — a forma
    // exata do bug antigo), e é o que prova que a régua discrimina: um teste que só
    // olha as âncoras dá verde nas duas, porque a polilinha é INSCRITA no círculo.
    let deviation = |p: &VecPath, chords: bool| {
        let mut worst: f64 = 0.0;
        for (base, n, radius) in arcs {
            for i in base..base + n {
                let (a, b) = (&p.verts[i], &p.verts[i + 1]);
                let (h0, h1) = if chords {
                    (a.anchor, b.anchor)
                } else {
                    (a.out_handle, b.in_handle)
                };
                for s in 1..32 {
                    let w = crate::cubic_at(a.anchor, h0, h1, b.anchor, f64::from(s) / 32.0);
                    let (uu, vv) = to_unit(w, m);
                    let r = (uu - O.0).hypot(vv - O.1);
                    worst = worst.max((r - radius).abs() / radius);
                }
            }
        }
        worst
    };

    let real = deviation(&out, false);
    let polygonised = deviation(&out, true);
    assert!(
        real < 2e-5,
        "desvio radial {real} do raio: a faixa esta POLIGONIZADA (a corda reta de 15 \
         graus dava 8.6e-3 — 8.6 px num raio de 1000; a cubica de 45 graus da 4.2e-6)"
    );
    assert!(
        polygonised > 1e-2 && polygonised > real * 100.0,
        "a regua nao discrimina: a mesma medida na copia poligonizada deu so \
         {polygonised} (a curva de verdade deu {real})"
    );
    // E a rede contra a regressão exata: o arco nasceu com HANDLES.
    let v = &out.verts[0];
    let h = (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]);
    assert!(h > 1e-3, "o arco nasceu polilinha: handle colado na ancora");
}

/// A seta circular ABRAÇA a caixa do gesto — medido na CURVA (âncoras e extremos das
/// cúbicas), que é o que o gizmo desenha. Medir pelo casco dos controles erraria em
/// até 1,3%: a mesma ordem do defeito que este módulo conserta.
#[test]
fn the_curved_arrow_fills_the_gesture_box_measured_on_the_true_curve() {
    for sweep in [90.0, 180.0, 270.0, 350.0, -270.0] {
        for start in [0.0, 45.0, 180.0, 300.0] {
            let p = arrow_curved(A, B, sweep, start, 0.125, 0.25, 0.12);
            let (lo, hi) = curve_bbox(&p);
            for (k, (want_lo, want_hi)) in [(A[0], B[0]), (A[1], B[1])].into_iter().enumerate() {
                assert!(
                    (lo[k] - want_lo).abs() < 1e-5 && (hi[k] - want_hi).abs() < 1e-5,
                    "sweep {sweep} start {start}: eixo {k} vai de {} a {} (queria {want_lo}..{want_hi})",
                    lo[k],
                    hi[k]
                );
            }
        }
    }
}

/// A CABEÇA fica no FIM do sweep e a PONTA vive na linha de centro — não na borda
/// externa da faixa. (Quem chega mais longe do centro é a esquina EXTERNA da cabeça,
/// e ela pousa exatamente no círculo inscrito: é a normalização, não um clamp.)
#[test]
fn the_curved_arrow_tip_ends_the_sweep_on_the_centreline_circle() {
    // O raio da linha de centro NÃO é lido de volta da implementação: é a identidade
    // que a normalização promete (`r_c = 0,5 − head_w/2`), escrita aqui à mão. Um
    // teste que perguntasse o raio à própria forma estaria provando a si mesmo.
    const HEAD_W: f64 = 0.25;
    let r_c = 0.5 - HEAD_W * 0.5;
    for sweep in [270.0, -270.0, 100.0] {
        for start in [0.0, 137.0, 300.0] {
            let c = Curved::resolve(sweep, start, 0.125, HEAD_W, 0.12);
            let tip = (c.tip.0 - O.0, c.tip.1 - O.1);
            let r = tip.0.hypot(tip.1);
            assert!(
                (r - r_c).abs() < 1e-12,
                "a ponta caiu em r={r}, e a linha de centro e {r_c}"
            );
            let want = (start + sweep).rem_euclid(360.0);
            let got = tip.1.atan2(tip.0).to_degrees().rem_euclid(360.0);
            assert!(
                (got - want).abs() < 1e-9 || (got - want).abs() > 360.0 - 1e-9,
                "a ponta esta em {got} graus, e o sweep acaba em {want}"
            );
            // E a esquina externa da cabeça nunca fura o círculo inscrito.
            let g = (c.corner_g.0 - O.0).hypot(c.corner_g.1 - O.1);
            assert!(
                g <= 0.5 + 1e-12,
                "a esquina da cabeca furou a caixa: |G|={g}"
            );
        }
    }
}

/// **O alvo irrefutável.** Os defaults da `circularArrow` do ECMA-376 (espessura
/// 12500, cabeça 12500, `adj2 = 1142319` = 19,039° de sweep de cabeça, `stAng` 180°,
/// 180° de volta) avaliados pela linguagem de guias do padrão dão os números abaixo.
/// Nossa forma fechada tem de reproduzi-los — se ela reproduz o padrão nos defaults
/// dele, a álgebra está certa, e não só "compila e parece uma seta".
///
/// (`head_len` é fração do sweep aqui: 19,039 / 180 = 0,10577.)
#[test]
fn the_closed_form_reproduces_the_ecma_376_worked_example() {
    let c = Curved::resolve(180.0, 180.0, 0.125, 0.25, 19.039 / 180.0);
    let near = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() < 1e-4,
            "{what}: {got} != {want} (o padrao publicado)"
        );
    };
    near(c.r_out, 0.4375, "raio externo");
    near(c.r_in, 0.3125, "raio interno");

    let e = polar(c.r_out, c.start);
    near(e.0, 0.0625, "E.u (a cauda)");
    near(e.1, 0.5, "E.v");
    near(c.tip.0, 0.875, "A.u (a PONTA, na linha de centro)");
    near(c.tip.1, 0.5, "A.v");
    near(c.corner_g.0, 0.97949, "G.u (esquina externa da cabeca)");
    near(c.corner_g.1, 0.37767, "G.v");
    near(c.corner_b.0, 0.72949, "B.u (esquina interna da cabeca)");
    near(c.corner_b.1, 0.37767, "B.v");

    near(c.span_out, 163.763, "sweep do arco externo");
    near(c.ang_c, 336.955 - 360.0, "angulo de C");
    near(c.span_in, -156.955, "sweep do arco interno");
    assert!(c.flare, "com estes numeros a cabeca ABRE (nao degenera)");
}

/// Um sweep NEGATIVO percorre ao contrário — é a seta circular canhota, de graça.
#[test]
fn a_negative_sweep_mirrors_the_circular_arrow() {
    let cw = Curved::resolve(270.0, 180.0, 0.125, 0.25, 0.12);
    let ccw = Curved::resolve(-270.0, 180.0, 0.125, 0.25, 0.12);
    assert!(
        cw.span_out > 0.0 && ccw.span_out < 0.0,
        "os sentidos opostos"
    );
    // Espelho pela horizontal que passa no centro: v ↦ 1 − v (a cauda é a mesma).
    assert!(
        (cw.tip.0 - ccw.tip.0).abs() < 1e-9 && (cw.tip.1 + ccw.tip.1 - 1.0).abs() < 1e-9,
        "a ponta canhota tinha de ser o espelho da destra: {:?} vs {:?}",
        cw.tip,
        ccw.tip
    );
}

/// A seta reta APONTA: a ponta é o único vértice no extremo +X, e ela fica na linha do
/// meio. É o que distingue uma seta de um retângulo com um entalhe.
#[test]
fn the_block_arrow_has_a_single_tip_on_the_right_centreline() {
    let p = arrow_block(A, B, 0.4, 0.4, 1.0);
    let hi = p.verts_all().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    let tips: Vec<&VecVertex> = p
        .verts
        .iter()
        .filter(|v| (v.anchor[0] - hi).abs() < 1e-9)
        .collect();
    assert_eq!(tips.len(), 1, "uma ponta so");
    assert!(
        tips[0].anchor[1].abs() < 1e-9,
        "a ponta esta na linha do meio"
    );
    assert!(
        (hi - B[0]).abs() < 1e-9,
        "a ponta encosta na borda da caixa"
    );
}

/// A cabeça é mais LARGA que a haste — senão não é uma seta, é uma barra. O clamp
/// garante isso mesmo com um `head_w` menor que o `tail` (o usuário pode digitar).
#[test]
fn the_head_is_never_narrower_than_the_tail() {
    let p = arrow_block(A, B, 0.9, 0.4, 0.1); // cabeça pedida MENOR que a haste
    let widest = p
        .verts
        .iter()
        .map(|v| v.anchor[1].abs())
        .fold(0.0, f64::max);
    let tail = 0.9; // meia-espessura pedida, em mundo (a caixa tem meia-altura 1)
    assert!(
        widest >= tail - 1e-9,
        "o contorno inverteria: cabeca {widest} < haste {tail}"
    );
}

/// A seta dupla tem ponta nos DOIS extremos, ambas na linha do meio.
#[test]
fn the_double_arrow_points_both_ways() {
    let p = arrow_double(A, B, 0.4, 0.3, 1.0);
    let lo = p.verts_all().map(|v| v.anchor[0]).fold(f64::MAX, f64::min);
    let hi = p.verts_all().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    assert!(
        (lo - A[0]).abs() < 1e-9 && (hi - B[0]).abs() < 1e-9,
        "as duas bordas"
    );
    let on_axis = p.verts.iter().filter(|v| v.anchor[1].abs() < 1e-9).count();
    assert_eq!(on_axis, 2, "as duas pontas ficam no eixo");
}

/// **A seta em L aponta para CIMA** (mundo Y-para-cima): a ponta é o vértice mais
/// alto, é única, e nasce sobre a haste vertical.
#[test]
fn the_bent_arrow_points_up() {
    for corner in [0.0, 0.35, 1.0] {
        let p = arrow_bent(A, B, 0.25, 0.3, 0.6, corner);
        let hi = p.verts_all().map(|v| v.anchor[1]).fold(f64::MIN, f64::max);
        let tips = p
            .verts
            .iter()
            .filter(|v| (v.anchor[1] - hi).abs() < 1e-9)
            .count();
        assert_eq!(tips, 1, "corner {corner}: uma ponta so");
        assert!(
            (hi - B[1]).abs() < 1e-9,
            "corner {corner}: a ponta encosta no TOPO da caixa ({hi})"
        );
    }
}

/// O cotovelo da seta em L é um par de arcos CONCÊNTRICOS — e com `corner = 0` ele
/// degenera na quina quadrada sem sobrar um vértice solto.
#[test]
fn the_bent_elbow_is_a_true_arc_and_squares_off_at_zero() {
    let round = arrow_bent(A, B, 0.25, 0.3, 0.6, 0.4);
    let curvy = round
        .verts
        .iter()
        .filter(|v| (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]) > 1e-6)
        .count();
    assert!(curvy >= 2, "o cotovelo redondo tem de ter handles: {curvy}");

    let square = arrow_bent(A, B, 0.25, 0.3, 0.6, 0.0);
    for v in square.verts_all() {
        let h = (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]);
        assert!(h < 1e-9, "corner=0 e o cotovelo QUADRADO: handle solto {h}");
    }
    assert_eq!(
        square.verts.len(),
        9,
        "quadrado = 9 quinas (cauda, cotovelo externo, 5 da cabeca, cotovelo interno, volta)"
    );
}

/// O chevron tem entalhe: com `notch = 0` é um pentágono; acima disso, o vértice de
/// trás entra na caixa (é o encaixe do próximo chevron).
#[test]
fn the_chevron_notch_bites_into_the_back_edge() {
    let flat = chevron(A, B, 0.3, 0.0);
    let bitten = chevron(A, B, 0.3, 0.25);
    assert!(
        (flat.verts[5].anchor[0] - A[0]).abs() < 1e-9,
        "sem entalhe, a traseira e reta"
    );
    assert!(
        bitten.verts[5].anchor[0] > flat.verts[5].anchor[0],
        "o entalhe entra na forma"
    );
}

/// Todas as setas cabem na caixa do gesto — medido na CURVA. Uma que vazasse faria a
/// bbox mentir e o gizmo desalinhar do desenho.
#[test]
fn every_arrow_fits_inside_the_gesture_box() {
    let shapes = [
        ("block", arrow_block(A, B, 0.4, 0.4, 1.0)),
        ("double", arrow_double(A, B, 0.4, 0.3, 1.0)),
        ("bent", arrow_bent(A, B, 0.25, 0.3, 0.6, 0.35)),
        ("chevron", chevron(A, B, 0.3, 0.2)),
        ("curved", curved_default()),
    ];
    for (name, p) in shapes {
        let (lo, hi) = curve_bbox(&p);
        assert!(
            lo[0] >= A[0] - 1e-6
                && hi[0] <= B[0] + 1e-6
                && lo[1] >= A[1] - 1e-6
                && hi[1] <= B[1] + 1e-6,
            "{name}: a curva vaza a caixa ({lo:?}..{hi:?})"
        );
    }
}

/// **Determinismo:** cozinhar duas vezes dá os MESMOS bytes. Zero aleatoriedade no
/// cozimento — é o que permite ao undo por snapshot comparar estados por igualdade.
#[test]
fn cooking_twice_gives_the_same_bytes() {
    let mk = || {
        vec![
            arrow_block(A, B, 0.37, 0.41, 0.83),
            arrow_double(A, B, 0.37, 0.29, 0.83),
            arrow_bent(A, B, 0.23, 0.31, 0.61, 0.37),
            chevron(A, B, 0.29, 0.19),
            arrow_curved(A, B, -233.0, 71.0, 0.13, 0.27, 0.11),
        ]
    };
    let (one, two) = (mk(), mk());
    for (a, b) in one.iter().zip(&two) {
        let (x, y) = (
            postcard::to_allocvec(a).expect("serializa"),
            postcard::to_allocvec(b).expect("serializa"),
        );
        assert_eq!(x, y, "o cozimento nao e deterministico");
    }
}

/// Nenhum parâmetro, em nenhum extremo da faixa que a UI publica, produz NaN ou uma
/// forma vazia — o clamp de cada um deles é uma promessa executável.
#[test]
fn the_parameter_extremes_never_degenerate() {
    for &sweep in &[-350.0, -10.0, 0.0, 10.0, 350.0] {
        for &th in &[0.02, 0.5] {
            for &hw in &[0.05, 0.5] {
                for &hl in &[0.02, 0.9] {
                    let p = arrow_curved(A, B, sweep, 40.0, th, hw, hl);
                    assert!(p.verts.len() >= 4, "sweep {sweep}: contorno raquitico");
                    for v in p.verts_all() {
                        assert!(
                            v.anchor.iter().all(|x| x.is_finite())
                                && v.in_handle.iter().all(|x| x.is_finite())
                                && v.out_handle.iter().all(|x| x.is_finite()),
                            "sweep {sweep} th {th} hw {hw} hl {hl}: NaN no contorno"
                        );
                    }
                }
            }
        }
    }
    for &t in &[0.02, 0.9] {
        for &c in &[0.0, 1.0] {
            let p = arrow_bent(A, B, t, 0.6, 1.0, c);
            assert!(
                p.verts_all()
                    .all(|v| v.anchor.iter().all(|x| x.is_finite()))
            );
        }
    }
}
