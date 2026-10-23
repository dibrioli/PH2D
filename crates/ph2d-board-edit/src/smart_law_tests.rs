//! W24 — as leis do desenho inteligente que o doc 17 §6 deixara sem gate (módulo filho de `smart_tests`):
//! toda forma pousa num elemento que a desenha, uma ponta presa, o `Ctrl+Z` por família e o censo dos
//! limiares.

use super::*;

/// A distância de `p` à linha `poly` (fechada se `closed`).
fn to_line(p: [f64; 2], poly: &[[f64; 2]], closed: bool) -> f64 {
    let n = if closed { poly.len() } else { poly.len() - 1 };
    (0..n)
        .map(|i| crate::point_segment(p, poly[i], poly[(i + 1) % poly.len()]))
        .fold(f64::INFINITY, f64::min)
}

/// Hausdorff entre duas linhas, cada uma amostrada a cada 0,25 ao longo dos lados.
fn hausdorff(a: &[[f64; 2]], b: &[[f64; 2]], closed: bool) -> f64 {
    let dense = |l: &[[f64; 2]]| {
        let n = if closed { l.len() } else { l.len() - 1 };
        let mut v = Vec::new();
        for i in 0..n {
            let (p, q) = (l[i], l[(i + 1) % l.len()]);
            let k = ((q[0] - p[0]).hypot(q[1] - p[1]) / 0.25).ceil().max(1.0) as usize;
            v.extend((0..=k).map(|j| {
                let t = j as f64 / k as f64;
                [p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]
            }));
        }
        v
    };
    let one = |x: &[[f64; 2]], y: &[[f64; 2]]| {
        dense(x)
            .into_iter()
            .map(|p| to_line(p, y, closed))
            .fold(0.0, f64::max)
    };
    one(a, b).max(one(b, a))
}

/// O que um caso TEM de dar (onde o desenho decide; `Any` = forma do quadro ou tinta, o que a desenhar).
#[derive(Debug, PartialEq)]
enum Lands {
    Shape(ShapeType),
    Wire([Head; 2]),
    Ink,
    Any,
}

