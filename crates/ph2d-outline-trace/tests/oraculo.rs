//! The laws against the Godot oracle inputs (`docs/GameEngineFuncional/oraculo_contorno/`). Numbers: README.md.

use ph2d_outline_trace::{
    Mask, Outline, Params, Q4, convex_pieces, hertel_mehlhorn, trace, triangulate,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const CUTOFF: u8 = 128;
const DETAILS_Q4: [u16; 3] = [8, 32, 64]; // 0.5, 2, 4 px

fn docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/GameEngineFuncional")
}

fn load(path: PathBuf) -> (u32, u32, Vec<u8>) {
    let f = std::io::BufReader::new(std::fs::File::open(&path).expect("png"));
    let mut r = png::Decoder::new(f).read_info().expect("info");
    let mut buf = vec![0; r.output_buffer_size().expect("size")];
    let info = r.next_frame(&mut buf).expect("frame");
    assert_eq!(info.color_type, png::ColorType::Rgba, "{path:?}");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

fn cell(sheet: &(u32, u32, Vec<u8>), cx: u32, cy: u32) -> Mask {
    let mut px = Vec::with_capacity(100 * 100 * 4);
    for y in 0..100 {
        let row = ((cy * 100 + y) * sheet.0 + cx * 100) as usize * 4;
        px.extend_from_slice(&sheet.2[row..row + 400]);
    }
    Mask::from_rgba8(100, 100, &px, CUTOFF)
}

fn cases() -> Vec<(&'static str, Mask)> {
    let sheet = load(docs().join("arte/heroi_azul.png"));
    let run: Vec<Mask> = (0..8).map(|c| cell(&sheet, c, 1)).collect();
    let all: Vec<Mask> = (0..32).map(|i| cell(&sheet, i % 8, i / 8)).collect();
    let mut v = vec![
        ("heroi_f00_parado", cell(&sheet, 0, 0)),
        ("heroi_f11_corre", cell(&sheet, 3, 1)),
        ("heroi_f20_agachado", cell(&sheet, 4, 2)),
        (
            "heroi_uniao_corrida_8",
            Mask::union(&run).expect("same size"),
        ),
        (
            "heroi_uniao_folha_32",
            Mask::union(&all).expect("same size"),
        ),
    ];
    for n in ["sint_L", "sint_anel", "sint_crescente", "sint_moeda_ilha"] {
        let (w, h, px) = load(docs().join(format!("oraculo_contorno/entradas/{n}.png")));
        v.push((n, Mask::from_rgba8(w, h, &px, CUTOFF)));
    }
    v
}

/// (opaque outside, empty covered), pixel-centre test.
fn coverage(m: &Mask, o: &Outline) -> (usize, usize) {
    let (mut out, mut empty) = (0, 0);
    for y in 0..i64::from(m.h) {
        for x in 0..i64::from(m.w) {
            let c = [x as i32 * Q4 + Q4 / 2, y as i32 * Q4 + Q4 / 2];
            match (m.get(x, y), o.contains_q4(c)) {
                (true, false) => out += 1,
                (false, true) => empty += 1,
                _ => {}
            }
        }
    }
    (out, empty)
}

fn params(d: u16) -> Params {
    Params {
        detail_q4: d,
        min_island_px: 0,
    }
}

fn area2(p: &[[i32; 2]]) -> i64 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            i64::from(a[0]) * i64::from(b[1]) - i64::from(b[0]) * i64::from(a[1])
        })
        .sum()
}

fn convex(p: &[[i32; 2]]) -> bool {
    let n = p.len();
    (0..n).all(|i| {
        let (a, b, c) = (p[i], p[(i + 1) % n], p[(i + 2) % n]);
        i64::from(b[0] - a[0]) * i64::from(c[1] - a[1])
            - i64::from(b[1] - a[1]) * i64::from(c[0] - a[0])
            >= 0
    })
}

