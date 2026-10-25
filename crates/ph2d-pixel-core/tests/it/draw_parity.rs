//! **P1 contra o MVP web** — a fixtura `mvp_draw.json` (`docs/PixelArt/ferramentas/extrai_desenho_mvp.mjs`):
//! pixel-perfect (lote e ao vivo), curva, retângulo, elipse, carimbos, balde, simetria, o funil e o
//! splitmix32. Cada caso compara a MESMA resposta; a ordem conta onde ela é o produto (caminhos).

use std::collections::BTreeSet;

use ph2d_pixel_core::brush::{Brush, BrushShape, SplitMix32};
use ph2d_pixel_core::fill::flood;
use ph2d_pixel_core::raster::{
    Cell, PixelPerfect, curve_path, ellipse_cells, ellipse_filled_cells, pixel_perfect, rect_cells,
};
use ph2d_pixel_core::symmetry::{Mirrors, Symmetry, Tiling, expand, symmetry_images};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_draw.json")).expect("json")
}

fn i(v: &Value) -> i32 {
    v.as_i64().expect("int") as i32
}

fn cell(v: &Value) -> Cell {
    (i(&v[0]), i(&v[1]))
}

fn cells(v: &Value) -> Vec<Cell> {
    v.as_array().expect("cells").iter().map(cell).collect()
}

fn list<'a>(f: &'a Value, k: &str, min: usize) -> &'a Vec<Value> {
    let a = f[k].as_array().expect(k);
    assert!(a.len() >= min, "a fixtura `{k}` encolheu: {}", a.len());
    a
}

