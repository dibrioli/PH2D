//! Gates da lei do Edit Shape — os dois oráculos (as fixtures, lidas) e as invariantes por tipo.

use super::*;
use crate::quickshape::ArrowShape;
use serde_json::Value;

const ORACLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape"
);

fn fixture(dir: &str, name: &str) -> Value {
    let p = format!("{ORACLE}/{dir}/{name}.json");
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: P2, b: P2, tol: f64) -> bool {
    dist(a, b) <= tol
}

/// A caixa (x, y, w, h) dos pontos — a medida que os dois oráculos gravam.
fn bbox(q: &[P2]) -> [f64; 4] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in q {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]]
}

fn rect(x: f64, y: f64, w: f64, h: f64, rot: f64) -> QuickShape {
    let c = [x + w / 2.0, y + h / 2.0];
    let at = |dx: f64, dy: f64| add(c, rotate([dx - w / 2.0, dy - h / 2.0], rot));
    QuickShape::Polygon {
        corners: vec![at(0.0, 0.0), at(w, 0.0), at(w, h), at(0.0, h)],
        closed: true,
    }
}

const CORNERS: [&str; 4] = ["nw", "ne", "se", "sw"];

#[test]
fn the_rectangle_corners_do_what_both_oracles_do() {
    // O Excalidraw (a forma do «draw shape», editada no editor) e o tldraw: cada canto arrastado
    // (+40, +25), livre e com `Shift`, dá a MESMA caixa e a mesma rotação.
    let mut n = 0;
    for (dir, name, key) in [
        (
            "fixtures_excalidraw_edita",
            "rect_alinhado_400x250",
            "alcas",
        ),
        ("fixtures_excalidraw_edita", "quadrado_300", "alcas"),
        ("fixtures_tldraw_edita", "rectangulo", "arrastos"),
        ("fixtures_tldraw_edita", "rectangulo_rodado_25", "arrastos"),
    ] {
        let fx = fixture(dir, name);
        let (shape, rot) = if key == "alcas" {
            let b = &fx["nascida"];
            (
                rect(f(&b["x"]), f(&b["y"]), f(&b["w"]), f(&b["h"]), 0.0),
                0.0,
            )
        } else {
            let s = &fx["base"]["shape"];
            let r = f(&s["rotation"]);
            let (w, h) = (f(&s["props"]["w"]), f(&s["props"]["h"]));
            // O tldraw guarda o canto de cima-esquerda e roda à volta dele.
            let o = [f(&s["x"]), f(&s["y"])];
            let at = |dx: f64, dy: f64| add(o, rotate([dx, dy], r));
            let c = vec![at(0.0, 0.0), at(w, 0.0), at(w, h), at(0.0, h)];
            (
                QuickShape::Polygon {
                    corners: c,
                    closed: true,
                },
                r,
            )
        };
        let QuickShape::Polygon { corners, .. } = &shape else {
            unreachable!()
        };
        for (i, c) in CORNERS.iter().enumerate() {
            for (mode, regular) in [("livre", false), ("shift", true)] {
                let out = &fx[key][format!("{c}/{mode}")];
                let got = drag(&shape, i, add(corners[i], [40.0, 25.0]), regular);
                let QuickShape::Polygon { corners: g, closed } = &got else {
                    panic!("{name} {c}/{mode}: deixou de ser polígono");
                };
                assert!(*closed && is_rectangle(g), "{name} {c}/{mode}: rectângulo");
                // A medida do oráculo: a caixa NO REFERENCIAL da forma (rodada de volta).
                let local: Vec<P2> = g.iter().map(|&p| rotate(p, -rot)).collect();
                let [_, _, w, h] = bbox(&local);
                let (ow, oh) = if key == "alcas" {
                    (f(&out["forma"]["w"]), f(&out["forma"]["h"]))
                } else {
                    (
                        f(&out["shape"]["props"]["w"]),
                        f(&out["shape"]["props"]["h"]),
                    )
                };
                assert!(
                    (w - ow).abs() < 0.01 && (h - oh).abs() < 0.01,
                    "{name} {c}/{mode}: {w}×{h} ≠ oráculo {ow}×{oh}"
                );
                assert!(
                    near(g[(i + 2) % 4], corners[(i + 2) % 4], 1e-9),
                    "{name} {c}/{mode}: o canto oposto fica"
                );
                n += 1;
            }
        }
    }
    assert_eq!(n, 32);
}

