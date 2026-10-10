//! Gates da SETA do QuickShape (F7d) — irmão de `quickshape_arrow.rs`; as sondas em
//! `quickshape_arrow_probe_tests.rs`.

use super::arc_tests::{arc_poly, stroke};
use super::*;
use crate::test_hand::Lcg;

/// A ponta desenhada: o V aberto (farpa, ponta, farpa), de ida-e-volta (e de novo à ponta), fechado
/// (farpa, farpa, ponta), ou uma farpa só — a mesma mão do `gera_entradas.py`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Kind {
    Open,
    Back,
    Closed,
    One,
}

pub(super) const KINDS: [Kind; 3] = [Kind::Open, Kind::Back, Kind::Closed];

/// As farpas a `deg` graus do cabo e `len` de comprimento, na ponta `tip` com o cabo a chegar de
/// `prev`.
fn barbs(tip: P2, prev: P2, len: f64, deg: f64, kind: Kind) -> Vec<P2> {
    let d = dist(tip, prev);
    let u = mul(sub(prev, tip), 1.0 / d);
    let b1 = add(tip, mul(rotate(u, deg.to_radians()), len));
    let b2 = add(tip, mul(rotate(u, -deg.to_radians()), len));
    match kind {
        Kind::Open => vec![b1, tip, b2],
        Kind::Back => vec![b1, tip, b2, tip],
        Kind::Closed => vec![b1, b2, tip],
        Kind::One => vec![b1],
    }
}

/// A seta: `shaft` (vértices), a ponta `kind` no fim e, com `both`, a de ida-e-volta no início (a
/// mão começa numa farpa, vai à ponta, à outra farpa, volta à ponta e desenha o cabo).
pub(super) fn arrow_poly(shaft: &[P2], len: f64, deg: f64, kind: Kind, both: bool) -> Vec<P2> {
    let n = shaft.len();
    // A de ida-e-volta (farpa, ponta, farpa, ponta) acaba na ponta = o 1.º vértice do cabo.
    let mut poly = if both {
        barbs(shaft[0], shaft[1], len, deg, Kind::Back)
    } else {
        vec![shaft[0]]
    };
    poly.extend_from_slice(&shaft[1..]);
    poly.extend(barbs(shaft[n - 1], shaft[n - 2], len, deg, kind));
    poly
}

/// O cabo de `s` px: a recta, o arco de 120° e o L.
pub(super) fn shafts(s: f64) -> Vec<(&'static str, Vec<P2>)> {
    let r = s / 120f64.to_radians();
    vec![
        ("recto", vec![[0.0, 0.0], [s, 0.0]]),
        ("arco", arc_poly(r, 210.0, 120.0)),
        ("L", vec![[0.0, s / 2.0], [0.0, 0.0], [s / 2.0, 0.0]]),
    ]
}

/// A mão em repouso no fim do traço: `n` amostras a tremer ±`amp` à volta da última.
pub(super) fn rest(pts: &[P2], amp: f64, n: usize, seed: u64) -> Vec<P2> {
    let mut g = Lcg(seed + 31);
    let end = pts[pts.len() - 1];
    let mut out = pts.to_vec();
    out.extend((0..n).map(|_| add(end, [amp * g.next(), amp * g.next()])));
    out
}

/// O que se espera de uma seta: o cabo do tipo do `shaft` e as pontas onde a mão as pôs.
pub(super) fn is_the_arrow(q: &Option<QuickShape>, shaft: &str, both: bool) -> bool {
    let Some(QuickShape::Arrow(a)) = q else {
        return false;
    };
    let kind = match (&*a.shaft, shaft) {
        (QuickShape::Line { .. }, "recto")
        | (QuickShape::Arc(_) | QuickShape::Biarc(_), "arco") => true,
        (QuickShape::Polygon { corners, .. }, "L") => corners.len() == 3,
        _ => false,
    };
    kind && a.heads[1].is_some() && a.heads[0].is_some() == both
}

/// Os «não é seta» de tamanho `s`: o gancho de mão no fim de uma linha (a quina que a lei
/// absorve), o zigue-zague, o L, o V, o taco, a farpa só, o S, o rabisco.
pub(super) fn not_arrows(s: f64, seed: u64) -> Vec<(&'static str, Vec<P2>)> {
    let mut g = Lcg(seed + 17);
    let hook_turn = 100.0 + 60.0 * (g.next() + 1.0) / 2.0;
    let hook = add([s, 0.0], mul(rotate([1.0, 0.0], hook_turn.to_radians()), 0.1 * s));
    let mut taco: Vec<P2> = vec![[-s / 2.0, s * 0.15], [0.0, s * 0.15]];
    taco.extend(arc_poly(s * 0.15, 90.0, -120.0).iter().skip(1));
    let mut scribble = vec![[0.0, 0.0]];
    for _ in 0..30 {
        let q = scribble[scribble.len() - 1];
        scribble.push([
            (q[0] + 0.1 * s * g.next()).clamp(-s / 2.0, s / 2.0),
            (q[1] + 0.1 * s * g.next()).clamp(-s / 2.0, s / 2.0),
        ]);
    }
    let h = 0.62 * s;
    vec![
        ("linha com gancho", vec![[0.0, 0.0], [s, 0.0], hook]),
        (
            "zigue-zague",
            vec![[0.0, 0.0], [s / 4.0, h], [s / 2.0, 0.0], [0.75 * s, h], [s, 0.0]],
        ),
        ("L", vec![[0.0, h], [0.0, 0.0], [s, 0.0]]),
        ("V", vec![[0.0, 0.0], [s / 2.0, h], [s, 0.0]]),
        ("taco", taco),
        (
            "farpa só",
            arrow_poly(&[[0.0, 0.0], [s, 0.0]], 0.15 * s, 30.0, Kind::One, false),
        ),
        (
            "S",
            (0..=100)
                .map(|i| {
                    let t = f64::from(i) / 100.0;
                    [s * t, s / 4.0 * (TAU * t).sin()]
                })
                .collect(),
        ),
        ("rabisco", scribble),
        ("linha", vec![[0.0, 0.0], [s, 0.3 * s]]),
        ("meio arco", arc_poly(s / 2.0, 180.0, 180.0)),
    ]
}

