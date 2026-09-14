//! ⛔⛔ **OS CASCOS POR FOLHA, GATEADOS ONDE SÃO DEFINIDOS** (W148).
//!
//! A cache de fitas do `ph2d-field-render` serve uma fita a toda região cujos cascos ela contém
//! ([`RegionHulls::contains`]). Um casco a mais do que devia torna-a mais lenta; **um a menos serve
//! uma fita onde ela não vale** — a distância sai grande demais e a marcha **atravessa a peça**. As
//! duas perguntas deste ficheiro são as duas metades disso, em POSES QUAISQUER (os gates do casco da
//! W59 só medem a identidade):
//!
//! 1. [`the_containment_of_hulls_is_sound_in_any_pose`] — se o `contains` diz sim, todo ponto do casco
//!    da consulta está no da fita, folha a folha;
//! 2. [`a_posed_leaf_region_holds_what_the_march_evaluates`] — a região de cada folha contém o que a
//!    marcha de facto avalia nela.
//!
//! Compostas: *o que a marcha avalia ⊆ casco da consulta ⊆ casco da fita* ⇒ a fita servida responde
//! certo.
//!
//! ⚠️ **Amostragem, e não o argumento dos vértices**: o `contains` usa exactamente esse argumento
//! (convexo em convexo ⇔ vértices), e um gate que o repetisse partilharia o defeito — *um controlo que
//! partilha o defeito não é um controlo* (W97).

use crate::RegionCompiler;
use crate::hull::in_convex;
use ph2d_field::{
    Blend, FieldDoc, FillRule, Node, NodeId, NodeKind, Op, Primitive, Profile, Unary, Xform,
};

/// Um misturador determinístico — a fixtura é a mesma em toda corrida.
struct Rng(u64);

impl Rng {
    fn unit(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^= x >> 31;
        (x >> 40) as f32 / 16_777_216.0
    }

    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }

    fn vec(&mut self, a: f32, b: f32) -> [f32; 3] {
        [self.range(a, b), self.range(a, b), self.range(a, b)]
    }
}

fn quat(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let s = (angle * 0.5).sin() / n;
    [axis[0] * s, axis[1] * s, axis[2] * s, (angle * 0.5).cos()]
}

fn ring(n: usize, r_in: f32, r_out: f32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let r = if i % 2 == 0 { r_out } else { r_in };
            [r * a.cos(), r * a.sin()]
        })
        .collect()
}