#[test]
fn the_ellipse_axis_nodes_do_what_the_tldraw_side_handles_do() {
    // Os nós da elipse são os extremos dos eixos — as alças de LADO do tldraw (e, s, w, n com o y
    // para baixo ⇒ os nossos +u, +v, −u, −v).
    let mut n = 0;
    for name in ["circulo", "elipse"] {
        let fx = fixture("fixtures_tldraw_edita", name);
        let s = &fx["base"]["shape"];
        let (w, h) = (f(&s["props"]["w"]), f(&s["props"]["h"]));
        let shape = QuickShape::Ellipse {
            centre: [f(&s["x"]) + w / 2.0, f(&s["y"]) + h / 2.0],
            rx: w / 2.0,
            ry: h / 2.0,
            rot: 0.0,
        };
        let at = nodes(&shape);
        for (k, side) in ["e", "s", "w", "n"].iter().enumerate() {
            for (mode, regular) in [("livre", false), ("shift", true)] {
                let out = &fx["arrastos"][format!("{side}/{mode}")]["shape"];
                let QuickShape::Ellipse { centre, rx, ry, .. } =
                    drag(&shape, k, add(at[k], [40.0, 25.0]), regular)
                else {
                    panic!("{name} {side}: deixou de ser elipse");
                };
                let (ow, oh) = (f(&out["props"]["w"]), f(&out["props"]["h"]));
                let (ox, oy) = (f(&out["x"]), f(&out["y"]));
                let got = [centre[0] - rx, centre[1] - ry, 2.0 * rx, 2.0 * ry];
                assert!(
                    got.iter()
                        .zip([ox, oy, ow, oh])
                        .all(|(a, b)| (a - b).abs() < 0.01),
                    "{name} {side}/{mode}: {got:?} ≠ oráculo {:?}",
                    [ox, oy, ow, oh]
                );
                n += 1;
            }
        }
    }
    assert_eq!(n, 16);
}

#[test]
fn the_line_ends_and_the_fifteen_degree_step_are_the_oracles() {
    // O fim da linha do Excalidraw, livre e com `Shift` (o lado a um múltiplo de 15°, o mesmo
    // comprimento): a ponta cai ao centésimo.
    let fx = fixture("fixtures_excalidraw_edita", "linha_diagonal_500");
    let pts = fx["nascida"]["points"].as_array().unwrap();
    let p = |v: &Value| [f(&v[0]), f(&v[1])];
    let shape = QuickShape::Line {
        a: p(&pts[0]),
        b: p(&pts[1]),
    };
    for (node, mode, regular) in [
        (0, "livre", false),
        (1, "livre", false),
        (1, "shift", true),
        (0, "shift", true),
    ] {
        let out = fx["alcas"][format!("ponto_{node}/{mode}")]["forma"]["points"]
            .as_array()
            .unwrap();
        let start = nodes(&shape)[node];
        let QuickShape::Line { a, b } = drag(&shape, node, add(start, [40.0, 25.0]), regular)
        else {
            unreachable!()
        };
        assert!(
            near(a, p(&out[0]), 0.01) && near(b, p(&out[1]), 0.01),
            "ponto_{node}/{mode}: {a:?} {b:?} ≠ {out:?}"
        );
    }
}

#[test]
fn the_curved_arrow_keeps_its_bend_and_the_middle_only_moves_the_bend_as_in_tldraw() {
    // A seta curva do tldraw: mover uma ponta guarda a flecha (o `bend`); a alça do meio só a mexe,
    // e fica no ápice. A nossa: o arco de círculo pelas pontas e pelo ápice.
    let fx = fixture("fixtures_tldraw_edita", "seta_curva");
    let props = &fx["base"]["shape"]["props"];
    let pt = |v: &Value| [f(&v["x"]), f(&v["y"])];
    let (s, e, bend) = (pt(&props["start"]), pt(&props["end"]), f(&props["bend"]));
    // A flecha do tldraw cai do lado da corda rodada de +90° (com o y para baixo): o nosso `+n`.
    let shape = QuickShape::Arc(circle_through(s, e, bend, 1.0));
    for (handle, node) in [("start", 0), ("end", 2), ("middle", 1)] {
        let key = if handle == "middle" {
            "virtual:middle/livre".to_owned()
        } else {
            format!("vertex:{handle}/livre")
        };
        let out = &fx["arrastos"][key];
        let mid = out["alcas"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["id"] == "middle")
            .unwrap();
        let got = drag(&shape, node, add(nodes(&shape)[node], [40.0, 25.0]), false);
        let QuickShape::Arc(a) = &got else {
            panic!("{handle}: deixou de ser arco")
        };
        assert!(
            is_circle(a),
            "{handle}: o arco de círculo continua de círculo"
        );
        let at = nodes(&got);
        let o = &out["shape"]["props"];
        assert!(
            near(at[0], pt(&o["start"]), 0.01) && near(at[2], pt(&o["end"]), 0.01),
            "{handle}: as pontas"
        );
        assert!(
            near(at[1], [f(&mid["x"]), f(&mid["y"])], 0.01),
            "{handle}: o ápice {:?} ≠ a alça do meio do tldraw {mid}",
            at[1]
        );
    }
}