fn collect(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> Vec<Cell> {
    let mut out = Vec::new();
    f(&mut |x, y| out.push((x, y)));
    out
}

#[test]
fn pixel_perfect_matches_the_mvp_in_batch_and_live() {
    let f = fx();
    for c in list(&f, "pixel_perfect", 9) {
        let path = cells(&c["path"]);
        assert_eq!(pixel_perfect(&path), cells(&c["batch"]), "lote {path:?}");
        let mut pp = PixelPerfect::default();
        let mut got = Vec::new();
        for &(x, y) in &path {
            let s = pp.push(x, y);
            if let Some(p) = s.paint {
                got.push(('+', p));
            }
            if let Some(r) = s.retract {
                got.push(('-', r));
            }
        }
        let want: Vec<(char, Cell)> = c["live"]
            .as_array()
            .expect("live")
            .iter()
            .map(|e| {
                let k = e[0].as_str().expect("op").chars().next().expect("op");
                (k, (i(&e[1]), i(&e[2])))
            })
            .collect();
        assert_eq!(got, want, "ao vivo {path:?}");
    }
}

#[test]
fn the_curve_matches_the_mvp_cell_for_cell() {
    let f = fx();
    for c in list(&f, "curve", 10) {
        let a: Vec<i32> = c["args"].as_array().expect("args").iter().map(i).collect();
        assert_eq!(
            curve_path(a[0], a[1], a[2], a[3], a[4], a[5]),
            cells(&c["cells"]),
            "curva {a:?}"
        );
    }
}

#[test]
fn rectangles_and_ellipses_match_the_mvp() {
    let f = fx();
    for c in list(&f, "rect", 16) {
        let b: Vec<i32> = c["box"].as_array().expect("box").iter().map(i).collect();
        let r = |filled| collect(|cb| rect_cells(b[0], b[1], b[2], b[3], filled, cb));
        assert_eq!(r(false), cells(&c["outline"]), "retângulo {b:?}");
        assert_eq!(r(true), cells(&c["filled"]), "retângulo cheio {b:?}");
    }
    for c in list(&f, "ellipse", 16) {
        let b: Vec<i32> = c["box"].as_array().expect("box").iter().map(i).collect();
        let o = collect(|cb| ellipse_cells(b[0], b[1], b[2], b[3], cb));
        let fl = collect(|cb| ellipse_filled_cells(b[0], b[1], b[2], b[3], cb));
        assert_eq!(o, cells(&c["outline"]), "elipse {b:?}");
        assert_eq!(fl, cells(&c["filled"]), "elipse cheia {b:?}");
    }
}

#[test]
fn brush_stamps_match_the_authored_table() {
    let f = fx();
    for c in list(&f, "stamps", 32) {
        let shape = match c["shape"].as_str().expect("shape") {
            "circle" => BrushShape::Circle,
            _ => BrushShape::Square,
        };
        let d = c["d"].as_u64().expect("d") as u32;
        let flat: Vec<i32> = c["offsets"]
            .as_array()
            .expect("offs")
            .iter()
            .map(i)
            .collect();
        let want: Vec<Cell> = flat.chunks_exact(2).map(|p| (p[0], p[1])).collect();
        let b = Brush {
            shape,
            size: d,
            angle: 0,
        };
        assert_eq!(b.offsets(), want, "{shape:?} {d}");
    }
}

#[test]
fn the_bucket_region_matches_the_mvp() {
    let f = fx();
    for c in list(&f, "fill", 24) {
        let art: Vec<&str> = c["art"]
            .as_array()
            .expect("art")
            .iter()
            .map(|r| r.as_str().expect("row"))
            .collect();
        let (w, h) = (art[0].len() as u32, art.len() as u32);
        let get = |x: i32, y: i32| art[y as usize].as_bytes()[x as usize];
        let seed = cell(&c["seed"]);
        let inside = seed.0 >= 0 && seed.1 >= 0 && seed.0 < w as i32 && seed.1 < h as i32;
        let target = if inside { get(seed.0, seed.1) } else { 0 };
        let mut got = BTreeSet::new();
        flood(
            w,
            h,
            seed,
            false,
            |x, y| get(x, y) == target,
            |x, y| {
                assert!(got.insert((x, y)), "({x},{y}) duas vezes");
            },
        );
        let want: BTreeSet<Cell> = cells(&c["contiguous"]).into_iter().collect();
        assert_eq!(got, want, "balde {art:?} em {seed:?}");
    }
}

fn mirrors(mode: &str) -> Mirrors {
    Mirrors {
        h: mode.contains('x'),
        v: mode.contains('y'),
        ..Mirrors::default()
    }
}

#[test]
fn symmetry_images_match_the_mvp() {
    let f = fx();
    for c in list(&f, "symmetry", 80) {
        let (w, h) = cell(&c["size"]);
        let s = Symmetry::centered(
            mirrors(c["mode"].as_str().expect("mode")),
            w as u32,
            h as u32,
        );
        let want: Vec<(Cell, bool, bool)> = c["images"]
            .as_array()
            .expect("images")
            .iter()
            .map(|e| {
                (
                    (i(&e[0]), i(&e[1])),
                    e[2].as_bool().expect("fx"),
                    e[3].as_bool().expect("fy"),
                )
            })
            .collect();
        assert_eq!(symmetry_images(s, cell(&c["cell"])), want, "{c}");
    }
}

#[test]
fn the_funnel_mirrors_the_stamp_and_wraps_like_the_mvp() {
    let f = fx();
    for c in list(&f, "funil", 48) {
        let (w, h) = cell(&c["size"]);
        let flat: Vec<i32> = c["offsets"]
            .as_array()
            .expect("offs")
            .iter()
            .map(i)
            .collect();
        let offs: Vec<Cell> = flat.chunks_exact(2).map(|p| (p[0], p[1])).collect();
        let s = Symmetry::centered(
            mirrors(c["mode"].as_str().expect("mode")),
            w as u32,
            h as u32,
        );
        let wrap = c["tiling"].as_bool().expect("tiling");
        let mut got = Vec::new();
        expand(
            cell(&c["cell"]),
            &offs,
            s,
            Tiling { x: wrap, y: wrap },
            (w as u32, h as u32),
            |x, y, k| got.push((x, y, k as i32)),
        );
        let want: Vec<(i32, i32, i32)> = c["out"]
            .as_array()
            .expect("out")
            .iter()
            .map(|e| (i(&e[0]), i(&e[1]), i(&e[2])))
            .collect();
        assert_eq!(got, want, "{c}");
    }
}

#[test]
fn splitmix32_matches_the_mvp_stream() {
    let f = fx();
    for c in list(&f, "rng", 5) {
        let seed = c["seed"].as_i64().expect("seed") as i32 as u32;
        let mut r = SplitMix32::new(seed);
        let got: Vec<u64> = (0..8).map(|_| u64::from(r.next_u32())).collect();
        let want: Vec<u64> = c["first"]
            .as_array()
            .expect("first")
            .iter()
            .map(|v| v.as_u64().expect("u32"))
            .collect();
        assert_eq!(got, want, "semente {seed}");
    }
}