/// Bound measured 09/11 (README table): the worst empty-covered share over all cases × details.
const EMPTY_COVERED_MAX_PERMILLE: usize = 170; // outward law 09/11, worst: sint_anel at 4 px, 594/3496 = 17.0 %

#[test]
fn no_opaque_pixel_is_left_outside_and_empty_covered_is_bounded() {
    for (name, m) in cases() {
        for d in DETAILS_Q4 {
            let o = trace(&m, params(d));
            let (out, empty) = coverage(&m, &o);
            let pieces = convex_pieces(&o, d);
            println!(
                "{name} d={d} rings={} v={} pieces={} out={out} empty={empty} opaque={}",
                o.rings.len(),
                o.vertex_count(),
                pieces.len(),
                m.count()
            );
            assert_eq!(out, 0, "{name} detail {d}: art cut");
            assert!(
                empty * 1000 <= m.count() * EMPTY_COVERED_MAX_PERMILLE,
                "{name} d={d}: {empty}"
            );
        }
    }
}

#[test]
fn the_ring_keeps_its_hole() {
    let m = cases()
        .into_iter()
        .find(|c| c.0 == "sint_anel")
        .expect("anel")
        .1;
    for d in DETAILS_Q4 {
        let o = trace(&m, params(d));
        assert_eq!(
            o.rings.iter().filter(|r| area2(r) < 0).count(),
            1,
            "one hole"
        );
        let staircase = trace(&m, params(0));
        let hole = Outline {
            rings: staircase
                .rings
                .iter()
                .filter(|r| area2(r) < 0)
                .map(|r| r.iter().map(|p| [p[0], p[1]]).collect())
                .collect(),
        };
        let (mut hole_px, mut covered) = (0, 0);
        for y in 0..m.h as i32 {
            for x in 0..m.w as i32 {
                let c = [x * Q4 + Q4 / 2, y * Q4 + Q4 / 2];
                if hole.contains_q4(c) {
                    hole_px += 1;
                    covered += usize::from(o.contains_q4(c));
                }
            }
        }
        println!("anel d={d} hole_px={hole_px} covered={covered}");
        // The rim moves ≤ detail into the hole: 0 % at 0.5 px, 6.0 % at 2 px (README). Godot: 100 %.
        assert!(
            covered * 100 < hole_px * 15,
            "Godot covers the whole hole; we cover {covered}/{hole_px}"
        );
        for p in convex_pieces(&o, d) {
            let hole_centre = [48 * Q4, 48 * Q4];
            let inside = (0..p.len()).all(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                i64::from(b[0] - a[0]) * i64::from(hole_centre[1] - a[1])
                    - i64::from(b[1] - a[1]) * i64::from(hole_centre[0] - a[0])
                    > 0
            });
            assert!(!inside, "a piece covers the hole");
        }
    }
}

#[test]
fn piece_counts_l_and_coin() {
    let all = cases();
    let get = |n: &str| &all.iter().find(|c| c.0 == n).expect("case").1;
    let l = trace(get("sint_L"), params(32));
    assert_eq!(convex_pieces(&l, 32).len(), 2, "L");
    let coin = get("sint_moeda_ilha");
    let o = trace(coin, params(32));
    assert_eq!(o.rings.len(), 2, "coin + island");
    let exact = trace(coin, params(0));
    let island_px = exact
        .rings
        .iter()
        .map(|r| area2(r) / 2 / i64::from(Q4 * Q4))
        .min()
        .expect("ring");
    let dropped = trace(
        coin,
        Params {
            detail_q4: 32,
            min_island_px: island_px as u16 + 1,
        },
    );
    assert_eq!(dropped.rings.len(), 1, "island dropped");
    assert_eq!(
        convex_pieces(&dropped, 32).len(),
        1,
        "coin in one piece at detail 2"
    );
    let kept = trace(
        coin,
        Params {
            detail_q4: 32,
            min_island_px: island_px as u16,
        },
    );
    assert_eq!(kept.rings.len(), 2, "island kept");
}