/// ⭐ Toda forma que a lei do vetor reconhece pousa num elemento do quadro que a DESENHA (a menos de um
/// px de ecrã) — objecto, ligação com as pontas certas, ou tinta limpa sobre ela — e nunca em nada. A
/// régua é independente da do produto (Hausdorff pelas amostras contra o caminho do vetor).
#[test]
fn every_recognised_shape_lands_on_an_element_that_draws_it() {
    use ph2d_vec_edit::quickshape::{ArcShape, ArrowShape, QuickShape};
    let poly = |c: &[[f64; 2]], closed| QuickShape::Polygon {
        corners: c.to_vec(),
        closed,
    };
    let regular = |n: usize, r: f64| -> Vec<[f64; 2]> {
        (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64 - std::f64::consts::FRAC_PI_2;
                [300.0 + r * a.cos(), 300.0 + r * a.sin()]
            })
            .collect()
    };
    let line = QuickShape::Line {
        a: [10.0, 20.0],
        b: [410.0, 220.0],
    };
    let arc = ArcShape {
        centre: [200.0, 200.0],
        rx: 150.0,
        ry: 90.0,
        rot: 0.4,
        start: 0.3,
        sweep: 2.2,
    };
    let arrow = |shaft: QuickShape, heads| {
        QuickShape::Arrow(ArrowShape {
            shaft: Box::new(shaft),
            heads,
        })
    };
    let rect = rotated(&RECT[..4], 30.0);
    let rhombus = [[0.0, -150.0], [100.0, 0.0], [0.0, 150.0], [-100.0, 0.0]];
    let cases: Vec<(&str, QuickShape, Lands)> = vec![
        (
            "rect rodado",
            poly(&rect, true),
            Lands::Shape(ShapeType::Rectangle),
        ),
        (
            "losango",
            poly(&rhombus, true),
            Lands::Shape(ShapeType::Diamond),
        ),
        (
            "triângulo regular",
            poly(&regular(3, 150.0), true),
            Lands::Any,
        ),
        (
            "triângulo qualquer",
            poly(&[[0.0, 0.0], [400.0, 0.0], [60.0, -220.0]], true),
            Lands::Ink,
        ),
        ("pentágono", poly(&regular(5, 150.0), true), Lands::Any),
        ("hexágono", poly(&regular(6, 150.0), true), Lands::Any),
        ("octógono", poly(&regular(8, 150.0), true), Lands::Any),
        (
            "paralelogramo",
            poly(
                &[[0.0, 0.0], [300.0, 0.0], [380.0, 150.0], [80.0, 150.0]],
                true,
            ),
            Lands::Any,
        ),
        (
            "polilinha aberta",
            poly(&[[0.0, 0.0], [200.0, 0.0], [200.0, 150.0]], false),
            Lands::Ink,
        ),
        (
            "elipse rodada",
            QuickShape::Ellipse {
                centre: [300.0, 200.0],
                rx: 180.0,
                ry: 70.0,
                rot: 0.5,
            },
            Lands::Shape(ShapeType::Ellipse),
        ),
        ("arco", QuickShape::Arc(arc), Lands::Ink),
        (
            "biarco",
            QuickShape::Biarc([
                arc,
                ArcShape {
                    centre: [200.0 + 2.0 * 150.0 * (0.3f64 + 2.2).cos(), 200.0],
                    sweep: -1.5,
                    ..arc
                },
            ]),
            Lands::Ink,
        ),
        ("recta", line.clone(), Lands::Wire([Head::None; 2])),
        (
            "seta no fim",
            arrow(line.clone(), [None, Some(30.0)]),
            Lands::Wire([Head::None, Head::Arrow]),
        ),
        (
            "seta no início",
            arrow(line.clone(), [Some(30.0), None]),
            Lands::Wire([Head::Arrow, Head::None]),
        ),
        (
            "seta nas duas",
            arrow(line.clone(), [Some(30.0); 2]),
            Lands::Wire([Head::Arrow; 2]),
        ),
        (
            "seta de cabo curvo",
            arrow(QuickShape::Arc(arc), [None, Some(30.0)]),
            Lands::Ink,
        ),
    ];
    let ink = ph2d_board_model::Rgba([20, 40, 60, 255]);
    let pen_style = Style::new(None, Some(ink), ink);
    for (name, shape, want) in cases {
        let mut w = world();
        let id = w.doc.mint_id();
        let z = ph2d_board_model::FracKey::between(None, None);
        let el =
            w.ed.smart_landing(&w.doc, &shape, Pen::Pen, &pen_style, PX, (id, z));
        assert_eq!(el.id, id, "{name}");
        let path = shape.to_path();
        let truth: Vec<[f64; 2]> = ph2d_board_geom::cover::vec_polyline(&path, 0.05).concat();
        let got = if let Some(s) = el.shape() {
            if let Lands::Shape(k) = want {
                assert_eq!(s.kind, k, "{name}");
            }
            assert!(
                matches!(want, Lands::Shape(_) | Lands::Any),
                "{name}: {:?} não devia ser forma",
                s.kind
            );
            assert!(path.closed, "{name}: só um contorno fechado vira forma");
            let o = ph2d_board_geom::world_outline(&el).expect("contorno");
            let polys = ph2d_board_geom::cover::polygons(&o.fill, 0.05);
            assert_eq!(polys.len(), 1, "{name}");
            polys[0].clone()
        } else if let Some(c) = el.connector() {
            let Lands::Wire(heads) = want else {
                panic!("{name}: uma ligação, devia ser {want:?}");
            };
            assert_eq!(c.heads, heads, "{name}: as pontas");
            let ends = [c.start, c.end].map(|e| match e {
                End::Free(p) => p,
                End::Bound { .. } => panic!("{name}: preso ao quadro vazio"),
            });
            let QuickShape::Line { a, b } = (match &shape {
                QuickShape::Arrow(a) => a.shaft.as_ref().clone(),
                s => s.clone(),
            }) else {
                panic!("{name}: ligação sem recta");
            };
            assert_eq!(ends, [a, b], "{name}: as pontas da recta");
            continue;
        } else if let Some(i) = el.ink() {
            assert!(
                matches!(want, Lands::Ink | Lands::Any),
                "{name}: tinta, devia ser {want:?}"
            );
            assert!(i.clean, "{name}: a tinta LIMPA");
            if matches!(shape, QuickShape::Arrow(_)) {
                // As farpas desenham-se por cima do cabo: o cabo inteiro está na tinta.
                let pts: Vec<[f64; 2]> = crate::world_points(&el)
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect();
                for p in &truth {
                    assert!(to_line(*p, &pts, false) <= PX, "{name}: o cabo na tinta");
                }
                continue;
            }
            crate::world_points(&el)
                .iter()
                .map(|p| [p[0], p[1]])
                .collect()
        } else {
            panic!("{name}: {:?}", el.kind);
        };
        let d = hausdorff(&got, &truth, path.closed);
        assert!(d <= PX, "{name}: desenha a {d:.3} px da forma reconhecida");
    }
}