fn arc(centre: P2, rx: f64, ry: f64, rot: f64, start: f64, sweep: f64) -> ArcShape {
    ArcShape {
        centre,
        rx,
        ry,
        rot,
        start,
        sweep,
    }
}

#[test]
fn every_type_shows_its_own_nodes() {
    let line = QuickShape::Line {
        a: [0.0, 0.0],
        b: [10.0, 0.0],
    };
    let tri = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [10.0, 0.0], [5.0, 8.0]],
        closed: true,
    };
    let ell = QuickShape::Ellipse {
        centre: [0.0, 0.0],
        rx: 4.0,
        ry: 2.0,
        rot: 0.3,
    };
    let a = arc([0.0, 0.0], 5.0, 5.0, 0.0, 0.0, 2.0);
    let bi = QuickShape::Biarc([
        arc([0.0, 0.0], 5.0, 5.0, 0.0, -1.0, 1.0),
        arc([0.0, 0.0], 5.0, 5.0, 0.0, 0.0, 1.0),
    ]);
    let arrow = QuickShape::Arrow(ArrowShape {
        shaft: Box::new(line.clone()),
        heads: [None, Some(2.0)],
    });
    for (shape, n) in [
        (&line, 2),
        (&tri, 3),
        (&rect(0.0, 0.0, 4.0, 2.0, 0.0), 4),
        (&ell, 4),
        (&QuickShape::Arc(a), 3),
        (&bi, 3),
        (&arrow, 2),
    ] {
        assert_eq!(nodes(shape).len(), n, "{shape:?}");
    }
    // Os nós da elipse ESTÃO na elipse, nos extremos dos eixos.
    for p in nodes(&ell) {
        let q = rotate(sub(p, [0.0, 0.0]), -0.3);
        assert!(((q[0] / 4.0).powi(2) + (q[1] / 2.0).powi(2) - 1.0).abs() < 1e-12);
    }
    // O ápice do arco é o meio dele; o do biarco, a junção (os dois arcos simétricos).
    assert!(near(nodes(&QuickShape::Arc(a))[1], a.at(1.0), 1e-12));
    assert!(near(nodes(&bi)[1], [5.0, 0.0], 1e-9));
}

#[test]
fn a_free_corner_goes_under_the_finger_and_shift_steps_its_side_by_fifteen_degrees() {
    let tri = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [10.0, 0.0], [5.0, 8.0]],
        closed: true,
    };
    let QuickShape::Polygon { corners, .. } = drag(&tri, 2, [6.0, 9.0], false) else {
        unreachable!()
    };
    assert_eq!(corners, vec![[0.0, 0.0], [10.0, 0.0], [6.0, 9.0]]);
    // `Shift`: o lado que CHEGA ao canto (do anterior) a um múltiplo de 15°, o mesmo comprimento.
    let QuickShape::Polygon { corners, .. } = drag(&tri, 2, [6.0, 9.0], true) else {
        unreachable!()
    };
    let d = sub(corners[2], corners[1]);
    let deg = angle(d).to_degrees();
    assert!((deg / 15.0 - (deg / 15.0).round()).abs() < 1e-9, "{deg}");
    assert!((d[0].hypot(d[1]) - dist([6.0, 9.0], [10.0, 0.0])).abs() < 1e-9);
    // A polilinha ABERTA: o 1.º nó não tem anterior — o lado é o que vai ao seguinte.
    let open = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [10.0, 0.0], [20.0, 5.0]],
        closed: false,
    };
    let QuickShape::Polygon { corners, .. } = drag(&open, 0, [1.0, 3.0], true) else {
        unreachable!()
    };
    let deg = angle(sub(corners[0], corners[1])).to_degrees();
    assert!((deg / 15.0 - (deg / 15.0).round()).abs() < 1e-9, "{deg}");
}

