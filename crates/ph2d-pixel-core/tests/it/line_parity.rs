//! A linha canonicalizada contra o MVP web (fixtura extraída por
//! `docs/PixelArt/ferramentas/extrai_linhas_mvp.mjs`): mesmas células, mesma ordem.

use ph2d_pixel_core::raster::line_cells;

type Case = ((i32, i32), (i32, i32), Vec<(i32, i32)>);

fn cells(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    line_cells(x0, y0, x1, y1, |x, y| v.push((x, y)));
    v
}

fn fixture() -> Vec<Case> {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/mvp_line_cells.json")).expect("json");
    let pt = |v: &serde_json::Value| {
        (
            v[0].as_i64().expect("x") as i32,
            v[1].as_i64().expect("y") as i32,
        )
    };
    raw["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .map(|c| {
            let cells = c["cells"]
                .as_array()
                .expect("cells")
                .iter()
                .map(pt)
                .collect();
            (pt(&c["from"]), pt(&c["to"]), cells)
        })
        .collect()
}

#[test]
fn the_line_matches_the_mvp_cell_for_cell() {
    let cases = fixture();
    assert!(cases.len() >= 18, "a fixtura encolheu: {}", cases.len());
    for ((x0, y0), (x1, y1), want) in cases {
        assert_eq!(cells(x0, y0, x1, y1), want, "({x0},{y0})→({x1},{y1})");
    }
}

#[test]
fn a_to_b_and_b_to_a_give_the_same_cells() {
    for ((x0, y0), (x1, y1), _) in fixture() {
        assert_eq!(
            cells(x0, y0, x1, y1),
            cells(x1, y1, x0, y0),
            "({x0},{y0})↔({x1},{y1})"
        );
    }
}

#[test]
fn the_line_has_both_ends_and_no_holes() {
    for ((x0, y0), (x1, y1), _) in fixture() {
        let v = cells(x0, y0, x1, y1);
        assert!(v.contains(&(x0, y0)) && v.contains(&(x1, y1)));
        for w in v.windows(2) {
            assert!(
                (w[0].0 - w[1].0).abs() <= 1 && (w[0].1 - w[1].1).abs() <= 1,
                "{w:?}"
            );
        }
    }
}