/// ⭐ **Uma peça com as poses que a identidade não tem** — rotação oblíqua, escala e translação em
/// cada folha E na raiz, mais as duas folhas que NÃO têm casco (o torno, e uma debaixo de um espelho,
/// que remapeia coordenadas).
fn posed_doc() -> FieldDoc {
    let prof =
        |pts: Vec<[f32; 2]>| Profile::new(vec![pts], FillRule::NonZero, 1e-3).expect("perfil");
    let x = |axis: [f32; 3], angle: f32, scale: f32, translation: [f32; 3]| Xform {
        translation,
        rotation: quat(axis, angle),
        scale,
    };
    let star = Node::new(
        x([1.0, 1.0, 0.0], 0.7, 0.8, [0.1, -0.05, 0.2]),
        NodeKind::Leaf(Primitive::Extrude {
            profile: prof(ring(24, 0.18, 0.4)),
            half_height: 0.2,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    let poly = Node::new(
        x([0.0, 0.3, 1.0], 1.9, 1.3, [-0.2, 0.1, -0.1]),
        NodeKind::Leaf(Primitive::Polygon {
            profile: prof(ring(12, 0.25, 0.3)),
            half_height: 0.15,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    let lathe = Node::new(
        x([0.0, 1.0, 0.0], 0.0, 1.0, [0.0, 0.3, 0.0]),
        NodeKind::Leaf(Primitive::Revolve {
            profile: prof(vec![[0.1, -0.2], [0.3, -0.2], [0.3, 0.2], [0.1, 0.2]]),
        }),
    );
    let mut mirrored = Node::new(
        x([1.0, 0.0, 0.0], 0.3, 1.0, [0.0, -0.3, 0.0]),
        NodeKind::Leaf(Primitive::Extrude {
            profile: prof(ring(16, 0.1, 0.2)),
            half_height: 0.1,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    mirrored.mods = vec![Unary::Mirror { offset: 0.0 }];
    let root = Node::new(
        x([0.0, 1.0, 0.0], 0.4, 0.9, [0.0; 3]),
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
        },
    );
    FieldDoc::new(vec![star, poly, lathe, mirrored, root], NodeId(4)).expect("a peça posta")
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let n = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-12);
    [a[0] / n, a[1] / n, a[2] / n]
}

/// Um tubo de fatia como a marcha os pede: oito cantos (a secção perto e a longe, que é mais larga),
/// a caixa deles e a folga da sonda da normal.
struct Tube {
    lo: [f32; 3],
    hi: [f32; 3],
    pts: Vec<[f32; 3]>,
    pad: f32,
}

fn tube(centre: [f32; 3], dir: [f32; 3], len: f32, near: f32, far: f32, pad: f32) -> Tube {
    let d = unit(dir);
    let helper = if d[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let e1 = unit(cross(d, helper));
    let e2 = cross(d, e1);
    let mut pts = Vec::with_capacity(8);
    for (s, w) in [(-0.5f32, near), (0.5, far)] {
        for a in [-1.0f32, 1.0] {
            for b in [-1.0f32, 1.0] {
                pts.push(std::array::from_fn(|k| {
                    centre[k] + d[k] * s * len + e1[k] * a * w + e2[k] * b * w
                }));
            }
        }
    }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k] - pad);
            hi[k] = hi[k].max(p[k] + pad);
        }
    }
    Tube { lo, hi, pts, pad }
}

/// O polígono que a distância de uma folha consome: o casco, ou a caixa local quando ele degenerou.
fn polygon(hull: &[[f32; 2]], local: ([f32; 3], [f32; 3])) -> Vec<[f32; 2]> {
    if hull.len() >= 3 {
        hull.to_vec()
    } else {
        let (lo, hi) = local;
        vec![
            [lo[0], lo[1]],
            [hi[0], lo[1]],
            [hi[0], hi[1]],
            [lo[0], hi[1]],
        ]
    }
}

/// Pontos DENTRO de um polígono convexo: os vértices, os meios das arestas e combinações convexas
/// aleatórias — puxados `1e-4` para o centro, para um ponto sobre a fronteira comum dos dois
/// polígonos não ser lido como fuga por um ULP.
fn samples_in(poly: &[[f32; 2]], rng: &mut Rng) -> Vec<[f32; 2]> {
    let n = poly.len();
    let c = poly.iter().fold([0.0f32; 2], |a, p| {
        [a[0] + p[0] / n as f32, a[1] + p[1] / n as f32]
    });
    let mut out: Vec<[f32; 2]> = poly.to_vec();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        out.push([0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]);
    }
    for _ in 0..24 {
        let w: Vec<f32> = (0..n).map(|_| -(rng.unit().max(1e-6)).ln()).collect();
        let s: f32 = w.iter().sum();
        out.push(poly.iter().zip(&w).fold([0.0f32; 2], |a, (p, wi)| {
            [a[0] + p[0] * wi / s, a[1] + p[1] * wi / s]
        }));
    }
    out.iter()
        .map(|p| {
            [
                c[0] + (p[0] - c[0]) * (1.0 - 1e-4),
                c[1] + (p[1] - c[1]) * (1.0 - 1e-4),
            ]
        })
        .collect()
}

/// As folhas que a compilação especializa contra o casco — ⚠️ lidas da MESMA regra que o produto usa.
fn hulled_leaves(doc: &FieldDoc, rc: &RegionCompiler) -> Vec<usize> {
    (0..doc.nodes().len())
        .filter(|&i| {
            let node = &doc.nodes()[i];
            matches!(
                node.kind,
                NodeKind::Leaf(Primitive::Extrude { .. } | Primitive::Polygon { .. })
            ) && rc.specialised_leaf(node, i).is_some()
        })
        .collect()
}

#[test]
fn the_containment_of_hulls_is_sound_in_any_pose() {
    let doc = posed_doc();
    let rc = RegionCompiler::new(&doc);
    let leaves = hulled_leaves(&doc, &rc);
    // ⛔ **O controlo positivo da fixtura**: as duas folhas com casco, e só elas — o torno corta pela
    // caixa, e o espelho remapeia coordenadas.
    assert_eq!(
        leaves,
        vec![0, 1],
        "a peça posta tem de ter casco no extrude e no polígono, e em mais nenhuma folha"
    );
    let mut rng = Rng(0x5EED_CA5C0);
    let (mut accepted, mut rejected, mut rejected_for_a_reason) = (0usize, 0usize, 0usize);
    for trial in 0..4000 {
        let centre = rng.vec(-0.45, 0.45);
        let dir = rng.vec(-1.0, 1.0);
        let len = rng.range(0.05, 0.4);
        let near = rng.range(0.005, 0.06);
        let far = near * rng.range(1.0, 1.4);
        let pad = rng.range(0.001, 0.008);
        let delta = rng.range(0.003, 0.08);
        let entry = tube(centre, dir, len, near, far, pad);
        let (elo, ehi) = (entry.lo.map(|v| v - delta), entry.hi.map(|v| v + delta));
        let eh = rc.hulls(&doc, elo, ehi, &entry.pts);
        assert_eq!(eh.probe_leaves(), 2, "a região da fita perdeu uma folha");
        // A consulta do quadro seguinte: o tubo andou até `1,5·δ` e rodou um pouco.
        let shift = rng.vec(-1.5 * delta, 1.5 * delta);
        let turn = rng.vec(-0.05, 0.05);
        let q = tube(
            std::array::from_fn(|k| centre[k] + shift[k]),
            std::array::from_fn(|k| unit(dir)[k] + turn[k]),
            len,
            near,
            far,
            pad,
        );
        // ⚠️ A caixa do MUNDO é pergunta de quem chama — sem ela, não há o que perguntar ao casco.
        if !(0..3).all(|k| q.lo[k] >= elo[k] && q.hi[k] <= ehi[k]) {
            continue;
        }
        let qh = rc.hulls(&doc, q.lo, q.hi, &q.pts);
        let mut escapes = 0usize;
        for &i in &leaves {
            let (m, _) = rc
                .specialised_leaf(&doc.nodes()[i], i)
                .expect("folha especializada");
            let outer = polygon(eh.hull_of(i), m.box_of(elo, ehi));
            let inner = polygon(qh.hull_of(i), m.box_of(q.lo, q.hi));
            escapes += samples_in(&inner, &mut rng)
                .iter()
                .filter(|s| !in_convex(**s, &outer))
                .count();
        }
        if eh.contains(&qh) {
            accepted += 1;
            assert_eq!(
                escapes, 0,
                "tentativa {trial}: o `contains` serviu a fita e {escapes} pontos do casco da \
                 consulta caem FORA do casco dela — a distância sairia grande demais e a marcha \
                 atravessaria a peça"
            );
        } else {
            rejected += 1;
            rejected_for_a_reason += usize::from(escapes > 0);
        }
    }
    // ⛔ **Os dois baldes têm de estar cheios**, senão o gate não prende nada: um `contains` que dissesse
    // sempre «não» passaria a asserção de cima com zero aceites.
    assert!(
        accepted >= 300,
        "só {accepted} consultas foram servidas — a fixtura não mede o lado do sim"
    );
    // ⭐⭐ **E o controlo que o torna um gate**: consultas que a CAIXA serviria e o casco recusa, com
    // pontos que de facto fogem. Um `contains` que só olhasse a caixa serviria estas — e a asserção
    // de cima reprovaria. Sem elas, apagar a metade do casco ficaria verde.
    assert!(
        rejected_for_a_reason >= 30,
        "de {rejected} recusas só {rejected_for_a_reason} tinham pontos a fugir — a fixtura não tem \
         consultas que a caixa serviria errado, e o gate não distingue um `contains` de casco de um \
         de caixa"
    );
}

#[test]
fn a_posed_leaf_region_holds_what_the_march_evaluates() {
    let doc = posed_doc();
    let rc = RegionCompiler::new(&doc);
    let leaves = hulled_leaves(&doc, &rc);
    assert_eq!(leaves, vec![0, 1], "a fixtura perdeu as folhas com casco");
    let mut rng = Rng(0xB0A7_F1E1D);
    let mut measured = 0usize;
    for trial in 0..1500 {
        let t = tube(
            rng.vec(-0.45, 0.45),
            rng.vec(-1.0, 1.0),
            rng.range(0.05, 0.4),
            rng.range(0.005, 0.06),
            rng.range(0.06, 0.09),
            rng.range(0.001, 0.008),
        );
        let h = rc.hulls(&doc, t.lo, t.hi, &t.pts);
        // O que a marcha avalia numa fatia: pontos do tubo (combinações convexas dos cantos) mais a
        // sonda da normal, que se afasta até `pad` — e nada fora da caixa, que é onde ela corta.
        let mut evaluated: Vec<[f32; 3]> = Vec::new();
        for _ in 0..48 {
            let w: Vec<f32> = (0..8).map(|_| -(rng.unit().max(1e-6)).ln()).collect();
            let s: f32 = w.iter().sum();
            let base: [f32; 3] =
                std::array::from_fn(|k| t.pts.iter().zip(&w).map(|(p, wi)| p[k] * wi / s).sum());
            let off = unit(rng.vec(-1.0, 1.0));
            let r = t.pad * rng.unit().cbrt() * (1.0 - 1e-3);
            evaluated.push(std::array::from_fn(|k| base[k] + off[k] * r));
        }
        for p in &t.pts {
            for dk in 0..3 {
                for sgn in [-1.0f32, 1.0] {
                    let mut q = *p;
                    q[dk] += sgn * t.pad * (1.0 - 1e-3);
                    evaluated.push(q);
                }
            }
        }
        evaluated.retain(|p| (0..3).all(|k| p[k] >= t.lo[k] && p[k] <= t.hi[k]));
        for &i in &leaves {
            let (m, _) = rc
                .specialised_leaf(&doc.nodes()[i], i)
                .expect("folha especializada");
            let poly = polygon(h.hull_of(i), m.box_of(t.lo, t.hi));
            for p in m.points_of(&evaluated) {
                measured += 1;
                assert!(
                    in_convex([p[0], p[1]], &poly),
                    "tentativa {trial}, folha {i}: um ponto que a marcha avalia cai FORA da região \
                     da folha ({:?} contra {} vértices) — a especialização deitaria fora a aresta \
                     mais próxima dele",
                    [p[0], p[1]],
                    poly.len()
                );
            }
        }
    }
    assert!(
        measured > 50_000,
        "só {measured} pontos medidos — a fixtura quase não avalia nada"
    );
}
