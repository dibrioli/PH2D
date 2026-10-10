//! Gates da SETA do QuickShape (F7d) — irmão de `quickshape_arrow.rs`; as sondas em
//! `quickshape_arrow_probe_tests.rs`.

use super::arc_tests::arc_poly;
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

/// As farpas a `deg ± skew` graus do cabo e `len` de comprimento, na ponta `tip` com o cabo a chegar
/// de `prev` (a mão real não as desenha simétricas).
fn barbs(tip: P2, prev: P2, len: f64, deg: f64, skew: f64, kind: Kind) -> Vec<P2> {
    let d = dist(tip, prev);
    let u = mul(sub(prev, tip), 1.0 / d);
    let b1 = add(tip, mul(rotate(u, (deg + skew).to_radians()), len));
    let b2 = add(tip, mul(rotate(u, -(deg - skew).to_radians()), len));
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
    skewed_arrow(shaft, len, deg, 0.0, kind, both)
}

/// [`arrow_poly`] com as farpas a `deg + skew` e `deg − skew`.
pub(super) fn skewed_arrow(
    shaft: &[P2],
    len: f64,
    deg: f64,
    skew: f64,
    kind: Kind,
    both: bool,
) -> Vec<P2> {
    let n = shaft.len();
    // A de ida-e-volta (farpa, ponta, farpa, ponta) acaba na ponta = o 1.º vértice do cabo.
    let mut poly = if both {
        barbs(shaft[0], shaft[1], len, deg, skew, Kind::Back)
    } else {
        vec![shaft[0]]
    };
    poly.extend_from_slice(&shaft[1..]);
    poly.extend(barbs(shaft[n - 1], shaft[n - 2], len, deg, skew, kind));
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
    let hook = add(
        [s, 0.0],
        mul(rotate([1.0, 0.0], hook_turn.to_radians()), 0.1 * s),
    );
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
            vec![
                [0.0, 0.0],
                [s / 4.0, h],
                [s / 2.0, 0.0],
                [0.75 * s, h],
                [s, 0.0],
            ],
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
        // O T: a barra a 90° nos dois lados (é a ponta `Bar`, não uma seta).
        (
            "T",
            vec![
                [0.0, 0.0],
                [s, 0.0],
                [s, 0.12 * s],
                [s, 0.0],
                [s, -0.12 * s],
                [s, 0.0],
            ],
        ),
        // A linha que VOLTA ATRÁS sobre si e só então faz uma farpa (uma farpa a 0°).
        (
            "linha que volta atrás",
            vec![
                [0.0, 0.0],
                [s, 0.0],
                [0.85 * s, 0.0],
                [s, 0.0],
                [s - 0.12 * s, 0.1 * s],
            ],
        ),
        // A pentagrama numa passada: cinco bicos em V (o gate `regular_keeps_a_star_a_star`).
        (
            "estrela",
            [0, 2, 4, 1, 3, 0]
                .map(|k| mul(unit(FRAC_PI_2 + TAU * f64::from(k) / 5.0), s / 2.0))
                .to_vec(),
        ),
    ]
}

/// A mão com o repouso de ±`rest_amp` px e o estabilizador `stab` (0 = desligado) — que filtra a
/// amostra na ENTRADA, o repouso incluído (é o lápis: `Pencil::on_drag` recebe o ponto já filtrado).
pub(super) fn held_with(poly: &[P2], amp: f64, seed: u64, stab: f64, rest_amp: f64) -> Vec<P2> {
    let h = rest(
        &crate::test_hand::hand(poly, 2.0, amp, seed),
        rest_amp,
        6,
        seed,
    );
    if stab > 0.0 {
        crate::test_hand::stabilise(&h, stab)
    } else {
        h
    }
}