#[test]
fn a_quadrilateral_that_is_not_a_rectangle_moves_its_corner_freely() {
    let quad = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [10.0, 0.0], [11.0, 6.0], [0.0, 5.0]],
        closed: true,
    };
    let QuickShape::Polygon { corners, .. } = drag(&quad, 2, [14.0, 9.0], false) else {
        unreachable!()
    };
    assert_eq!(corners[2], [14.0, 9.0]);
    assert_eq!(corners[1], [10.0, 0.0], "os vizinhos ficam");
}

#[test]
fn the_rectangle_corner_lands_on_the_finger_and_the_sides_keep_their_rotation() {
    let r = rect(0.0, 0.0, 30.0, 20.0, 0.4);
    let QuickShape::Polygon { corners: c0, .. } = &r else {
        unreachable!()
    };
    let to = [33.0, 12.0];
    let QuickShape::Polygon { corners, .. } = drag(&r, 2, to, false) else {
        unreachable!()
    };
    assert!(near(corners[2], to, 1e-9), "o canto debaixo do dedo");
    let dir = |c: &[P2]| angle(sub(c[1], c[0]));
    assert!((dir(&corners) - dir(c0)).abs() < 1e-9, "a rotação fica");
}

#[test]
fn a_circle_becomes_an_ellipse_unless_shift_and_the_opposite_node_stays() {
    let c = QuickShape::Ellipse {
        centre: [0.0, 0.0],
        rx: 10.0,
        ry: 10.0,
        rot: 0.0,
    };
    let QuickShape::Ellipse { centre, rx, ry, .. } = drag(&c, 0, [14.0, 7.0], false) else {
        unreachable!()
    };
    // O arrasto ATRAVÉS do eixo não conta; o nó oposto (−10, 0) fica.
    assert!((rx - 12.0).abs() < 1e-12 && (ry - 10.0).abs() < 1e-12);
    assert!(near(centre, [2.0, 0.0], 1e-12));
    let QuickShape::Ellipse { centre, rx, ry, .. } = drag(&c, 0, [14.0, 7.0], true) else {
        unreachable!()
    };
    assert!(
        (rx - 12.0).abs() < 1e-12 && (ry - 12.0).abs() < 1e-12,
        "Shift: círculo"
    );
    assert!(near(centre, [2.0, 0.0], 1e-12));
    // O arrasto pode passar do oposto: a elipse vira-se, nunca colapsa a zero.
    let QuickShape::Ellipse { rx, .. } = drag(&c, 0, [-10.0, 0.0], false) else {
        unreachable!()
    };
    assert!(rx > 0.0);
}

#[test]
fn an_elliptic_arc_follows_the_chord_affinity_exactly() {
    let a = arc([0.0, 0.0], 8.0, 3.0, 0.5, 0.2, 2.4);
    let shape = QuickShape::Arc(a);
    let [s, top, e] = nodes(&shape)[..] else {
        unreachable!()
    };
    let to = add(top, [1.5, 2.0]);
    let QuickShape::Arc(b) = drag(&shape, 1, to, false) else {
        unreachable!()
    };
    let [s2, top2, e2] = nodes(&QuickShape::Arc(b))[..] else {
        unreachable!()
    };
    assert!(near(s2, s, 1e-9) && near(e2, e, 1e-9), "as pontas ficam");
    // O ápice anda SÓ através da corda (o paralelo não conta — o tldraw).
    let chord = sub(e, s);
    let n = mul(
        rotate(chord, std::f64::consts::FRAC_PI_2),
        1.0 / chord[0].hypot(chord[1]),
    );
    assert!((dot(sub(top2, s), n) - dot(sub(to, s), n)).abs() < 1e-9);
    assert!((dot(sub(top2, top), chord)).abs() < 1e-9);
    // E a curva inteira é a imagem afim da de antes, parâmetro a parâmetro.
    let (l, h, h2) = (
        chord[0].hypot(chord[1]),
        dot(sub(top, s), n),
        dot(sub(to, s), n),
    );
    let u = mul(chord, 1.0 / l);
    for i in 0..=8 {
        let t = a.start + a.sweep * f64::from(i) / 8.0;
        let p = a.at(t);
        let want = add(
            s,
            add(
                mul(u, dot(sub(p, s), u)),
                mul(n, dot(sub(p, s), n) * h2 / h),
            ),
        );
        let tb = b.start + b.sweep * f64::from(i) / 8.0;
        assert!(near(b.at(tb), want, 1e-9), "t {i}");
    }
}

