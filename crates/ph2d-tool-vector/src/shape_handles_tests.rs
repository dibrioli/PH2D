//! Os gates da lei das alças (F9): onde cada alça nasce, para onde a mão a leva, e os números do
//! Inkscape 1.4.4 reproduzidos com os NOSSOS parâmetros (`oraculo_alcas/fixtures`).

use super::*;
use ph2d_vec_scene::{ALL_SHAPES, cook};

const HALF: [f64; 2] = [3.0, 2.0];
const PX: f64 = 0.01;

pub(super) fn cooker(kind: ShapeKind, half: [f64; 2]) -> impl Fn(&ShapeValues) -> VecPath {
    move |v: &ShapeValues| cook(kind, [-half[0], -half[1]], [half[0], half[1]], v)
}

/// A distância de `q` ao contorno de `p` (as cúbicas amostradas, sub-contornos incluídos).
fn dist_to_outline(p: &VecPath, q: [f64; 2]) -> f64 {
    let mut best = f64::INFINITY;
    let mut run = |vs: &[ph2d_vec_scene::VecVertex], closed: bool| {
        let n = vs.len();
        let segs = if closed { n } else { n.saturating_sub(1) };
        for k in 0..segs {
            let (a, b) = (&vs[k], &vs[(k + 1) % n]);
            let pts = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            for s in 0..=256 {
                let t = f64::from(s) / 256.0;
                let u = 1.0 - t;
                let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                let x = (0..4).map(|i| w[i] * pts[i][0]).sum::<f64>();
                let y = (0..4).map(|i| w[i] * pts[i][1]).sum::<f64>();
                best = best.min((x - q[0]).hypot(y - q[1]));
            }
        }
        if n == 1 {
            best = best.min((vs[0].anchor[0] - q[0]).hypot(vs[0].anchor[1] - q[1]));
        }
    };
    run(&p.verts, p.closed);
    for s in &p.subpaths {
        run(&s.verts, s.closed);
    }
    best
}

/// Valores de prova para os campos de `h`: o de fábrica e três pontos da faixa.
fn variants(kind: ShapeKind, h: &HandleDesc) -> Vec<ShapeValues> {
    let mut out = vec![kind.defaults()];
    for t in [0.1, 0.35, 0.7] {
        let mut v = kind.defaults();
        for &i in h.fields {
            let (lo, hi, int) = range_world(kind, i, PX).expect("campo do descritor");
            let x = lo + (hi - lo) * t;
            v[i] = if int { x.round() } else { x };
        }
        out.push(v);
    }
    out
}

/// ⭐ Toda alça de toda forma pousa NO contorno que o `cook` desenha (ou no contorno VIVO, para
/// as que se leem nele) — o que se agarra é o que se vê. E todo campo com alça é do descritor.
#[test]
fn every_handle_sits_on_its_outline() {
    let mut n = 0;
    for &kind in ALL_SHAPES {
        let ck = cooker(kind, HALF);
        let nf = shapes::desc(kind).fields.len();
        for (k, h) in handles_of(kind).iter().enumerate() {
            assert!(
                h.fields.iter().all(|&i| i < nf),
                "{kind:?} alça {k}: campo fora do descritor"
            );
            for v in variants(kind, h) {
                let q = handle_point(h, &v, HALF, &ck)
                    .unwrap_or_else(|| panic!("{kind:?} alça {k}: sem ponto em {v:?}"));
                let on = if h.sharp.is_empty() || matches!(h.spot, Spot::Rounding { .. }) {
                    ck(&v)
                } else {
                    ck(&zeroed(&v, h.sharp))
                };
                // Um furo ZERO não tem contorno: a alça dele mora no CENTRO (o *Hole radius* do
                // Affinity nasce lá).
                let d = match h.spot {
                    Spot::Ellipse { scale: Some(i), .. } if v[i] <= 0.0 => q[0].hypot(q[1]),
                    _ => dist_to_outline(&on, q),
                };
                assert!(d < 1e-6, "{kind:?} alça {k}: {d} fora do contorno em {v:?}");
                n += 1;
            }
        }
    }
    assert!(n > 0, "nenhuma alça medida — a tabela está vazia?");
}

