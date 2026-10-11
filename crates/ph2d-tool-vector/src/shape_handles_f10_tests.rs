//! **O oráculo dos parâmetros da F10** — o Inkscape 1.4.4 (GPL: caixa preta, só a saída) sobre
//! formas NOSSAS, pelo catálogo inteiro (`ph2d_vec_scene::cook`): o liso (*Rounded*), a torção
//! (`arg2`) e a espiral (*Divergence* + *Inner radius*) cozidos pelo Inkscape
//! (`oraculo_parametros/`), e a torção do vale pela mão (`oraculo_alcas/`, a F9). Doc 15 §2.

use super::*;
use ph2d_vec_scene::cook;

fn fixture_f10(name: &str) -> serde_json::Value {
    let p = format!(
        "{}/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_parametros/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .expect("json")
}

/// O `d` do Inkscape (m/M l/L c/C z/Z, relativos e absolutos, repetições implícitas) em cúbicas,
/// já no NOSSO espaço por `map`.
fn svg_cubics(d: &str, map: &dyn Fn([f64; 2]) -> [f64; 2]) -> Vec<[[f64; 2]; 4]> {
    let toks: Vec<String> = d
        .replace(',', " ")
        .split_whitespace()
        .flat_map(|t| {
            let mut out = Vec::new();
            let mut cur = String::new();
            for ch in t.chars() {
                // O `e` de um expoente (`1.2e-5`) é do número; os comandos SVG nunca são `e`.
                if ch.is_ascii_alphabetic() && ch != 'e' && ch != 'E' {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                    out.push(ch.to_string());
                } else {
                    cur.push(ch);
                }
            }
            if !cur.is_empty() {
                out.push(cur);
            }
            out
        })
        .collect();
    let (mut i, mut cmd, mut p, mut start) = (0, 'M', [0.0; 2], [0.0; 2]);
    let mut out = Vec::new();
    let num = |i: &mut usize| -> f64 {
        *i += 1;
        toks[*i - 1].parse().unwrap()
    };
    while i < toks.len() {
        if toks[i].chars().all(|c| c.is_ascii_alphabetic()) {
            cmd = toks[i].chars().next().unwrap();
            i += 1;
            if cmd == 'z' || cmd == 'Z' {
                if p != start {
                    out.push([p, p, start, start]);
                }
                p = start;
                continue;
            }
        }
        let rel = cmd.is_ascii_lowercase();
        let at = |q: [f64; 2], p: [f64; 2]| if rel { [p[0] + q[0], p[1] + q[1]] } else { q };
        match cmd.to_ascii_uppercase() {
            'M' => {
                p = at([num(&mut i), num(&mut i)], p);
                start = p;
                cmd = if rel { 'l' } else { 'L' };
            }
            'L' => {
                let q = at([num(&mut i), num(&mut i)], p);
                out.push([p, p, q, q]);
                p = q;
            }
            'C' => {
                let c1 = at([num(&mut i), num(&mut i)], p);
                let c2 = at([num(&mut i), num(&mut i)], p);
                let q = at([num(&mut i), num(&mut i)], p);
                out.push([p, c1, c2, q]);
                p = q;
            }
            c => panic!("comando SVG {c} fora do que o Inkscape escreve aqui"),
        }
    }
    out.into_iter()
        .map(|c| [map(c[0]), map(c[1]), map(c[2]), map(c[3])])
        .collect()
}

fn path_cubics(p: &VecPath) -> Vec<[[f64; 2]; 4]> {
    let n = p.verts.len();
    let segs = if p.closed { n } else { n - 1 };
    (0..segs)
        .map(|i| {
            let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
            [a.anchor, a.out_handle, b.in_handle, b.anchor]
        })
        .collect()
}

fn flatten(cs: &[[[f64; 2]; 4]]) -> Vec<[f64; 2]> {
    cs.iter()
        .flat_map(|c| (0..=48).map(move |k| cubic_at(c, f64::from(k) / 48.0)))
        .collect()
}

/// A distância de Hausdorff entre as duas curvas (nos dois sentidos, ponto → polilinha).
fn hausdorff(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    hausdorff_one(a, b).max(hausdorff_one(b, a))
}

/// O ponto de `a` mais longe da polilinha `b`.
fn hausdorff_one(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter().map(|&p| dist_to_poly(p, b)).fold(0.0, f64::max)
}

/// O ponto de `pts` mais longe da curva ANALÍTICA `law(t)`, `t ∈ [t0, 1]`, com os pontos na mesma
/// ordem da curva: uma procura grossa numa janela a partir do anterior e o `t` refinado por secção
/// áurea — exacto e linear (com 60 000 amostras da lei o gate da espiral levava 29 s: o R9 de
/// `PROMPT_LINHAS_RAPIDAS_2026-10-11.md`).
fn dist_to_law(pts: &[[f64; 2]], law: &dyn Fn(f64) -> [f64; 2], t0: f64) -> f64 {
    const N: usize = 4_000;
    // Entre duas âncoras do Inkscape (12 cúbicas a 3 voltas) a grossa anda ~330 amostras.
    const AHEAD: usize = 800;
    let t_of = |k: usize| t0 + (1.0 - t0) * k as f64 / N as f64;
    let far = |p: [f64; 2], t: f64| {
        let q = law(t);
        (q[0] - p[0]).hypot(q[1] - p[1])
    };
    let mut j: usize = 0;
    let mut worst: f64 = 0.0;
    for &p in pts {
        let k = (j.saturating_sub(8)..=(j + AHEAD).min(N))
            .min_by(|&x, &y| far(p, t_of(x)).total_cmp(&far(p, t_of(y))))
            .unwrap_or(j);
        j = k;
        let (mut lo, mut hi) = (t_of(k.saturating_sub(1)), t_of((k + 1).min(N)));
        let g = 0.618_033_988_749_895;
        for _ in 0..60 {
            let (a, b) = (hi - g * (hi - lo), lo + g * (hi - lo));
            if far(p, a) < far(p, b) {
                hi = b;
            } else {
                lo = a;
            }
        }
        worst = worst.max(far(p, 0.5 * (lo + hi)));
    }
    worst
}

fn dist_to_poly(p: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    poly.windows(2)
        .map(|w| {
            let (s, e) = (w[0], w[1]);
            let d = [e[0] - s[0], e[1] - s[1]];
            let l2 = d[0] * d[0] + d[1] * d[1];
            let t = if l2 > 0.0 {
                (((p[0] - s[0]) * d[0] + (p[1] - s[1]) * d[1]) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (p[0] - s[0] - d[0] * t).hypot(p[1] - s[1] - d[1] * t)
        })
        .fold(f64::INFINITY, f64::min)
}

/// O documento do Inkscape (Y para baixo, centro `(400, 300)`) no nosso espaço normalizado: a
/// MESMA reflexão dos gates da F9 (as pontas dele à direita do topo, as nossas à esquerda).
fn ink(r: f64) -> impl Fn([f64; 2]) -> [f64; 2] {
    move |p| [(400.0 - p[0]) / r, (300.0 - p[1]) / r]
}

fn values(v: &[f64]) -> ShapeValues {
    let mut out = [0.0; ph2d_vec_scene::MAX_SHAPE_FIELDS];
    out[..v.len()].copy_from_slice(v);
    out
}

fn the_same_curve(name: &str, d: &str, kind: ShapeKind, v: &[f64], r: f64, spin: bool) -> f64 {
    let mut ours = cook(kind, [-1.0, -1.0], [1.0, 1.0], &values(v));
    if spin {
        // A espiral do Inkscape começa às 3h e a nossa às 12h: roda-se a nossa 90°.
        for vx in &mut ours.verts {
            for q in [&mut vx.anchor, &mut vx.in_handle, &mut vx.out_handle] {
                *q = [-q[1], q[0]];
            }
        }
    }
    let h = hausdorff(
        &flatten(&path_cubics(&ours)),
        &flatten(&svg_cubics(d, &ink(r))),
    );
    assert!(h.is_finite(), "{name}");
    h
}

/// ⭐ **O LISO é o *Rounded* do Inkscape** (8 estrelas e polígonos, de −0,25 a 1, cozidos por ele):
/// as alças tangentes de `liso × corda` desenham a MESMA curva.
#[test]
fn smooth_draws_the_inkscape_rounded_star_and_polygon() {
    let mut n = 0;
    for (name, kind) in [
        ("estrela_rounded_randomized", ShapeKind::Star),
        ("poligono_rounded_randomized", ShapeKind::Polygon),
    ] {
        for e in fixture_f10(name)["experiencias"].as_array().unwrap() {
            let p = &e["params"];
            if p["randomized"].as_f64().unwrap() != 0.0 {
                continue; // o acaso não se reproduz — doc 15 §5 (recusa)
            }
            let s = p["rounded"].as_f64().unwrap();
            let v = match kind {
                ShapeKind::Star => vec![5.0, 0.45, 0.0, 0.0, 0.0, 0.0, s, 0.0],
                _ => vec![6.0, 0.0, 0.0, 0.0, s],
            };
            let h = the_same_curve(name, e["d"].as_str().unwrap(), kind, &v, 100.0, false);
            assert!(h < SMOOTH_TOL, "{name} rounded {s}: Hausdorff {h}");
            n += 1;
        }
    }
    assert_eq!(n, 12, "6 lisos × estrela e polígono");
}

/// A curva do Inkscape e a nossa são cúbicas da MESMA lei: sem torção batem a `1e-6`; torcida, a
/// alça dele aponta `0,015°` ao lado da recta vizinho → vizinho (medido: `1,8e-5` da forma, `0,002`
/// px num raio de 100) — sub-pixel, e a lei de omissão (sem torção) é exacta.
const SMOOTH_TOL: f64 = 5e-5;

/// ⭐ **A TORÇÃO é o `arg2` do Inkscape** (3 estrelas, com e sem liso).
#[test]
fn twist_draws_the_inkscape_arg2_star() {
    let f = fixture_f10("estrela_torcida");
    let mut n = 0;
    for e in f["experiencias"].as_array().unwrap() {
        let p = &e["params"];
        let (tw, s) = (
            p["twist_deg_ydown"].as_f64().unwrap(),
            p["rounded"].as_f64().unwrap(),
        );
        let v = [5.0, 0.45, 0.0, 0.0, 0.0, 0.0, s, tw];
        let h = the_same_curve(
            "torcida",
            e["d"].as_str().unwrap(),
            ShapeKind::Star,
            &v,
            100.0,
            false,
        );
        assert!(h < SMOOTH_TOL, "torção {tw}, liso {s}: Hausdorff {h}");
        n += 1;
    }
    assert_eq!(n, 3);
}

/// ⭐ **A ESPIRAL é a do Inkscape**: `r = t^divergência`, `θ = voltas·2π·t`, `t` de `t0` a 1 (12
/// espirais: a divergência 0,5 · 1 · 2 · 4 × o raio interno 0 · 0,25 · 0,5; a nossa expansão =
/// divergência − 1). ⚠️ A régua são as ÂNCORAS dele — amostras exactas da lei (a 1e-6 dela,
/// medido) —, não as cúbicas: ele ajusta 4 por volta e afasta-se até `0,006` da lei; nós 16.
#[test]
fn spiral_ex_matches_the_inkscape_divergence_and_inner_radius() {
    let (mut worst_anchor, mut worst_curve): (f64, f64) = (0.0, 0.0);
    let f = fixture_f10("espiral_expansion_t0");
    for e in f["experiencias"].as_array().unwrap() {
        let p = &e["params"];
        let (div, t0) = (p["expansion"].as_f64().unwrap(), p["t0"].as_f64().unwrap());
        let mut ours = cook(
            ShapeKind::Spiral,
            [-1.0, -1.0],
            [1.0, 1.0],
            &values(&[3.0, div - 1.0, t0]),
        );
        // A espiral do Inkscape começa às 3h e a nossa às 12h: roda-se a nossa 90°.
        for vx in &mut ours.verts {
            for q in [&mut vx.anchor, &mut vx.in_handle, &mut vx.out_handle] {
                *q = [-q[1], q[0]];
            }
        }
        let ours = flatten(&path_cubics(&ours));
        let theirs = svg_cubics(e["d"].as_str().unwrap(), &ink(150.0));
        let anchors: Vec<[f64; 2]> = theirs.iter().flat_map(|c| [c[0], c[3]]).collect();
        // A lei, densa, no referencial do Inkscape (3h, anti-horário depois da reflexão).
        let law = |t: f64| {
            let (a, r) = (
                std::f64::consts::TAU * 3.0 * t + std::f64::consts::PI,
                t.powf(div),
            );
            [r * a.cos(), r * a.sin()]
        };
        let a = dist_to_law(&anchors, &law, t0);
        let ours_law = dist_to_law(&ours, &law, t0);
        assert!(
            a < SPIRAL_LAW_TOL,
            "divergência {div}, t0 {t0}: a âncora do Inkscape a {a} da lei"
        );
        assert!(
            ours_law < SPIRAL_LAW_TOL,
            "divergência {div}, t0 {t0}: a nossa a {ours_law} da lei"
        );
        worst_anchor = worst_anchor.max(a);
        worst_curve = worst_curve.max(ours_law);
    }
    eprintln!(
        "[espiral] âncoras do Inkscape à lei ≤ {worst_anchor:.2e}; a nossa curva à lei ≤ {worst_curve:.2e}"
    );
}

/// A lei contra as âncoras dele e contra a nossa curva (a distância à lei é exacta: o `t` refinado).
const SPIRAL_LAW_TOL: f64 = 1e-3;

/// ⭐ **O vale da estrela NUNCA torce** — o produto do dono decide a lei, não o oráculo: nem o
/// Affinity nem o Illustrator rodam a estrela pelo raio interior (as fotos dos dois: só radial, com
/// o círculo tracejado «R2» no Illustrator). O Inkscape torce (o `arg2`), e a 1.ª redacção da F10
/// copiou-o — o dono: *«a estrela ficou horrível»*. A torção vive no cartão.
#[test]
fn the_star_valley_never_twists() {
    let k = ShapeKind::Star;
    let ck = super::tests::cooker(k, [1.0, 1.0]);
    let v = k.defaults();
    let i = handles_of(k).iter().position(|h| h.fields == [1]).unwrap();
    let q = handle_point(&handles_of(k)[i], &v, [1.0, 1.0], &ck).unwrap();
    // A mão desvia 40° do raio do vale e afasta-se: só o raio muda.
    let a = q[1].atan2(q[0]) + 40f64.to_radians();
    let r = q[0].hypot(q[1]) * 1.3;
    let mut d = HandleDrag::start(i, v, q, q, [1.0, 1.0]);
    let out = drag(
        k,
        &mut d,
        [r * a.cos(), r * a.sin()],
        Mods::default(),
        [1.0, 1.0],
        1.0,
        &ck,
    );
    assert_eq!(out[7], 0.0, "a torção ficou");
    assert!(
        (out[1] - v[1] * 1.3).abs() < 2e-3,
        "o raio interior seguiu a distância: {}",
        out[1]
    );
}

/// ⭐ **A ponta de cima arredonda, e o ponto DESCE pelo eixo dela** (o *Outer circle* do Affinity):
/// a alça é o cruzamento do contorno com o eixo da ponta; arrastá-la para o centro arredonda.
#[test]
fn the_tip_handle_slides_down_the_tip_axis_and_rounds_the_tips() {
    let k = ShapeKind::Star;
    let half = [1.0, 1.0];
    let ck = super::tests::cooker(k, half);
    let v = k.defaults();
    let i = handles_of(k).iter().position(|h| h.fields == [2]).unwrap();
    let h = &handles_of(k)[i];
    let q = handle_point(h, &v, half, &ck).unwrap();
    assert!(
        q[0].abs() < 1e-9 && (q[1] - 1.0).abs() < 1e-9,
        "sem raio é a ponta: {q:?}"
    );
    let p = [0.0, 0.85];
    let mut d = HandleDrag::start(i, v, q, q, half);
    let out = drag(k, &mut d, p, Mods::default(), half, 1.0, &ck);
    assert!(out[2] > 0.0, "a ponta arredondou");
    let r = handle_point(h, &out, half, &ck).unwrap();
    assert!(
        r[0].abs() < 1e-6 && (r[1] - 0.85).abs() < 1e-3,
        "o ponto ficou no eixo, onde a mão o largou: {r:?}"
    );
    // Desenhada `park_at` para DENTRO do topo do contorno — que é onde a alça do meio-topo da caixa
    // mora (a caixa abraça o contorno COZIDO): sem e com arredondamento (a foto da cena 16).
    for vals in [v, out] {
        let top = handle_point(h, &vals, half, &ck).unwrap()[1];
        let g = handle_grip(k, h, &vals, half, 1.0, 0.1, &ck).unwrap();
        assert!(
            (g[1] - (top - 0.1)).abs() < 1e-9 && g[0].abs() < 1e-9,
            "desenhada para dentro: {g:?} (topo {top})"
        );
    }
}

/// ⭐ **O ⇕ das pontas** (o *Points* do Illustrator): na borda direita da caixa; arrastar para CIMA
/// soma pontas, para baixo tira — um degrau por 1/24 da altura.
#[test]
fn the_points_widget_steps_the_count_up_and_down() {
    for k in [ShapeKind::Star, ShapeKind::Polygon] {
        let half = [1.0, 1.0];
        let ck = super::tests::cooker(k, half);
        let v = k.defaults();
        let i = handles_of(k)
            .iter()
            .position(|h| h.law == Law::Steps)
            .unwrap();
        let q = handle_point(&handles_of(k)[i], &v, half, &ck).unwrap();
        // Na borda direita da caixa VISÍVEL (a do contorno), a um quarto do topo dela.
        let ys: Vec<f64> = ck(&v).verts.iter().map(|x| x.anchor[1]).collect();
        let right = ck(&v)
            .verts
            .iter()
            .map(|x| x.anchor[0])
            .fold(f64::MIN, f64::max);
        let (top, bottom) = (
            ys.iter().copied().fold(f64::MIN, f64::max),
            ys.iter().copied().fold(f64::MAX, f64::min),
        );
        assert!(
            (q[0] - right).abs() < 1e-9,
            "{k:?}: na borda direita visível: {q:?}"
        );
        assert!(
            (q[1] - (top - (top - bottom) / 4.0)).abs() < 1e-9,
            "{k:?}: a um quarto do topo: {q:?}"
        );
        let step = 2.0 / 24.0;
        for (dy, want) in [(3.0 * step, 3.0), (-2.0 * step, -2.0), (0.4 * step, 0.0)] {
            let mut d = HandleDrag::start(i, v, q, q, half);
            let out = drag(
                k,
                &mut d,
                [q[0] + 0.3, q[1] + dy],
                Mods::default(),
                half,
                1.0,
                &ck,
            );
            assert_eq!(out[0], v[0] + want, "{k:?}: dy {dy}");
        }
    }
}

/// A estrela NOVA nasce REGULAR (os ombros alinhados — a das fotos do Affinity e do Illustrator):
/// os dois vales de cima na mesma altura que as pontas dos lados.
#[test]
fn a_new_star_is_born_regular() {
    let p = cook(
        ShapeKind::Star,
        [-1.0, -1.0],
        [1.0, 1.0],
        &ShapeKind::Star.defaults(),
    );
    let (side_tip, valley) = (p.verts[2].anchor, p.verts[1].anchor);
    assert!(
        (side_tip[1] - valley[1]).abs() < 1e-12,
        "{side_tip:?} {valley:?}"
    );
}

/// ⭐ **Alt no trapézio: só o lado agarrado** (o *Left/Right point* do Affinity) — sem Alt os dois
/// cantos curtos andam juntos; com Alt o esquerdo anda e o direito fica onde estava, e vice-versa.
#[test]
fn alt_moves_only_the_grabbed_side_of_the_trapezoid() {
    let k = ShapeKind::Trapezoid;
    let half = [2.0, 1.0];
    let ck = super::tests::cooker(k, half);
    let corner = |v: &ShapeValues, i: usize| ck(v).verts[i].anchor;
    for (h, other) in [(0, 1), (1, 0)] {
        let v = k.defaults();
        let grip = handle_point(&handles_of(k)[h], &v, half, &ck).unwrap();
        let mut d = HandleDrag::start(h, v, grip, grip, half);
        let push = if h == 0 { 0.3 } else { -0.3 };
        let p = [grip[0] + push, grip[1]];
        let alone = drag(
            k,
            &mut d,
            p,
            Mods {
                alone: true,
                ..Mods::default()
            },
            half,
            1.0,
            &ck,
        );
        assert!(
            (corner(&alone, h)[0] - p[0]).abs() < 1e-6,
            "o lado {h} foi à mão: {:?}",
            corner(&alone, h)
        );
        assert!(
            (corner(&alone, other)[0] - corner(&v, other)[0]).abs() < 1e-9,
            "Alt: o outro lado ({other}) ficou"
        );
        let mut d = HandleDrag::start(h, v, grip, grip, half);
        let both = drag(k, &mut d, p, Mods::default(), half, 1.0, &ck);
        assert!(
            (corner(&both, other)[0] - corner(&v, other)[0]).abs() > 0.1,
            "sem Alt os dois andam juntos"
        );
    }
}

/// ⭐ **O meio do losango desenha-se para DENTRO** — o vértice da direita vive sobre a aresta da
/// caixa, onde mora a alça de escala: a alça dele fica `park_at` para o centro (e a pega mede-se
/// ao ponto verdadeiro: agarrar e soltar não muda nada).
#[test]
fn the_diamond_middle_handle_is_drawn_inside_the_box_edge() {
    let k = ShapeKind::Diamond;
    let half = [2.0, 1.0];
    let ck = super::tests::cooker(k, half);
    let v = k.defaults();
    let h = &handles_of(k)[0];
    let truth = handle_point(h, &v, half, &ck).unwrap();
    let grip = handle_grip(k, h, &v, half, 1.0, 0.3, &ck).unwrap();
    assert!(
        (truth[0] - half[0]).abs() < 1e-9,
        "o vértice está na aresta da caixa"
    );
    assert!(
        grip[0] < half[0] - 0.29,
        "a alça desenha-se para dentro: {grip:?}"
    );
    let mut d = HandleDrag::start(0, v, grip, truth, half);
    let held = drag(k, &mut d, grip, Mods::default(), half, 1.0, &ck);
    assert!(
        (held[0] - v[0]).abs() < 1e-9,
        "agarrar e soltar não muda o meio"
    );
}

/// ⭐ **A alça do canto do polígono pousa no contorno também com a aresta CURVA** — ela é o
/// cruzamento do contorno com o eixo do vértice de cima (`Spot::Apex`), e a envergadura não a tira
/// de lá; um raio maior desce-a pelo eixo.
#[test]
fn the_corner_handle_still_rides_a_curved_edge() {
    let k = ShapeKind::Polygon;
    let half = [1.0, 1.0];
    let ck = super::tests::cooker(k, half);
    let mut v = k.defaults();
    v[1] = 0.15; // o raio do canto (mundo)
    v[3] = 0.4; // a envergadura
    let h = &handles_of(k)[handles_of(k).iter().position(|h| h.fields == [1]).unwrap()];
    let p = handle_point(h, &v, half, &ck).expect("a alça do canto existe");
    assert!(p[0].abs() < 1e-9, "no eixo do vértice de cima: {p:?}");
    assert!(
        super::tests::dist_to_outline(&ck(&v), p) < 1e-6,
        "no contorno curvo: {p:?}"
    );
    let mut big = v;
    big[1] = 0.3;
    let q = handle_point(h, &big, half, &ck).unwrap();
    assert!(
        q[1] < p[1] - 1e-3,
        "um raio maior desce o ponto pelo eixo: {q:?} {p:?}"
    );
}

/// ⭐ **O eixo pode cortar o contorno DUAS vezes** (uma estrela muito curva ou torcida, uma ponta que
/// dobra): o ponto da quina é o cruzamento mais PERTO dela em distância ao centro, não outro.
#[test]
fn the_apex_is_the_crossing_nearest_its_corner() {
    use ph2d_vec_scene::VecVertex;
    // Um contorno que cruza o eixo +y a 0,5 e a 1,0 (um «M» fechado por baixo).
    let p = VecPath {
        verts: [
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.5],
            [0.2, 1.0],
            [-0.2, 1.0],
            [0.2, 0.5],
            [-1.0, 0.5],
        ]
        .into_iter()
        .map(VecVertex::corner)
        .collect(),
        closed: true,
        ..VecPath::default()
    };
    let near = |q: [f64; 2]| super::spots::spoke_hit(&p, q).unwrap();
    assert!(
        (near([0.0, 1.0])[1] - 1.0).abs() < 1e-9,
        "a quina a 1,0: {:?}",
        near([0.0, 1.0])
    );
    assert!(
        (near([0.0, 0.5])[1] - 0.5).abs() < 1e-9,
        "a quina a 0,5: {:?}",
        near([0.0, 0.5])
    );
}
