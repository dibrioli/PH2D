//! **A varredura do Offset na estrela, versionada como gate** (prompt 49, a cerca da linha).
//!
//! Estrela do gate [`the_offset_never_takes_the_app_down`](super::the_offset_never_takes_the_app_down)
//! (`R = 1,4`, `r = 0,63`), 100 distâncias `0,04..4,0` por junção, para fora e para dentro — as
//! 600 combinações do estudo `docs/Vector Module/Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md`
//! §3. Cada uma é comparada com a **verdade analítica** do polígono:
//!
//! - fora: `S ∪ B`, dentro: `S ∖ B`, onde `B` = o traço de `∂S` com largura `2d`, a MESMA junção
//!   e `miterlimit 4` (retângulo por aresta + a peça da junção no lado de fora da curva);
//! - a régua é a máscara AMOSTRADA ([`super::raster`]): `ok` ⟺ `xor ≤ 0,5 %` da área de
//!   referência (a verdade, fora; a estrela, dentro — perto da morte a verdade tende a zero).
//!
//! A verdade analítica é atada a um rasterizador EXTERNO pela fixture
//! `fixtures/estrela_offset_resvg.tsv` (áreas do `resvg`, cabeçalho com origem, comando e data):
//! se a analítica desviar dele, o gate cai antes de medir o produto.
//!
//! ⚠️ A tabela [`EXPECTED`] é a coluna do PRODUTO de hoje. Cada onda que a mexe escreve o número
//! novo aqui e na mensagem do commit; o gate falha nos dois sentidos (pior **e** melhor sem
//! registo), porque uma melhoria não registada é uma régua que ninguém relê.

use std::cell::Cell;
use std::collections::BTreeMap;

use kurbo::Point;
use ph2d_vec_scene::{FillRule, LineJoin, OffsetSide, ShapeKind, VecPath, cook};

use super::raster::{
    Grid, anchors, bbox, count, rasterize, rasterize_paths, signed_area, xor_count,
};

/// Lado da grelha, em pixels (o maior lado da caixa).
const GRID_N: usize = 480;
/// `xor / referência` acima disto = resposta errada (o critério do prompt 49, W2).
const OK_PCT: f64 = 0.5;
/// O `miterlimit` da caneta do Offset (o `Stroke` da kurbo por omissão = o SVG por omissão).
const MITER_LIMIT: f64 = 4.0;

/// A estrela do gate de robustez — a MESMA chamada.
pub fn star() -> VecPath {
    cook(
        ShapeKind::Star,
        [-3.6, -1.4],
        [-0.8, 1.4],
        &[5.0, 0.45, 0.0],
    )
}

/// `d` da combinação `i` (1..=100), como no estudo: `0,04 · i`.
pub fn dist(i: u32) -> f64 {
    f64::from(i) * 0.04
}

pub const JOINS: [(&str, LineJoin); 3] = [
    ("miter", LineJoin::Miter),
    ("round", LineJoin::Round),
    ("bevel", LineJoin::Bevel),
];

/// Orienta um polígono com área positiva (a união por NonZero só vale com todos no mesmo sentido).
fn ccw(mut p: Vec<Point>) -> Option<Vec<Point>> {
    let a = signed_area(&p);
    if a.abs() < 1e-18 {
        return None;
    }
    if a < 0.0 {
        p.reverse();
    }
    Some(p)
}

/// **O traço analítico** da fronteira do polígono `poly`, largura `2d`, junção `join`: um
/// retângulo por aresta + a peça da junção no lado de FORA da curva em cada vértice. Todas as
/// peças com área positiva — a união é o NonZero delas.
pub fn stroke_parts(poly: &[Point], d: f64, join: LineJoin) -> Vec<Vec<Point>> {
    let n = poly.len();
    let mut parts = Vec::new();
    let unit = |a: Point, b: Point| (b - a) / (b - a).hypot();
    let left = |u: kurbo::Vec2| kurbo::Vec2::new(-u.y, u.x);
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let nn = left(unit(a, b)) * d;
        parts.extend(ccw(vec![a + nn, b + nn, b - nn, a - nn]));
    }
    for i in 0..n {
        let v = poly[i];
        let u0 = unit(poly[(i + n - 1) % n], v);
        let u1 = unit(v, poly[(i + 1) % n]);
        let cross = u0.cross(u1);
        if cross.abs() < 1e-12 {
            continue;
        }
        let s = -cross.signum();
        let (n0, n1) = (left(u0) * s, left(u1) * s);
        let (p0, p1) = (v + n0 * d, v + n1 * d);
        match join {
            LineJoin::Round => {
                let k = 720;
                let disc = (0..k)
                    .map(|j| {
                        let t = std::f64::consts::TAU * f64::from(j) / f64::from(k);
                        v + kurbo::Vec2::new(t.cos(), t.sin()) * d
                    })
                    .collect();
                parts.extend(ccw(disc));
            }
            LineJoin::Bevel => parts.extend(ccw(vec![v, p0, p1])),
            LineJoin::Miter => {
                let bis = n0 + n1;
                let half_cos = bis.hypot() / 2.0; // cos(φ/2), φ = ângulo de curva
                if half_cos > 1e-12 && 1.0 / half_cos <= MITER_LIMIT {
                    let m = v + bis / bis.hypot() * (d / half_cos);
                    parts.extend(ccw(vec![v, p0, m, p1]));
                } else {
                    parts.extend(ccw(vec![v, p0, p1]));
                }
            }
        }
    }
    parts
}