/// ⭐ A mão leva a alça para onde a larga: partindo do valor de fábrica, largar a mão sobre o
/// ponto que OUTRO valor dá devolve esse ponto (as métricas de perto e de raio) ou essa contagem.
#[test]
fn the_hand_takes_the_handle_where_it_is_dropped() {
    for &kind in ALL_SHAPES {
        let ck = cooker(kind, HALF);
        for (k, h) in handles_of(kind).iter().enumerate() {
            if !matches!(
                h.law,
                Law::Nearest | Law::Radial | Law::Count | Law::BoxPoint
            ) {
                continue;
            }
            let def = kind.defaults();
            let Some(q0) = handle_point(h, &def, HALF, &ck) else {
                continue;
            };
            for b in variants(kind, h).into_iter().skip(1) {
                let qb = handle_point(h, &b, HALF, &ck).expect("ponto do alvo");
                let mut d = HandleDrag::start(k, def, q0, q0, HALF);
                let r = drag(kind, &mut d, qb, Mods::default(), HALF, PX, &ck);
                let qr = handle_point(h, &r, HALF, &ck).expect("ponto do resultado");
                match h.law {
                    Law::Count => assert_eq!(
                        h.fields.iter().map(|&i| r[i]).collect::<Vec<_>>(),
                        h.fields.iter().map(|&i| b[i]).collect::<Vec<_>>(),
                        "{kind:?} alça {k}: a contagem não seguiu o ângulo"
                    ),
                    // O ponto da caixa: os campos andam o que a mão andou, em frações da caixa.
                    Law::BoxPoint => {
                        let f = |i: usize| range_world(kind, h.fields[i], PX).unwrap();
                        let (fu, fv) = (f(0), f(1));
                        let u = def[h.fields[0]] + (qb[0] - q0[0]) / (2.0 * HALF[0]);
                        let w = def[h.fields[1]] - (qb[1] - q0[1]) / (2.0 * HALF[1]);
                        assert!(
                            (r[h.fields[0]] - u.clamp(fu.0, fu.1)).abs() < 1e-12
                                && (r[h.fields[1]] - w.clamp(fv.0, fv.1)).abs() < 1e-12,
                            "{kind:?} alça {k}: o ponto da caixa não andou o que a mão andou"
                        );
                    }
                    Law::Radial => {
                        let rr = |x: [f64; 2]| (x[0] / HALF[0]).hypot(x[1] / HALF[1]);
                        assert!(
                            (rr(qr) - rr(qb)).abs() < 1e-7,
                            "{kind:?} alça {k}: raio {} ≠ {}",
                            rr(qr),
                            rr(qb)
                        );
                    }
                    _ => assert!(
                        (qr[0] - qb[0]).hypot(qr[1] - qb[1]) < 1e-6,
                        "{kind:?} alça {k}: largada em {qb:?}, ficou em {qr:?}"
                    ),
                }
            }
        }
    }
}

/// A pega viaja com a mão: agarrar a alça FORA do centro e não mexer não muda nada.
#[test]
fn grabbing_a_handle_off_centre_does_not_move_it() {
    for &kind in ALL_SHAPES {
        let ck = cooker(kind, HALF);
        for (k, h) in handles_of(kind).iter().enumerate() {
            let v = variants(kind, h)[2];
            let Some(q) = handle_point(h, &v, HALF, &ck) else {
                continue;
            };
            let p = [q[0] + 0.03, q[1] - 0.02];
            let mut d = HandleDrag::start(k, v, p, q, HALF);
            let r = drag(kind, &mut d, p, Mods::default(), HALF, PX, &ck);
            for &i in h.fields {
                assert!(
                    (r[i] - v[i]).abs() < 1e-6 * (1.0 + v[i].abs()),
                    "{kind:?} alça {k}: o campo {i} saltou de {} para {} sem a mão andar",
                    v[i],
                    r[i]
                );
            }
        }
    }
}