/// ⭐ Uma ponta a cair numa forma prende-se a ela e a outra fica solta — cada ponta pela porta da seta.
#[test]
fn a_line_with_one_end_on_a_box_binds_only_that_end() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 100.0, 100.0]);
    w.cmd(Command::Escape);
    let out: Vec<[f64; 2]> = (0..=60)
        .map(|i| [50.0 + 10.0 * f64::from(i), 50.0])
        .collect();
    w.ink(&out, Some(HOLD_NS), NONE);
    let el = w.top();
    let c = el.connector().expect("uma ligação");
    assert!(
        matches!(c.start, End::Bound { target, .. } if target == a),
        "{:?}",
        c.start
    );
    assert!(matches!(c.end, End::Free(_)), "{:?}", c.end);
    let back: Vec<[f64; 2]> = out.iter().rev().map(|p| [p[0], p[1] + 300.0]).collect();
    let mut back = back;
    back.extend((1..=25).map(|i| [50.0, 350.0 - 10.0 * f64::from(i)]));
    w.ink(&back[back.len() - 31..], Some(HOLD_NS), NONE);
    let el = w.top();
    let c = el.connector().expect("uma ligação");
    assert!(matches!(c.start, End::Free(_)), "{:?}", c.start);
    assert!(
        matches!(c.end, End::Bound { target, .. } if target == a),
        "{:?}",
        c.end
    );
}

/// ⭐ Em TODAS as famílias o 1.º `Ctrl+Z` devolve o traço tal como foi desenhado (cada ponto) e o 2.º
/// apaga-o: forma do quadro, ligação e tinta limpa.
#[test]
fn ctrl_z_gives_every_family_its_stroke_back() {
    let circle: Vec<[f64; 2]> = (0..=120)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 120.0;
            [500.0 + 200.0 * a.cos(), 500.0 + 120.0 * a.sin()]
        })
        .collect();
    let rhombus = sides(
        &[
            [0.0, -150.0],
            [100.0, 0.0],
            [0.0, 150.0],
            [-100.0, 0.0],
            [0.0, -150.0],
        ],
        40,
    );
    let scalene = sides(&[[0.0, 0.0], [400.0, 0.0], [60.0, -220.0], [0.0, 0.0]], 50);
    let line: Vec<[f64; 2]> = (0..=60).map(|i| [10.0 * f64::from(i), 0.0]).collect();
    let mut arrow = line.clone();
    arrow.extend(sides(&[[600.0, 0.0], [540.0, -40.0]], 10));
    arrow.extend(sides(&[[540.0, -40.0], [600.0, 0.0], [540.0, 40.0]], 10));
    for (name, pts, family) in [
        ("elipse", circle, "ellipse"),
        ("losango", rhombus, "diamond"),
        ("triângulo qualquer", scalene, "ink"),
        ("recta", line, "line"),
        ("seta", arrow, "arrow"),
    ] {
        let mut w = world();
        w.ink(&pts, Some(HOLD_NS), NONE);
        let el = w.top();
        let got = match (el.shape(), el.connector(), el.ink()) {
            (Some(s), ..) => format!("{:?}", s.kind).to_lowercase(),
            (_, Some(c), _) if c.heads == [Head::None; 2] => "line".into(),
            (_, Some(_), _) => "arrow".into(),
            (.., Some(i)) if i.clean => "ink".into(),
            _ => format!("{:?}", el.kind),
        };
        assert_eq!(got, family, "{name}");
        assert!(w.cmd(Command::Undo), "{name}");
        let raw = w.top();
        assert!(raw.ink().is_some_and(|i| !i.clean), "{name}: o traço cru");
        let drawn: Vec<[f64; 2]> = crate::world_points(&raw)
            .iter()
            .map(|p| [p[0], p[1]])
            .collect();
        let mut pts = pts;
        pts.dedup();
        assert_eq!(drawn.len(), pts.len(), "{name}: cada ponto");
        for (p, q) in drawn.iter().zip(&pts) {
            assert!(
                (p[0] - q[0]).hypot(p[1] - q[1]) < 1e-3,
                "{name}: {p:?} ≠ {q:?}"
            );
        }
        assert!(w.cmd(Command::Undo), "{name}");
        assert_eq!(w.doc.live_len(), 0, "{name}");
    }
}