/// O que UMA combinação mediu.
#[derive(Clone, Copy, Debug)]
pub struct Cell1 {
    pub d100: i32,
    pub ours_n: usize,
    /// `xor / referência`, em %.
    pub diff_pct: f64,
    /// Área da verdade, em unidades² (a régua que a fixture do `resvg` ata).
    pub truth_area: f64,
}

/// Mede a combinação `(join, sign·dist(i))` com a grelha da verdade.
pub fn measure(star: &VecPath, join: LineJoin, sign: f64, i: u32) -> Cell1 {
    let d = dist(i);
    let poly = anchors(star);
    let s_poly = vec![ccw(poly.clone()).expect("a estrela tem área")];
    let band = stroke_parts(&poly, d, join);
    let (lo, hi) = if sign > 0.0 {
        let mut all = band.clone();
        all.extend(s_poly.iter().cloned());
        bbox(&all).expect("a verdade tem pontos")
    } else {
        bbox(&s_poly).expect("a estrela tem pontos")
    };
    let g = Grid::covering(lo, hi, GRID_N);
    let s_mask = rasterize(&s_poly, FillRule::NonZero, &g);
    let b_mask = rasterize(&band, FillRule::NonZero, &g);
    let truth: Vec<bool> = s_mask
        .iter()
        .zip(&b_mask)
        .map(|(&s, &b)| if sign > 0.0 { s || b } else { s && !b })
        .collect();
    let ours = ph2d_vec_boolean::offset_path(star, sign * d, join, OffsetSide::Outer);
    let ours_mask = rasterize_paths(&ours, &g, g.px * 0.05);
    let reference = if sign > 0.0 {
        count(&truth)
    } else {
        count(&s_mask)
    };
    Cell1 {
        d100: (sign * d * 100.0).round() as i32,
        ours_n: ours.len(),
        diff_pct: 100.0 * xor_count(&truth, &ours_mask) as f64 / reference.max(1) as f64,
        truth_area: count(&truth) as f64 * g.px_area(),
    }
}

thread_local! {
    /// Pânicos vistos NESTA thread enquanto o gate mede (o hook abaixo os conta).
    static PANICS: Cell<u64> = const { Cell::new(0) };
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

/// Corre `f` contando os pânicos da thread — inclusive os que um `catch_unwind` engole. Fora
/// desta janela o hook anterior responde, então os outros testes do binário não perdem a voz.
pub fn counting_panics<T>(f: impl FnOnce() -> T) -> (T, u64) {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if COUNTING.with(Cell::get) {
                PANICS.with(|c| c.set(c.get() + 1));
            } else {
                prev(info);
            }
        }));
    });
    PANICS.with(|c| c.set(0));
    COUNTING.with(|c| c.set(true));
    let out = f();
    COUNTING.with(|c| c.set(false));
    (out, PANICS.with(Cell::get))
}

/// A coluna do PRODUTO: por `(junção, lado)`, as distâncias (`d·100`) que NÃO acertam.
///
/// | onda | round fora | bevel fora | bevel dentro | pânicos |
/// |---|---|---|---|---|
/// | merge-base (0.4) | 87/100 | 35/100 | 94/100 | 104 |
/// | W1 (`linesweeper` 0.5) | 100/100 | 40/100 (vazio ≥ 164) | 94/100 (vazio 44..64) | 0 |
const EXPECTED: &[(&str, &str, &str)] = &[
    ("miter", "fora", ""),
    ("miter", "dentro", ""),
    ("round", "fora", ""),
    ("round", "dentro", ""),
    (
        "bevel",
        "fora",
        "164 168 172 176 180 184 188 192 196 200 204 208 212 216 220 224 228 232 236 240 244 248 \
         252 256 260 264 268 272 276 280 284 288 292 296 300 304 308 312 316 320 324 328 332 336 \
         340 344 348 352 356 360 364 368 372 376 380 384 388 392 396 400",
    ),
    ("bevel", "dentro", "44 48 52 56 60 64"),
];
/// Pânicos que o `catch_unwind` do `offset_path` engole na varredura inteira.
const EXPECTED_PANICS: u64 = 0;