/// A alça de canto com raio ZERO estaciona a `park_at` da quina, ao longo da aresta — e a de
/// raio largo fica no ponto verdadeiro.
#[test]
fn a_corner_handle_parks_off_the_box_corner() {
    let kind = ShapeKind::RoundRect;
    let ck = cooker(kind, HALF);
    let h = &handles_of(kind)[0];
    let mut v = kind.defaults();
    v[0] = 0.0;
    let corner = handle_point(h, &v, HALF, &ck).unwrap();
    assert!((corner[0] + HALF[0]).abs() < 1e-9 && (corner[1] - HALF[1]).abs() < 1e-9);
    let g = handle_grip(kind, h, &v, HALF, PX, 0.25, &ck).unwrap();
    assert!(
        (g[0] - (corner[0] + 0.25)).abs() < 1e-9 && (g[1] - corner[1]).abs() < 1e-9,
        "{g:?}"
    );
    v[0] = 1.0;
    let g = handle_grip(kind, h, &v, HALF, PX, 0.25, &ck).unwrap();
    assert!((g[0] - (corner[0] + 1.0)).abs() < 1e-9, "{g:?}");
}

/// O Alt arredonda SÓ o canto agarrado: os outros guardam o raio deles — também quando o
/// agarrado é o do raio base (os desvios compensam).
#[test]
fn alt_rounds_only_the_grabbed_corner() {
    let kind = ShapeKind::RoundRect;
    let ck = cooker(kind, HALF);
    let mut v = kind.defaults();
    v[0] = 0.5;
    v[2] = 0.2; // BR mais redondo
    let radii = |v: &ShapeValues| ph2d_vec_scene::round_rect_radii(v[0], [v[1], v[2], v[3]]);
    for k in 0..4 {
        let h = &handles_of(kind)[k];
        let q = handle_point(h, &v, HALF, &ck).unwrap();
        let mut d = HandleDrag::start(k, v, q, q, HALF);
        // Mais para dentro ao longo da aresta horizontal (para o centro em x).
        let p = [q[0] - q[0].signum() * 0.4, q[1]];
        let alone = drag(
            kind,
            &mut d,
            p,
            Mods {
                alone: true,
                ..Mods::default()
            },
            HALF,
            PX,
            &ck,
        );
        let (before, after) = (radii(&v), radii(&alone));
        // A ordem dos raios é [TL, TR, BR, BL]; a das alças, TL, TR, BR, BL.
        for c in 0..4 {
            if c == k {
                assert!(
                    (after[c] - (before[c] + 0.4)).abs() < 1e-6,
                    "canto {k}: {after:?}"
                );
            } else {
                assert!(
                    (after[c] - before[c]).abs() < 1e-9,
                    "canto {k} mexeu o {c}: {after:?}"
                );
            }
        }
        let mut d = HandleDrag::start(k, v, q, q, HALF);
        let all = drag(kind, &mut d, p, Mods::default(), HALF, PX, &ck);
        let after = radii(&all);
        for c in 0..4 {
            assert!(
                (after[c] - (before[c] + 0.4)).abs() < 1e-6,
                "todos, canto {k}: {after:?}"
            );
        }
    }
}

fn spin(kind: ShapeKind, h: usize, v: ShapeValues, turn_deg: f64, mods: Mods) -> ShapeValues {
    let ck = cooker(kind, HALF);
    let hd = &handles_of(kind)[h];
    let q = handle_point(hd, &v, HALF, &ck).unwrap();
    let mut d = HandleDrag::start(h, v, q, q, HALF);
    let a0 = norm_angle(q, HALF);
    let r = (q[0] / HALF[0]).hypot(q[1] / HALF[1]);
    let steps = 64;
    let mut out = v;
    for s in 1..=steps {
        let a = (a0 + turn_deg * f64::from(s) / f64::from(steps)).to_radians();
        let p = [HALF[0] * r * a.cos(), HALF[1] * r * a.sin()];
        out = drag(kind, &mut d, p, mods, HALF, PX, &ck);
    }
    out
}