/// O caminho amostrado (as cúbicas, 32 pontos por troço).
pub(super) fn path_points(p: &VecPath) -> Vec<P2> {
    let mut out = vec![p.verts[0].anchor];
    for w in p.verts.windows(2) {
        let (a, b, c, d) = (w[0].anchor, w[0].out_handle, w[1].in_handle, w[1].anchor);
        out.extend((1..=32).map(|i| {
            let t = f64::from(i) / 32.0;
            let u = 1.0 - t;
            add(
                add(mul(a, u * u * u), mul(b, 3.0 * u * u * t)),
                add(mul(c, 3.0 * u * t * t), mul(d, t * t * t)),
            )
        }));
    }
    out
}

/// A distância de Hausdorff entre o caminho de `q` e a polilinha `poly` (as duas reamostradas).
pub(super) fn hausdorff(q: &QuickShape, poly: &[P2]) -> f64 {
    let a = path_points(&q.to_path());
    let b = crate::quickshape_geom::resample(poly, 400);
    let one = |x: &[P2], y: &[P2]| {
        x.iter()
            .map(|&p| y.iter().map(|&q| dist(p, q)).fold(f64::MAX, f64::min))
            .fold(0.0, f64::max)
    };
    one(&a, &b).max(one(&b, &a))
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
    assert!(
        theirs >= 9,
        "controlo positivo: {theirs} setas do Excalidraw"
    );
    let bad = excalidraw_failures(&Law::MEASURED);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// O veredito de cada fixture do Excalidraw sob `law` — a régua do gate e da sonda das faixas.
pub(super) fn excalidraw_failures(law: &Law) -> Vec<String> {
    let mut bad = Vec::new();
    for (name, kind, input, out) in &excalidraw() {
        let ours = recognise_with(law, input, 1.0, false);
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
    bad
}

// ── a lei, sobre as famílias de mão ─────────────────────────────────────────────────────────────

/// ⭐ **As setas de mão saem a seta certa** — cabo recto, arco ou L; V aberto, de ida-e-volta ou
/// fechado; uma ou duas pontas; farpas de 20° e 60° (e assimétricas ±10°), de 15 % e 35 % do cabo;
/// tremor ±0,5–2 px, o estabilizador e o repouso. 160 e 400 px (as pequenas: a tabela da sonda).
#[test]
fn hand_arrows_come_out_as_the_arrow() {
    let mut bad = Vec::new();
    for s in [160.0, 400.0] {
        for (name, shaft) in shafts(s) {
            for (k, kind) in KINDS.into_iter().enumerate() {
                for both in [false, true] {
                    for (deg, skew) in [(20.0, 0.0), (60.0, 0.0), (35.0, 10.0)] {
                        for frac in [0.15, 0.35] {
                            let (seed, amp) = [(0, 0.5), (1, 1.0), (2, 2.0)][k];
                            let p = skewed_arrow(&shaft, frac * s, deg, skew, kind, both);
                            let q =
                                recognise(&held_with(&p, amp, seed, 0.5, amp.max(1.0)), 1.0, false);
                            if !is_the_arrow(&q, name, both) {
                                bad.push(format!(
                                    "{s} {name} {kind:?} {both} {deg}±{skew} {frac}: {q:?}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} falharam:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// ⭐ **O que não é seta nunca sai seta** — o gancho de mão no fim de uma linha (a quina que a lei
/// já absorve), o zigue-zague, o L, o V, o taco, a farpa só, o S, a linha, o meio arco e as formas
/// das sondas, o T e a estrela, de 48 a 400 px, tremor ±0,5–2 px e o repouso. Ficam de fora, MEDIDOS
/// e não gateados: o RABISCO aleatório (0 % no ponto da lei) e a LINHA QUE VOLTA ATRÁS sobre si e
/// só então faz uma farpa — à vista é uma farpa só (a volta fica por baixo do traço), e nenhum
/// ângulo mínimo a separa das farpas assimétricas de 10° (até 50 % a 48–160 px, 10 % a 400 px —
/// `00_resultados.md` §3h).
#[test]
fn what_is_not_an_arrow_never_comes_out_as_one() {
    let mut bad = Vec::new();
    for s in [48.0, 96.0, 160.0, 400.0] {
        for seed in 0..3 {
            let mut all: Vec<(String, Vec<P2>)> = not_arrows(s, seed)
                .into_iter()
                .map(|(n, p)| (n.to_owned(), p))
                .collect();
            all.extend(
                super::measure_tests::cases(s)
                    .into_iter()
                    .map(|(n, p, _)| (n.to_owned(), p)),
            );
            let measured = ["rabisco", "linha que volta atrás"];
            for (name, p) in all.iter().filter(|(n, _)| !measured.contains(&n.as_str())) {
                for amp in [0.5, 1.0, 2.0] {
                    let q = recognise(&held_with(p, amp, seed, 0.5, amp.max(1.0)), 1.0, false);
                    if let Some(QuickShape::Arrow(a)) = q {
                        bad.push(format!("{s} {name} ±{amp} (mão {seed}): {a:?}"));
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} falsas setas:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// **A ponta nunca piora o cabo, e cai onde a mão a pôs** — o cabo da seta contra o MESMO cabo
/// desenhado sozinho (a mesma mão até à ponta), Hausdorff ao desenho, 96–400 px (as pequenas
/// nem sempre saem seta: a tabela da sonda): o fim do cabo a ≤ 4 px da ponta desenhada.
#[test]
fn the_head_never_makes_the_shaft_worse_and_the_tip_lands_where_the_hand_put_it() {
    use crate::test_hand::{hand, stabilise};
    let (mut worse, mut tip, mut n) = (f64::MIN, 0.0_f64, 0);
    for s in [96.0, 160.0, 400.0] {
        for (_, shaft) in shafts(s) {
            for kind in KINDS {
                for seed in 0..3 {
                    let p = arrow_poly(&shaft, 0.2 * s, 30.0, kind, false);
                    let whole = rest(&stabilise(&hand(&p, 2.0, 1.0, seed), 0.5), 1.0, 6, seed);
                    let alone = rest(&stabilise(&hand(&shaft, 2.0, 1.0, seed), 0.5), 1.0, 6, seed);
                    let Some(QuickShape::Arrow(a)) = recognise(&whole, 1.0, false) else {
                        panic!("{s} {kind:?} (mão {seed}): não saiu seta");
                    };
                    let b = recognise(&alone, 1.0, false).expect("o cabo sozinho é forma");
                    worse = worse.max(hausdorff(&a.shaft, &shaft) - hausdorff(&b, &shaft));
                    let end = a.shaft.to_path().verts.last().unwrap().anchor;
                    tip = tip.max(dist(end, shaft[shaft.len() - 1]));
                    n += 1;
                }
            }
        }
    }
    assert!(n >= 27, "controlo: {n} pares");
    assert!(
        worse <= 2.0,
        "o cabo com ponta pior {worse:.2} px que sozinho"
    );
    assert!(tip <= 4.0, "a ponta a {tip:.2} px da desenhada");
}

/// **`regular` (Shift / o 2.º dedo)**: o cabo regulariza como sem ponta — a linha a 15° — e as
/// pontas FICAM (são o desenho, não a forma: o `regular` não as inventa nem as tira).
#[test]
fn regular_squares_the_shaft_and_keeps_the_heads() {
    let a = 37f64.to_radians();
    let shaft = [[0.0, 0.0], [300.0 * a.cos(), 300.0 * a.sin()]];
    let h = held_with(
        &arrow_poly(&shaft, 60.0, 30.0, Kind::Open, true),
        1.0,
        0,
        0.5,
        1.0,
    );
    let angle_of = |q: &Option<QuickShape>| match q {
        Some(QuickShape::Arrow(ArrowShape { shaft, heads })) => match **shaft {
            QuickShape::Line { a, b } => {
                (angle(sub(b, a)).to_degrees(), heads.map(|h| h.is_some()))
            }
            ref other => panic!("cabo {other:?}"),
        },
        other => panic!("{other:?}"),
    };
    let (free, heads) = angle_of(&recognise(&h, 1.0, false));
    assert!((free - 37.0).abs() < 2.0, "sem regular: {free}°");
    let (reg, reg_heads) = angle_of(&recognise(&h, 1.0, true));
    assert!(
        (reg - 30.0).abs() < 1e-6 || (reg - 45.0).abs() < 1e-6,
        "regular: {reg}°"
    );
    assert_eq!(heads, [true, true]);
    assert_eq!(reg_heads, heads, "as pontas ficam");
}

/// A semelhança leva o cabo E as farpas (o arrasto escala a seta inteira).
#[test]
fn transformed_scales_the_heads_with_the_shaft() {
    let arrow = QuickShape::Arrow(ArrowShape {
        shaft: Box::new(QuickShape::Line {
            a: [0.0, 0.0],
            b: [100.0, 0.0],
        }),
        heads: [None, Some(20.0)],
    });
    let t = arrow.transformed(arrow.pivot(), 2.0, FRAC_PI_2);
    let QuickShape::Arrow(ArrowShape { shaft, heads }) = &t else {
        panic!("{t:?}");
    };
    assert_eq!(*heads, [None, Some(40.0)]);
    let QuickShape::Line { a, b } = **shaft else {
        panic!("{shaft:?}");
    };
    assert!(
        dist(a, [0.0, 0.0]) < 1e-9 && dist(b, [0.0, 200.0]) < 1e-9,
        "{a:?} {b:?}"
    );
    assert_eq!(t.to_path().verts.len(), 2, "o caminho é o do cabo");
}

// ── o gesto, pelo lápis ─────────────────────────────────────────────────────────────────────────

use super::pencil_tests::draw;
use crate::quickshape_hold::{HOLD_NS, HoldTick};
use ph2d_vec_scene::Marker;

/// Uma seta de mão de cabo recto de 300 px a 37° e farpas de 60 px: a ponta desenhada.
fn arrow_hand() -> (Vec<P2>, P2) {
    let a = 37f64.to_radians();
    let tip = [300.0 * a.cos(), 300.0 * a.sin()];
    let p = arrow_poly(&[[0.0, 0.0], tip], 60.0, 30.0, Kind::Open, false);
    (held_with(&p, 1.0, 0, 0.5, 1.0), tip)
}

fn held_pencil(style: crate::PenStyle) -> (ph2d_vec_scene::VecScene, crate::Pencil, P2, u128) {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    pencil.set_style(style);
    let (pts, tip) = arrow_hand();
    let t = draw(&mut pencil, &mut scene, &pts);
    let id = pencil.active_path().unwrap();
    assert_eq!(
        scene.path(id).unwrap().stroke.as_ref().unwrap().marker_end,
        style.marker_end,
        "o traço livre tem a ponta do estilo"
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    (scene, pencil, tip, t)
}

/// ⭐ **Segurar a seta põe a ponta no traço vivo, e soltar guarda-a** — a `Open` (a «arrow» de
/// nascença do Excalidraw) com o lápis sem ponta armada; o tamanho é o das farpas (o alcance da
/// `Open` é √20 larguras); o caminho acaba na ponta desenhada.
#[test]
fn holding_an_arrow_puts_the_head_on_the_live_path_and_release_keeps_it() {
    let (mut scene, mut pencil, tip, _) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!((s.marker_start, s.marker_end), (Marker::None, Marker::Open));
    let drawn = 60.0 / (3.0 * 20f64.sqrt());
    assert!(
        (s.marker_scale / drawn - 1.0).abs() < 0.35,
        "tamanho {} vs {drawn}",
        s.marker_scale
    );
    assert_eq!(p.verts.len(), 2, "o cabo é uma recta: {:?}", p.verts);
    assert!(
        dist(p.verts[1].anchor, tip) < 4.0,
        "{:?} vs {tip:?}",
        p.verts[1].anchor
    );
    assert!(pencil.on_release(&mut scene));
    let s = scene.path(id).unwrap().stroke.as_ref().unwrap();
    assert_eq!(
        s.marker_end,
        Marker::Open,
        "soltar comete a seta com a ponta"
    );
}

/// **`Esc` devolve o traço livre SEM ponta** (e com o tamanho de ponta do estilo).
#[test]
fn esc_gives_back_the_free_stroke_without_the_head() {
    let (mut scene, mut pencil, _, _) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    assert!(pencil.quick_dismiss(&mut scene));
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(
        (s.marker_start, s.marker_end, s.marker_scale),
        (Marker::None, Marker::None, 1.0)
    );
    assert!(
        p.verts.len() > 2,
        "o traço livre volta: {} nós",
        p.verts.len()
    );
}

/// ⭐ **Arrastar escala e roda a seta INTEIRA** — o cabo à volta do início e a ponta com ele (o
/// tamanho dobra quando a seta dobra).
#[test]
fn the_drag_scales_and_turns_the_whole_arrow() {
    let (mut scene, mut pencil, _, t) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    let (s0, v0) = {
        let p = scene.path(id).unwrap();
        (p.stroke.as_ref().unwrap().marker_scale, p.verts.clone())
    };
    let end = *pencil.samples().last().unwrap();
    let pivot = v0[0].anchor;
    let far = add(pivot, mul(rotate(sub(end, pivot), FRAC_PI_2), 2.0));
    let dy = crate::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: t + HOLD_NS,
    };
    assert!(pencil.on_drag(&mut scene, far, dy));
    let p = scene.path(id).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(s.marker_end, Marker::Open);
    assert!(
        (s.marker_scale / s0 - 2.0).abs() < 1e-9,
        "a ponta dobra: {s0} → {}",
        s.marker_scale
    );
    let want = add(pivot, mul(rotate(sub(v0[1].anchor, pivot), FRAC_PI_2), 2.0));
    assert!(dist(p.verts[0].anchor, pivot) < 1e-9, "o início fica");
    assert!(
        dist(p.verts[1].anchor, want) < 1e-6,
        "{:?} vs {want:?}",
        p.verts[1].anchor
    );
}

/// **O `Shift` a meio** regulariza o cabo e a seta continua seta, com a ponta.
#[test]
fn the_shift_mid_hold_keeps_the_arrow_and_its_head() {
    let (mut scene, mut pencil, _, t) = held_pencil(crate::PenStyle::default());
    let id = pencil.active_path().unwrap();
    assert_eq!(
        pencil.hold_tick(&mut scene, t + 2 * HOLD_NS, true, true),
        HoldTick::Reshaped
    );
    let p = scene.path(id).unwrap();
    assert_eq!(p.stroke.as_ref().unwrap().marker_end, Marker::Open);
    let deg = angle(sub(p.verts[1].anchor, p.verts[0].anchor)).to_degrees();
    assert!(
        (deg - 30.0).abs() < 1e-6 || (deg - 45.0).abs() < 1e-6,
        "{deg}°"
    );
}

/// **A ponta ARMADA no lápis é a da seta, e o bico cai na ponta desenhada** — o triângulo cresce
/// para lá do nó (`Marker::inset`), por isso o caminho recua o que ele avança.
#[test]
fn an_armed_head_is_the_arrows_and_its_tip_lands_on_the_drawn_tip() {
    let style = crate::PenStyle {
        marker_end: Marker::Triangle,
        ..crate::PenStyle::default()
    };
    let (scene, pencil, tip, _) = held_pencil(style);
    let p = scene.path(pencil.active_path().unwrap()).unwrap();
    let s = p.stroke.as_ref().unwrap();
    assert_eq!(s.marker_end, Marker::Triangle);
    let (node, dir) = ph2d_vec_scene::end_tangent(p, false).unwrap();
    let bico = add(
        node,
        mul(dir, Marker::Triangle.inset(s.marker_scale) * s.width),
    );
    assert!(
        dist(bico, tip) < 4.0,
        "o bico em {bico:?}, a mão em {tip:?}"
    );
    assert!(dist(node, tip) > 20.0, "o caminho recuou: {node:?}");
}

/// **Quando a forma deixa de ser seta, as pontas voltam às do estilo** — a função que o `rebuild`
/// chama a cada quadro: com farpas, as da seta; sem, as do estilo, tamanho incluído.
#[test]
fn the_ends_go_back_to_the_style_when_the_shape_stops_being_an_arrow() {
    use crate::pencil::arrow_ends;
    let style = crate::PenStyle {
        marker_start: Marker::Bar,
        marker_end: Marker::Triangle,
        marker_scale: 1.7,
        ..crate::PenStyle::default()
    };
    let arrow = arrow_ends(&style, [None, Some(30.0)], 3.0);
    assert_eq!(
        arrow.markers,
        [Marker::None, Marker::Triangle],
        "só onde a mão desenhou"
    );
    assert!(arrow.inset[1] > 0.0 && arrow.inset[0] == 0.0);
    let back = arrow_ends(&style, [None; 2], 3.0);
    assert_eq!(back.markers, [Marker::Bar, Marker::Triangle]);
    assert_eq!((back.scale, back.inset), (1.7, [0.0; 2]));
    let plain = arrow_ends(&crate::PenStyle::default(), [Some(30.0), Some(30.0)], 3.0);
    assert_eq!(
        plain.markers,
        [Marker::Open; 2],
        "sem ponta armada: a Open nas duas"
    );
}

/// ⭐ **A ponta da seta não passa ao traço seguinte** (a foto da cena 12, 10/10: o gelado solto
/// nascia com seta). Ao soltar, a seta é selecionada e a tool ADOTA o traço dela; o próximo caminho
/// nasce com as pontas armadas ANTES da seta — até o artista mexer nas pontas ou selecionar outra
/// coisa, e aí volta o «último usado».
#[test]
fn the_arrows_head_does_not_pass_to_the_next_stroke() {
    use crate::PenStyle;
    let (mut scene, mut pencil, _, t) = held_pencil(PenStyle::default());
    let id = pencil.active_path().unwrap();
    assert!(pencil.on_release(&mut scene));
    let s = scene.path(id).unwrap().stroke.clone().unwrap();
    let adopted = PenStyle {
        marker_start: s.marker_start,
        marker_end: s.marker_end,
        marker_scale: s.marker_scale,
        ..PenStyle::default()
    };
    assert_eq!(
        adopted.marker_end,
        Marker::Open,
        "controlo: a tool adotou a seta"
    );
    let next = pencil.next_style(adopted, Some(id));
    assert_eq!(
        (next.marker_start, next.marker_end, next.marker_scale),
        (Marker::None, Marker::None, 1.0),
        "o próximo caminho nasce com as pontas de antes"
    );
    assert_eq!(
        pencil.next_style(adopted, None),
        adopted,
        "outra seleção: a lei"
    );
    let edited = PenStyle {
        marker_end: Marker::Triangle,
        ..adopted
    };
    assert_eq!(
        pencil.next_style(edited, Some(id)),
        edited,
        "o artista mexeu: a lei"
    );
    // E o traço seguinte, com esse estilo, nasce sem ponta — e um traço livre não arma nada.
    pencil.set_style(next);
    let line = held_with(&[[0.0, 0.0], [200.0, 80.0]], 1.0, 3, 0.5, 1.0);
    let dy = |i: usize| crate::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: t + (i as u128 + 1) * 4_000_000,
    };
    let free = pencil.on_press(&mut scene, line[0], 1.0, dy(0));
    for (i, &p) in line.iter().enumerate().skip(1) {
        pencil.on_drag(&mut scene, p, dy(i));
    }
    assert!(pencil.on_release(&mut scene));
    assert_eq!(
        scene
            .path(free)
            .unwrap()
            .stroke
            .as_ref()
            .unwrap()
            .marker_end,
        Marker::None
    );
    assert_eq!(
        pencil.next_style(adopted, Some(free)),
        adopted,
        "um traço livre não é seta"
    );
}