/// A varredura inteira: `(junção, lado) → células`.
pub fn sweep() -> (BTreeMap<(&'static str, &'static str), Vec<Cell1>>, u64) {
    let star = star();
    counting_panics(|| {
        let mut t = BTreeMap::new();
        for (jn, j) in JOINS {
            for (side, sign) in [("fora", 1.0), ("dentro", -1.0)] {
                let cells = (1..=100).map(|i| measure(&star, j, sign, i)).collect();
                t.insert((jn, side), cells);
            }
        }
        t
    })
}

fn not_ok(cells: &[Cell1]) -> String {
    cells
        .iter()
        .filter(|c| c.diff_pct > OK_PCT)
        .map(|c| c.d100.abs().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A fixture do `resvg`: `(junção, lado, i) → área`.
fn resvg_areas() -> BTreeMap<(String, &'static str, u32), f64> {
    include_str!("fixtures/estrela_offset_resvg.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let side = if f[1] == "1" { "fora" } else { "dentro" };
            let i: u32 = f[2].parse().expect("i");
            ((f[0].to_owned(), side, i), f[3].parse().expect("área"))
        })
        .collect()
}

/// ⚠️ A régua antes do produto: a verdade analítica tem de bater o `resvg` em TODAS as 600.
fn assert_truth_is_tied_to_resvg(table: &BTreeMap<(&'static str, &'static str), Vec<Cell1>>) {
    let fx = resvg_areas();
    assert_eq!(
        fx.len(),
        600,
        "a fixture do resvg tem de cobrir as 600 combinações"
    );
    let s_area = signed_area(&anchors(&star())).abs();
    let mut worst = (0.0_f64, String::new());
    for ((jn, side), cells) in table {
        for (k, c) in cells.iter().enumerate() {
            let r = fx[&((*jn).to_owned(), *side, k as u32 + 1)];
            // Fora a referência é a própria área; dentro, a estrela (perto da morte a área → 0).
            let den = if *side == "fora" { r } else { s_area };
            let rel = (c.truth_area - r).abs() / den * 100.0;
            if rel > worst.0 {
                worst = (rel, format!("{jn} {side} d={}", c.d100));
            }
        }
    }
    eprintln!(
        "verdade analítica × resvg: pior {:.3} % ({})",
        worst.0, worst.1
    );
    assert!(
        worst.0 <= 0.3,
        "a verdade analítica deixou de bater o resvg: {:.3} % em {} (a régua mente antes do produto)",
        worst.0,
        worst.1
    );
}

#[test]
fn the_offset_sweep_matches_the_table() {
    let (table, panics) = sweep();
    assert_truth_is_tied_to_resvg(&table);
    let mut report = String::new();
    let mut bad = Vec::new();
    for &(jn, side, want) in EXPECTED {
        let cells = &table[&(jn, side)];
        let got = not_ok(cells);
        let ok = cells.iter().filter(|c| c.diff_pct <= OK_PCT).count();
        let empty = cells
            .iter()
            .filter(|c| c.diff_pct > OK_PCT && c.ours_n == 0)
            .count();
        let worst = cells
            .iter()
            .filter(|c| c.diff_pct <= OK_PCT)
            .map(|c| c.diff_pct)
            .fold(0.0_f64, f64::max);
        report.push_str(&format!(
            "{jn:5} {side:6} ok {ok:3}/100 (vazios entre os errados: {empty}, pior acerto {worst:.3} %) \
             erram em: [{got}]\n"
        ));
        if got != want {
            bad.push(format!("{jn} {side}: esperado [{want}], medido [{got}]"));
        }
    }
    report.push_str(&format!("pânicos engolidos pelo catch_unwind: {panics}\n"));
    eprintln!("{report}");
    assert!(
        bad.is_empty(),
        "a varredura do Offset mudou — escreva a coluna nova em EXPECTED (e o número no commit):\n{}\n\n{report}",
        bad.join("\n")
    );
    assert_eq!(
        panics, EXPECTED_PANICS,
        "o número de pânicos engolidos mudou — o catch_unwind não pode esconder um pânico sem que \
         ele suba\n{report}"
    );
}

/// A varredura célula a célula (`--ignored --nocapture`): `junção lado d n diff% área`.
#[test]
#[ignore = "sonda: imprime as 600 células"]
fn prints_the_sweep_cell_by_cell() {
    let filter = std::env::var("SWEEP_JOIN").unwrap_or_default();
    let star = star();
    for (jn, j) in JOINS {
        if !filter.is_empty() && filter != jn {
            continue;
        }
        for (side, sign) in [("fora", 1.0), ("dentro", -1.0)] {
            for i in 1..=100 {
                let c = measure(&star, j, sign, i);
                println!(
                    "{jn}\t{side}\t{}\t{}\t{:.3}\t{:.5}",
                    c.d100, c.ours_n, c.diff_pct, c.truth_area
                );
            }
        }
    }
}