/// A espiral conta VOLTAS: duas voltas inteiras da mão no sentido do enrolar somam duas; meia
/// volta não soma nada; e a faixa do descritor manda (1..8).
#[test]
fn the_spiral_counts_whole_turns_of_the_hand() {
    let k = ShapeKind::Spiral;
    let v = k.defaults();
    assert_eq!(spin(k, 0, v, 720.0, Mods::default())[0], v[0] + 2.0);
    assert_eq!(spin(k, 0, v, -360.0, Mods::default())[0], v[0] - 1.0);
    assert_eq!(spin(k, 0, v, 170.0, Mods::default())[0], v[0]);
    assert_eq!(spin(k, 0, v, 360.0 * 20.0, Mods::default())[0], 8.0);
}

/// O fim do arco segue o ângulo da mão sem saltar: de uma elipse INTEIRA, recuar 90° abre uma
/// pizza de 270°; avançar não passa da volta inteira.
#[test]
fn the_arc_end_follows_the_hand_and_saturates_at_a_full_turn() {
    let k = ShapeKind::Ellipse;
    let v = k.defaults();
    let end = handles_of(k)
        .iter()
        .position(|h| matches!(h.law, Law::Sweep { .. }))
        .unwrap();
    assert!((spin(k, end, v, -90.0, Mods::default())[0] - 270.0).abs() < 1e-6);
    assert!((spin(k, end, v, 45.0, Mods::default())[0] - 360.0).abs() < 1e-9);
    // Com Shift, o FIM cai em múltiplos de 15°.
    let s = spin(
        k,
        end,
        v,
        -97.0,
        Mods {
            snap: true,
            ..Mods::default()
        },
    );
    assert!((s[0] - 270.0).abs() < 1e-9, "{}", s[0]);
}

/// O início anda com o fim PARADO; com Ctrl («em pares») o arco roda inteiro.
#[test]
fn the_arc_start_keeps_the_end_unless_ctrl_rotates_both() {
    let k = ShapeKind::Pie;
    let v = k.defaults(); // 180° a partir de 0°
    let st = handles_of(k)
        .iter()
        .position(|h| matches!(h.law, Law::ArcStart { .. }))
        .unwrap();
    let r = spin(k, st, v, 40.0, Mods::default());
    assert!(
        (r[1] - 40.0).abs() < 1e-6 && (r[0] - 140.0).abs() < 1e-6,
        "{r:?}"
    );
    let r = spin(
        k,
        st,
        v,
        40.0,
        Mods {
            pairs: true,
            ..Mods::default()
        },
    );
    assert!(
        (r[1] - 40.0).abs() < 1e-6 && (r[0] - 180.0).abs() < 1e-9,
        "{r:?}"
    );
    let r = spin(k, st, v, -30.0, Mods::default());
    assert!(
        (r[1] - 330.0).abs() < 1e-6 && (r[0] - 210.0).abs() < 1e-6,
        "o início dá a volta: {r:?}"
    );
}

/// O duplo-clique repõe SÓ a alça clicada.
#[test]
fn a_double_click_resets_only_that_handle() {
    let k = ShapeKind::Star;
    let mut v = k.defaults();
    v[0] = 9.0;
    v[1] = 0.3;
    let inner = handles_of(k)
        .iter()
        .position(|h| matches!(h.law, Law::Polar { .. }))
        .unwrap();
    let r = reset(k, inner, &v);
    assert_eq!(r[1], k.defaults()[1]);
    assert_eq!(r[0], 9.0);
}