#[test]
fn the_affine_image_of_an_arc_is_exact_even_through_a_mirror() {
    let a = arc([1.0, 2.0], 5.0, 2.0, 0.7, -0.4, 1.9);
    for lin in [
        [[1.3, 0.4], [-0.2, 0.8]],
        [[0.9, 0.0], [0.0, -1.4]],
        [[-1.1, 0.3], [0.5, 0.6]],
    ] {
        let m = Affine {
            lin,
            t: [3.0, -1.0],
        };
        let b = m.arc(&a);
        assert!(b.rx >= b.ry && b.ry > 0.0);
        for i in 0..=6 {
            let t = f64::from(i) / 6.0;
            assert!(
                near(
                    b.at(b.start + b.sweep * t),
                    m.point(a.at(a.start + a.sweep * t)),
                    1e-9
                ),
                "{lin:?} t {t}"
            );
        }
    }
}

#[test]
fn the_biarc_stays_two_tangent_arcs() {
    // Um biarco de verdade: o 2.º arco sai da junção (ângulo −0,8 no 1.º) com a mesma tangente.
    let j = unit(-0.8);
    let bi = QuickShape::Biarc([
        arc([0.0, 0.0], 10.0, 10.0, 0.0, -2.0, 1.2),
        arc(mul(j, 5.0), 5.0, 5.0, 0.0, -0.8, 1.6),
    ]);
    // Arrasta a ponta do fim e o ápice; a junção continua UMA e tangente.
    for (node, to) in [(2, [12.0, 9.0]), (1, [3.0, -6.0]), (0, [-7.0, -9.0])] {
        let QuickShape::Biarc([p, q]) = drag(&bi, node, to, false) else {
            panic!("deixou de ser biarco");
        };
        let tan = |a: &ArcShape, t: f64| sub(a.at(t + 1e-7), a.at(t - 1e-7));
        let (j0, j1) = (p.at(p.start + p.sweep), q.at(q.start));
        assert!(near(j0, j1, 1e-9), "a junção");
        let (d0, d1) = (tan(&p, p.start + p.sweep), tan(&q, q.start));
        assert!(
            cross(d0, d1).abs() / (d0[0].hypot(d0[1]) * d1[0].hypot(d1[1])) < 1e-6,
            "tangente"
        );
        assert!(dot(d0, d1) > 0.0);
    }
}

#[test]
fn the_arrow_tip_is_the_shaft_end_and_the_heads_stay() {
    let arrow = QuickShape::Arrow(ArrowShape {
        shaft: Box::new(QuickShape::Line {
            a: [0.0, 0.0],
            b: [10.0, 0.0],
        }),
        heads: [None, Some(2.0)],
    });
    let got = drag(&arrow, 1, [12.0, 4.0], false);
    assert_eq!(got.heads(), [None, Some(2.0)]);
    let QuickShape::Arrow(a) = got else {
        unreachable!()
    };
    assert_eq!(
        *a.shaft,
        QuickShape::Line {
            a: [0.0, 0.0],
            b: [12.0, 4.0]
        }
    );
}

#[test]
fn dragging_the_body_scales_the_shape_evenly_about_its_pivot() {
    let r = rect(0.0, 0.0, 30.0, 20.0, 0.0);
    let got = scale(&r, [30.0, 10.0], [45.0, 10.0]);
    // O pivô de uma forma fechada é o centro (15, 10); 15 → 30 ⇒ o dobro.
    let QuickShape::Polygon { corners, .. } = got else {
        unreachable!()
    };
    assert!(near(corners[0], [0.0 - 15.0, 0.0 - 10.0], 1e-9));
    assert!(near(corners[2], [45.0, 30.0], 1e-9));
}