/// ⭐ Censo: o quadro NÃO tem limiares de reconhecimento seus (doc 17 §2.2). Só o `smart.rs` fala com a
/// lei e o gatilho do vetor; ele usa as folgas do vetor pelo nome; e os seus números são só os do
/// desenho (o passo da tinta, a farpa), cada um nomeado aqui.
#[test]
fn the_board_has_no_recognition_thresholds_of_its_own() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let mut files = Vec::new();
    for dir in std::fs::read_dir(root).unwrap() {
        let dir = dir.unwrap().path();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with("ph2d-board-") || name == "ph2d-app-board" {
            files.extend(rs_files(&dir.join("src")));
        }
    }
    let hero = format!("{root}/ph2d-editor-core/src/screens/hero");
    files.extend(
        rs_files(std::path::Path::new(&hero))
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("board")
            }),
    );
    files.retain(|p| {
        let n = p.file_name().unwrap().to_string_lossy();
        !n.ends_with("_tests.rs") && !n.starts_with("smart_test")
    });
    assert!(files.len() > 100, "o censo varre {} ficheiros", files.len());
    // O que cada ficheiro pede ao vetor (o caminho até ao `;`).
    let mut asks = std::collections::BTreeMap::new();
    for f in &files {
        let code = strip_comments(&std::fs::read_to_string(f).unwrap());
        let uses: Vec<String> = code
            .match_indices("ph2d_vec_edit::")
            .map(|(i, _)| {
                code[i..]
                    .split(';')
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .collect()
            })
            .collect();
        if !uses.is_empty() {
            let rel = f.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            asks.insert(rel, uses);
        }
    }
    let want: std::collections::BTreeMap<String, Vec<String>> = [
        (
            "ph2d-board-edit/src/ink.rs",
            &["ph2d_vec_edit::quickshape_hold::HoldTick"][..],
        ),
        (
            "ph2d-board-edit/src/smart.rs",
            &[
                "ph2d_vec_edit::pencil::MIN_SAMPLE_PX",
                "ph2d_vec_edit::quickshape::QuickShape",
                "ph2d_vec_edit::quickshape_hold::{HOLD_SLOP_PX,Hold,HoldTick,rest_index}",
            ][..],
        ),
    ]
    .into_iter()
    .map(|(f, u)| (f.to_owned(), u.iter().map(|s| (*s).to_owned()).collect()))
    .collect();
    assert_eq!(asks, want, "quem pede o quê à lei do vetor");
    let code = strip_comments(include_str!("smart.rs"));
    for name in [
        "HOLD_SLOP_PX * self.px",
        "MIN_SAMPLE_PX * self.px",
        ".tick(&self.samples",
    ] {
        assert!(
            code.contains(name),
            "o gatilho usa a folga do vetor: {name}"
        );
    }
    let consts: Vec<&str> = code
        .match_indices("const ")
        .map(|(i, _)| code[i + 6..].split(':').next().unwrap().trim())
        .collect();
    assert_eq!(
        consts,
        ["POLYGON_KINDS", "BARB_DEG"],
        "as constantes do smart.rs"
    );
    // Os números do smart.rs: índices e tamanhos (0–4, os 7 tipos), metades, o quarto de px do achatar,
    // a pressão 0,5 da tinta limpa, o ε, e a farpa de 30°.
    let allowed = [
        "0", "1", "2", "3", "4", "7", "1.0", "2.0", "4.0", "0.25", "0.5", "1e-9", "30.0",
    ];
    let mut odd = Vec::new();
    let b = code.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let starts = b[i].is_ascii_digit()
            && (i == 0
                || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_' || b[i - 1] == b'.'));
        if !starts {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < b.len()
            && (b[j].is_ascii_alphanumeric()
                || b[j] == b'_'
                || (b[j] == b'.' && b.get(j + 1).is_some_and(u8::is_ascii_digit))
                || (b[j] == b'-' && b[j - 1] == b'e'))
        {
            j += 1;
        }
        let lit = code[i..j].trim_end_matches('.');
        if !allowed.contains(&lit) {
            odd.push(lit.to_owned());
        }
        i = j;
    }
    assert!(odd.is_empty(), "números novos no smart.rs: {odd:?}");
}

fn rs_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in rd {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(rs_files(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| l.find("//").map_or(l, |i| &l[..i]))
        .collect::<Vec<_>>()
        .join("\n")
}