/// Cada campo do descritor ou tem alça ou está CONTADO como sem gesto (fila ②) — e os sem gesto
/// das formas principais são os que o doc 13 §3 nomeia.
#[test]
fn the_fields_without_a_gesture_are_counted_not_listed() {
    let none = |k: ShapeKind| fields_without_handle(k).collect::<Vec<_>>();
    use ShapeKind as K;
    // A lista EXACTA por forma; toda forma fora dela tem alça para todo campo.
    let without: &[(ShapeKind, &[usize])] = &[
        // A suavização do canto: um perfil, não um ponto; o tipo do canto (F10) é uma ESCOLHA.
        (K::RoundRect, &[4, 5]),
        // F10: o tipo do canto (escolha) e o liso (um perfil) — o cartão lateral.
        (K::Polygon, &[2, 4]),
        (K::Star, &[4, 6]),
        // F10: a divergência da espiral e a envergadura dos flancos da engrenagem.
        (K::Spiral, &[1]),
        (K::Gear, &[4]),
        // A faixa do rabo encolhe a elipse inteira: nenhum vértice anda só com ela.
        (K::SpeechOval, &[3]),
        // O número de bolhas: a bolha 0 não muda com ele, as outras somem com uma só.
        (K::Thought, &[2]),
        // A desordem radial e angular: nenhum ponto a representa.
        (K::Burst, &[2, 3]),
        // O fofo: o `refit` normaliza a nuvem e o vale estável quase não anda; a suavização dos
        // vales e a irregularidade: nenhum ponto as representa.
        (K::Cloud, &[1, 2, 3]),
        // O ponto de vista é uma ESCOLHA (fila ②).
        (K::IsoCube, &[2]),
        (K::IsoCone, &[1]),
        (K::IsoPyramid, &[2]),
    ];
    for &k in ALL_SHAPES {
        let want = without.iter().find(|w| w.0 == k).map_or(&[][..], |w| w.1);
        assert_eq!(none(k), want, "{k:?}: os campos sem alça");
    }
    // As formas sem campo não têm alça nenhuma.
    for k in [K::Rectangle, K::Line, K::Pill, K::Delay, K::Junction] {
        assert!(handles_of(k).is_empty(), "{k:?}");
    }
}

// ── O ORÁCULO: o Inkscape 1.4.4 corrido sobre formas nossas (doc 13 §2) ─────────────────────────