#[test]
fn pieces_are_convex_and_tile_the_polygon() {
    for (name, m) in cases() {
        for d in DETAILS_Q4 {
            let o = trace(&m, params(d));
            let mut sum = 0;
            for (v, tris) in triangulate(&o) {
                for p in hertel_mehlhorn(&v, &tris) {
                    let pts: Vec<[i32; 2]> = p.iter().map(|i| v[*i as usize]).collect();
                    assert!(convex(&pts), "{name} d={d}: HM piece not convex");
                    sum += area2(&pts);
                }
            }
            assert_eq!(sum, o.area2(), "{name} d={d}: pieces do not tile");
            assert!(
                convex_pieces(&o, d).iter().all(|p| convex(p)),
                "{name} d={d}: merged piece not convex"
            );
        }
    }
}

#[test]
fn hertel_mehlhorn_matches_parry() {
    for (name, m) in cases() {
        for d in DETAILS_Q4 {
            for (v, tris) in triangulate(&trace(&m, params(d))) {
                let fv: Vec<parry2d::math::Vector> = v
                    .iter()
                    .map(|p| parry2d::math::Vector::new(p[0] as f32, p[1] as f32))
                    .collect();
                let norm = |mut ps: Vec<Vec<u32>>| {
                    for p in &mut ps {
                        let k = p.iter().enumerate().min_by_key(|x| *x.1).map_or(0, |x| x.0);
                        p.rotate_left(k);
                    }
                    ps.sort();
                    ps
                };
                let ours = norm(hertel_mehlhorn(&v, &tris));
                let theirs = norm(parry2d::transformation::hertel_mehlhorn_idx(&fv, &tris));
                assert_eq!(ours, theirs, "{name} d={d}");
            }
        }
    }
}

#[test]
fn hero_union_outline_is_byte_stable() {
    let m = cases()
        .into_iter()
        .find(|c| c.0 == "heroi_uniao_corrida_8")
        .expect("union")
        .1;
    let o = trace(&m, params(32));
    let mut h = Sha256::new();
    h.update(o.to_bytes());
    for p in convex_pieces(&o, 32) {
        for q in p {
            h.update(q[0].to_le_bytes());
            h.update(q[1].to_le_bytes());
        }
    }
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "4d870de44bc63d5c56ad1141c51220c771dc5a74aab2553da65b7b056a9ea0d6"
    );
}

#[test]
#[ignore = "timing: --profile smoke, run by name"]
fn timing_table() {
    let cs = cases();
    let mut best = vec![u128::MAX; cs.len() * DETAILS_Q4.len()];
    for _ in 0..15 {
        for (i, (_, m)) in cs.iter().enumerate() {
            for (k, d) in DETAILS_Q4.iter().enumerate() {
                let t = std::time::Instant::now();
                let o = trace(m, params(*d));
                std::hint::black_box(convex_pieces(&o, *d));
                let s = &mut best[i * DETAILS_Q4.len() + k];
                *s = (*s).min(t.elapsed().as_micros());
            }
        }
    }
    for (i, (n, _)) in cs.iter().enumerate() {
        println!("{n} us_min {:?}", &best[i * 3..i * 3 + 3]);
    }
}

#[test]
fn a_small_hole_is_never_merged_over() {
    let mut bits = vec![1u8; 20 * 20];
    for (y, x) in [(9, 9), (9, 10), (10, 9), (10, 10)] {
        bits[y * 20 + x] = 0;
    }
    let m = Mask { w: 20, h: 20, bits };
    let o = trace(&m, params(64));
    assert_eq!(o.rings.len(), 2);
    let centre = [10 * Q4, 10 * Q4];
    for p in convex_pieces(&o, 64) {
        let inside = (0..p.len()).all(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            i64::from(b[0] - a[0]) * i64::from(centre[1] - a[1])
                - i64::from(b[1] - a[1]) * i64::from(centre[0] - a[0])
                > 0
        });
        assert!(!inside, "a piece covers the hole: {p:?}");
    }
}
