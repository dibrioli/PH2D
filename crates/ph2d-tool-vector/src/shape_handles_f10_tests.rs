//! **O oráculo dos parâmetros da F10** — o Inkscape 1.4.4 (GPL: caixa preta, só a saída) sobre
//! formas NOSSAS, pelo catálogo inteiro (`ph2d_vec_scene::cook`): o liso (*Rounded*), a torção
//! (`arg2`) e a espiral (*Divergence* + *Inner radius*) cozidos pelo Inkscape
//! (`oraculo_parametros/`), e a torção do vale pela mão (`oraculo_alcas/`, a F9). Doc 15 §2.

use super::tests::{attr, fixture, pointer, replay};
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
        let law: Vec<[f64; 2]> = (0..=60_000)
            .map(|k| {
                let t = t0 + (1.0 - t0) * f64::from(k) / 60_000.0;
                let (a, r) = (
                    std::f64::consts::TAU * 3.0 * t + std::f64::consts::PI,
                    t.powf(div),
                );
                [r * a.cos(), r * a.sin()]
            })
            .collect();
        let a = hausdorff_one(&anchors, &law);
        let ours_law = hausdorff_one(&ours, &law);
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

/// A lei contra as âncoras dele e contra a nossa curva (a amostragem densa da lei é `≈5e-5`).
const SPIRAL_LAW_TOL: f64 = 1e-3;

/// ⭐ **O vale torce com a mão, como o do Inkscape** (os 6 arrastos do vale da F9, em 2 estrelas,
/// direita e rodada 25°): o arrasto sem modificador escreve o raio E a torção; Ctrl (o Ctrl dele:
/// sem torcer) só o raio. A reflexão da F9 preserva o sentido dos ângulos.
#[test]
fn the_star_valley_twists_with_the_hand_like_inkscape() {
    let mut n = 0;
    for (name, rot) in [("star5", 0.0), ("star5_rot25", 25.0)] {
        let f = fixture(name);
        for e in f["experiencias"].as_array().unwrap() {
            let nome = e["nome"].as_str().unwrap();
            if !nome.starts_with("inner_") || e["mod"] == "shift" {
                continue;
            }
            let m = |p: [f64; 2]| [400.0 - p[0], 300.0 - p[1]];
            let (a, b) = (
                m(pointer(&f, e, "alca_tela", rot)),
                m(pointer(&f, e, "alvo_tela", rot)),
            );
            let mut v = ShapeKind::Star.defaults();
            v[1] = 0.45;
            let valley = handles_of(ShapeKind::Star)
                .iter()
                .position(|h| matches!(h.law, Law::Polar { .. }))
                .unwrap();
            let mods = Mods {
                pairs: e["mod"] == "ctrl",
                ..Mods::default()
            };
            let r = replay(ShapeKind::Star, valley, v, [100.0, 100.0], a, b, mods);
            let want = (attr(e, "sodipodi:arg2") - attr(e, "sodipodi:arg1")).to_degrees() - 36.0;
            assert!(
                (r[7] - want).abs() < 0.3,
                "{name}/{nome}: torção {} contra {want}",
                r[7]
            );
            n += 1;
        }
    }
    assert_eq!(n, 12);
}