pub(super) fn fixture(name: &str) -> serde_json::Value {
    let p = format!(
        "{}/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_alcas/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .expect("json")
}

/// O ponteiro REAL do oráculo (o ecrã inteiro do `xdotool` pela calibração medida), em
/// coordenadas do documento dele, desfeita a rotação da forma.
pub(super) fn pointer(
    f: &serde_json::Value,
    e: &serde_json::Value,
    key: &str,
    rot_deg: f64,
) -> [f64; 2] {
    let c = &f["calibracao_tela"];
    let g = |k: &str| c[k].as_f64().unwrap();
    let t = &e[key];
    let x = (t[0].as_f64().unwrap() - g("ox")) / g("sx");
    let y = (t[1].as_f64().unwrap() - g("oy")) / g("sy");
    let a = -rot_deg.to_radians();
    let (dx, dy) = (x - 400.0, y - 300.0);
    [
        400.0 + dx * a.cos() - dy * a.sin(),
        300.0 + dx * a.sin() + dy * a.cos(),
    ]
}

pub(super) fn attr(e: &serde_json::Value, k: &str) -> f64 {
    e["depois"][k].as_str().unwrap().parse().unwrap()
}

/// O arrasto de uma alça nossa pelo CAMINHO do oráculo (a recta de 12 passos do `xdotool`), com
/// o documento dele levado ao nosso espaço por `map`.
pub(super) fn replay(
    kind: ShapeKind,
    handle: usize,
    v: ShapeValues,
    half: [f64; 2],
    a: [f64; 2],
    b: [f64; 2],
    mods: Mods,
) -> ShapeValues {
    let ck = cooker(kind, half);
    // O nó do Inkscape GUARDA a pega (medido: o `rx` é o do press + o deslocamento da mão, a
    // 0,000 nos 20 arrastos; a mão absoluta erra 0,04–0,14) — a nossa lei, sem tirar nem pôr.
    let grip = handle_point(&handles_of(kind)[handle], &v, half, &ck).unwrap();
    let mut d = HandleDrag::start(handle, v, a, grip, half);
    let mut out = v;
    for s in 1..=12 {
        let t = f64::from(s) / 12.0;
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        out = drag(kind, &mut d, p, mods, half, 1.0, &ck);
    }
    out
}

/// ⭐ O RAIO DO CANTO é a projecção da mão na aresta (o `rx` do Inkscape, 2 rectângulos, a
/// direito e rodado 25°): o nosso TR no mesmo lado de cima, com a nossa saturação (o canto é um
/// CÍRCULO: satura em meia aresta CURTA, 60, onde o elíptico do Inkscape vai a 100).
#[test]
fn the_corner_radius_is_the_hand_projected_on_the_edge_like_inkscape() {
    let mut n = 0;
    for (name, rot) in [("rect", 0.0), ("rect_rot25", 25.0)] {
        let f = fixture(name);
        for e in f["experiencias"].as_array().unwrap() {
            let nome = e["nome"].as_str().unwrap();
            if !nome.starts_with("rx_") || e["mod"] == "ctrl" {
                continue;
            }
            // Documento dele (y para baixo, centro 400,300) → o nosso local (y para cima).
            let m = |p: [f64; 2]| [p[0] - 400.0, 300.0 - p[1]];
            let a = m(pointer(&f, e, "alca_tela", rot));
            let b = m(pointer(&f, e, "alvo_tela", rot));
            let mut v = ShapeKind::RoundRect.defaults();
            v[0] = 10.0;
            let r = replay(
                ShapeKind::RoundRect,
                1,
                v,
                [100.0, 60.0],
                a,
                b,
                Mods::default(),
            );
            let want = attr(e, "rx").min(60.0);
            assert!(
                (r[0] - want).abs() < 2e-3,
                "{name}/{nome}: {} contra o oráculo {want}",
                r[0]
            );
            n += 1;
        }
    }
    assert_eq!(
        n, 18,
        "as experiências do rx (9 por rectângulo: 6 na aresta, 2 fora dela, 1 Shift)"
    );
}

/// ⭐ O RAIO INTERIOR da estrela é a DISTÂNCIA da mão ao centro (o `r2` do Inkscape, com e sem
/// Ctrl, direita e rodada): a nossa razão = `r2 / r1`. O Inkscape vira as pontas para a direita
/// do topo e nós para a esquerda: o documento dele entra ESPELHADO.
#[test]
fn the_star_inner_radius_is_the_distance_to_the_centre_like_inkscape() {
    let mut n = 0;
    for (name, rot) in [("star5", 0.0), ("star5_rot25", 25.0)] {
        let f = fixture(name);
        for e in f["experiencias"].as_array().unwrap() {
            let nome = e["nome"].as_str().unwrap();
            if !nome.starts_with("inner_") || e["mod"] == "shift" {
                continue;
            }
            let m = |p: [f64; 2]| [400.0 - p[0], 300.0 - p[1]];
            let a = m(pointer(&f, e, "alca_tela", rot));
            let b = m(pointer(&f, e, "alvo_tela", rot));
            let mut v = ShapeKind::Star.defaults();
            v[1] = 0.45;
            let inner = handles_of(ShapeKind::Star)
                .iter()
                .position(|h| matches!(h.law, Law::Polar { .. }));
            let r = replay(
                ShapeKind::Star,
                inner.unwrap(),
                v,
                [100.0, 100.0],
                a,
                b,
                Mods::default(),
            );
            // A faixa do NOSSO descritor (0,05..0,95) manda; o Inkscape deixa o vale passar a ponta.
            let want = (attr(e, "sodipodi:r2") / attr(e, "sodipodi:r1")).min(0.95);
            assert!(
                (r[1] - want).abs() < 5e-3,
                "{name}/{nome}: {} contra {want}",
                r[1]
            );
            n += 1;
        }
    }
    assert_eq!(n, 12, "as experiências do vale (6 por estrela)");
}

/// ⭐ OS ÂNGULOS DO ARCO: o ângulo NORMALIZADO da mão, o outro fim parado; Ctrl do Inkscape (15°)
/// = o nosso Shift, e o Shift dele (rodar os dois) = o nosso Ctrl. O documento dele entra com o y
/// virado, o que troca o início e o fim: o arco dele `[0°, 270°]` é o nosso de 90° com 270° de
/// abertura, e o INÍCIO dele é o nosso FIM.
#[test]
fn the_arc_angles_follow_the_normalised_angle_like_inkscape() {
    let k = ShapeKind::Pie;
    let hs = handles_of(k);
    let start_h = hs
        .iter()
        .position(|h| matches!(h.law, Law::ArcStart { .. }))
        .unwrap();
    let end_h = hs
        .iter()
        .position(|h| matches!(h.law, Law::Sweep { .. }))
        .unwrap();
    let mut n = 0;
    for (name, rot) in [("arc_slice", 0.0), ("arc_slice_rot25", 25.0)] {
        let f = fixture(name);
        for e in f["experiencias"].as_array().unwrap() {
            let nome = e["nome"].as_str().unwrap();
            let m = |p: [f64; 2]| [p[0] - 400.0, 300.0 - p[1]];
            let a = m(pointer(&f, e, "alca_tela", rot));
            let b = m(pointer(&f, e, "alvo_tela", rot));
            let mut v = k.defaults();
            v[1] = 90.0;
            v[0] = 270.0;
            let ink_start = attr(e, "sodipodi:start").to_degrees().rem_euclid(360.0);
            let ink_end = attr(e, "sodipodi:end").to_degrees().rem_euclid(360.0);
            // O arco dele [s, e] (y para baixo) é o nosso [−e, −s]: início 360 − e, fim 360 − s.
            let want_start = (360.0 - ink_end).rem_euclid(360.0);
            let want_end = 360.0 - ink_start;
            let (handle, mods) = match (nome.split('_').next().unwrap(), e["mod"].as_str().unwrap())
            {
                ("start", "") => (end_h, Mods::default()),
                ("start", "ctrl") => (
                    end_h,
                    Mods {
                        snap: true,
                        ..Mods::default()
                    },
                ),
                ("start", "shift") => (
                    end_h,
                    Mods {
                        pairs: true,
                        ..Mods::default()
                    },
                ),
                ("end", "") => (start_h, Mods::default()),
                _ => continue,
            };
            let r = replay(k, handle, v, [120.0, 80.0], a, b, mods);
            let got_start = r[1].rem_euclid(360.0);
            let got_end = r[1] + r[0];
            // ⛔ A ÚNICA divergência, e é decisão: quando a mão passa por cima do outro fim, o
            // Inkscape salta (a abertura dá a volta: 270° → 70°) e nós SATURAMOS na volta inteira
            // — a forma não desaparece debaixo da mão. Mede-se o caminho desenrolado e confere-se
            // que o oráculo saltou mesmo.
            let open = (want_end - want_start).rem_euclid(360.0);
            let swept = (got_end - got_start).rem_euclid(360.0);
            if (r[0] - 360.0).abs() < 1e-9 && !mods.pairs && (open - 360.0).abs() > 1.0 {
                assert!(
                    open < 180.0 && swept < 1e-6,
                    "{name}/{nome}: saturou sem o oráculo saltar ({open:.2})"
                );
                n += 1;
                continue;
            }
            assert!(
                // A translação da pega reproduz o oráculo a ~0,001° (a rotação erraria até 0,6°).
                wrap180(got_start - want_start).abs() < 0.02
                    && wrap180(got_end - want_end).abs() < 0.02,
                "{name}/{nome}: início {got_start:.2} fim {got_end:.2} contra {want_start:.2} {want_end:.2}"
            );
            n += 1;
        }
    }
    assert_eq!(n, 24, "as 12 experiências de cada arco");
}

/// ⭐ AS VOLTAS da espiral: o Inkscape soma `Δθ/360` contínuo; as nossas voltas são INTEIRAS — a
/// mesma lei arredondada (e a distância da mão não conta, nas duas).
#[test]
fn the_spiral_turns_follow_the_winding_like_inkscape() {
    let f = fixture("spiral");
    let mut n = 0;
    for e in f["experiencias"].as_array().unwrap() {
        let nome = e["nome"].as_str().unwrap();
        // O Shift dele escala e roda (não é uma contagem); e o arrasto de 180° do roteiro é uma
        // recta PELO centro — o sentido de uma volta assim não existe.
        if !nome.starts_with("outer_") || e["mod"] == "shift" || nome.starts_with("outer_a+180") {
            continue;
        }
        let rev = attr(e, "sodipodi:revolution");
        let a0 = pointer(&f, e, "alca_tela", 0.0);
        let b0 = pointer(&f, e, "alvo_tela", 0.0);
        let ang = |p: [f64; 2]| (300.0 - p[1]).atan2(p[0] - 400.0).to_degrees();
        // O y virado inverte o sentido dos dois (o dele enrola para baixo, o nosso para cima).
        let turn = -wrap180(ang(b0) - ang(a0));
        let mut v = ShapeKind::Spiral.defaults();
        v[0] = 3.0;
        let got = spin(ShapeKind::Spiral, 0, v, turn, Mods::default())[0];
        let want = 3.0 + (rev - 3.0).round();
        assert_eq!(
            got, want,
            "{nome}: {got} contra o oráculo {rev} arredondado"
        );
        n += 1;
    }
    assert_eq!(n, 6, "as experiências da ponta de fora");
}

/// ⭐ As alças de ÂNGULO recolhem-se `park_at` para dentro do aro (nunca em cima das alças do meio
/// das arestas da caixa), até metade do raio — e o arrasto delas continua a ler só o ângulo.
#[test]
fn an_angle_handle_lives_inside_the_rim_off_the_box_handles() {
    let k = ShapeKind::Pie;
    let ck = cooker(k, HALF);
    let v = k.defaults(); // início a 0° (3h): no aro, é o meio da aresta direita da caixa
    let st = handles_of(k)
        .iter()
        .position(|h| matches!(h.law, Law::ArcStart { .. }))
        .unwrap();
    let h = &handles_of(k)[st];
    let truth = handle_point(h, &v, HALF, &ck).unwrap();
    assert!((truth[0] - HALF[0]).abs() < 1e-9 && truth[1].abs() < 1e-9);
    let g = handle_grip(k, h, &v, HALF, PX, 0.25, &ck).unwrap();
    assert!(
        (g[0] - (HALF[0] - 0.25)).abs() < 1e-9 && g[1].abs() < 1e-9,
        "{g:?}"
    );
    // Até metade do raio: num raio de 0,3 a alça fica a 0,15.
    let tiny = handle_grip(k, h, &v, [0.3, 0.3], PX, 0.25, &cooker(k, [0.3, 0.3])).unwrap();
    assert!((tiny[0] - 0.15).abs() < 1e-9, "{tiny:?}");
    // Agarrada no sítio recolhido, a 40° de volta: o início vai a 40°, como do aro.
    // Agarrada no sítio DESENHADO (a pega mede-se a ele) e levada por um quarto de volta perto do
    // centro (meio raio): o ângulo da mão é o que conta.
    let mut d = HandleDrag::start(st, v, g, g, HALF);
    let mut r = v;
    for s in 1..=8 {
        let a = (40.0 * f64::from(s) / 8.0).to_radians();
        r = drag(
            k,
            &mut d,
            [HALF[0] * 0.5 * a.cos(), HALF[1] * 0.5 * a.sin()],
            Mods::default(),
            HALF,
            PX,
            &ck,
        );
    }
    assert!((r[1] - 40.0).abs() < 1e-9, "{r:?}");
}