/// Uma mão de `amp` px sobre `poly`, com o estabilizador e o repouso no fim.
pub(super) fn held(poly: &[P2], amp: f64, seed: u64) -> Vec<P2> {
    rest(&stroke(poly, amp, seed), amp.max(1.0), 6, seed)
}

// ── o 2.º oráculo: o Excalidraw (`corre_excalidraw.mjs`) ─────────────────────────────────────────

const EXCALIDRAW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/fixtures_excalidraw"
);

/// O que o Excalidraw não chama seta e o dono chama (o pedido de 10/10: a seta de cabo CURVO, e a
/// de duas pontas — o painel tem «Both Ends»); o cabo em L é o cabo polilinha da nossa lei.
const OWNERS_ARROWS: [&str; 5] = [
    "seta_curva_v_aberto",
    "seta_curva_duas_pontas",
    "seta_cabo_L",
    "seta_recta_duas_pontas",
    "seta_pequena_v_aberto",
];

/// Uma fixture do Excalidraw: o nome, o tipo dele, os pontos da entrada (px) e os do elemento.
fn excalidraw() -> Vec<(String, String, Vec<P2>, Vec<P2>)> {
    let pair = |l: &str| {
        let v: Vec<f64> = l
            .split_whitespace()
            .filter_map(|t| t.parse().ok())
            .collect();
        [v[0], v[1]]
    };
    let mut all: Vec<_> = std::fs::read_dir(EXCALIDRAW)
        .expect("a pasta das fixtures do Excalidraw")
        .filter_map(|e| {
            let path = e.ok()?.path();
            let text = std::fs::read_to_string(&path).ok()?;
            let kind = text.lines().find_map(|l| l.strip_prefix("# tipo: "))?;
            let input = text
                .lines()
                .filter_map(|l| l.strip_prefix("in "))
                .map(pair)
                .collect();
            let out = text
                .lines()
                .filter_map(|l| l.strip_prefix("out "))
                .map(pair)
                .collect();
            let name = path.file_stem()?.to_string_lossy().into_owned();
            Some((name, kind.to_owned(), input, out))
        })
        .collect();
    all.sort_by(|a, b| a.0.cmp(&b.0));
    all
}

/// ⭐ **As setas do Excalidraw são setas nossas, e nada mais é** — exceptuadas as do dono
/// ([`OWNERS_ARROWS`]). Onde ele diz seta: o cabo recto começa onde o dele (≤ 3 px: o dele é a 1.ª
/// amostra, com o tremor dela; o nosso, a recta ajustada) e aponta para onde o dele (≤ 5°: o fim do
/// dele fica até 38 px AQUÉM da ponta desenhada, o nosso vai até ela — o nosso cabo nunca é mais
/// curto, a menos de 1 px).
#[test]
fn the_excalidraw_arrows_are_arrows_and_nothing_else_is() {
    let all = excalidraw();
    assert!(all.len() >= 80, "só {} fixtures lidas", all.len());
    let theirs = all.iter().filter(|f| f.1 == "arrow").count();
    assert!(theirs >= 9, "controlo positivo: {theirs} setas do Excalidraw");
    let mut bad = Vec::new();
    for (name, kind, input, out) in &all {
        let ours = recognise(input, 1.0, false);
        let verdict = match (&ours, kind.as_str()) {
            (Some(QuickShape::Arrow(a)), "arrow") => match (&*a.shaft, a.heads) {
                (QuickShape::Line { a: p, b: q }, [None, Some(_)]) => {
                    let (e0, e1) = (out[0], out[1]);
                    let turn = wrap_pi(angle(sub(*q, *p)) - angle(sub(e1, e0))).abs();
                    (dist(*p, e0) <= 3.0
                        && turn <= 5f64.to_radians()
                        && dist(*p, *q) >= dist(e0, e1) - 1.0)
                    .then_some(())
                    .ok_or(format!("cabo {p:?}→{q:?} vs {e0:?}→{e1:?}"))
                }
                other => Err(format!("cabo/pontas {other:?}")),
            },
            (other, "arrow") => Err(format!("o Excalidraw diz seta, saiu {other:?}")),
            (Some(QuickShape::Arrow(_)), _) if OWNERS_ARROWS.contains(&name.as_str()) => Ok(()),
            (Some(QuickShape::Arrow(_)), k) => Err(format!("o Excalidraw diz {k}, saiu seta")),
            (_, _) if OWNERS_ARROWS.contains(&name.as_str()) => {
                Err(format!("a seta do dono saiu {ours:?}"))
            }
            _ => Ok(()),
        };
        if let Err(e) = verdict {
            bad.push(format!("{name}: {e}"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
