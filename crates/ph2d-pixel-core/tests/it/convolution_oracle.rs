//! **Convolution Matrix contra o Aseprite Trial** (`oracle_trial_convolution.json`): cada kernel
//! da lista sobre as CINCO entradas medidas (r1 `imp`/`edge` pretas, r2 `impg`/`imp0`/`stepg`
//! cinzentas), célula a célula. As matrizes nasceram de `fit_convolucao.py`; este gate as prende.

use ph2d_pixel_core::filter::convolution::{Convolution, KERNELS};
use ph2d_pixel_core::filter::{Edge, Filter, Source};
use serde_json::Value;

const N: usize = 15;

fn gray_rows(v: &Value) -> Vec<Vec<u8>> {
    v.as_array()
        .expect("linhas")
        .iter()
        .map(|r| {
            r.as_str()
                .expect("linha")
                .split_whitespace()
                .map(|n| n.parse().expect("u8"))
                .collect()
        })
        .collect()
}

/// As entradas NOSSAS que o Trial filtrou (o `method` da fixtura).
fn input(name: &str) -> Source {
    let g = |x: usize, y: usize| -> u8 {
        match name {
            "imp" => u8::from((x, y) == (7, 7) || (x, y) == (0, 0)) * 255,
            "edge" => u8::from(x < 7) * 255,
            "impg" => {
                if (x, y) == (7, 7) {
                    255
                } else {
                    128
                }
            }
            "imp0" => {
                if (x, y) == (7, 7) {
                    0
                } else {
                    128
                }
            }
            "stepg" => {
                if x < 7 {
                    64
                } else {
                    192
                }
            }
            _ => unreachable!("entrada {name}"),
        }
    };
    let rgba = (0..N * N).map(|i| {
        let v = g(i % N, i / N);
        [v, v, v, 255]
    });
    Source::from_rgba(N as u32, N as u32, rgba.collect())
}

#[test]
fn every_listed_kernel_gives_the_trials_cells_on_all_five_inputs() {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_convolution.json"))
        .expect("fixtura");
    let find = |list: &Value, name: &str| -> Option<Value> {
        list.as_array()?.iter().find(|k| k["name"] == name).cloned()
    };
    let mut checked = 0;
    for k in KERNELS {
        let r1 = find(&d["kernels"], k.name).expect("kernel medido na rodada 1");
        let r2 = find(&d["round2_gray"]["kernels"], k.name).expect("kernel medido na rodada 2");
        let f = Convolution {
            kernel: k,
            edge: Edge::Clamp,
        };
        for (case, rows) in [
            ("imp", &r1["imp"]),
            ("edge", &r1["edge"]),
            ("impg", &r2["impg"]),
            ("imp0", &r2["imp0"]),
            ("stepg", &r2["stepg"]),
        ] {
            if rows.is_null() {
                continue;
            }
            let want = gray_rows(rows);
            let src = input(case);
            for (y, row) in want.iter().enumerate() {
                for (x, &v) in row.iter().enumerate() {
                    let got = f.at(&src, x as i32, y as i32);
                    assert_eq!(got, [v, v, v, 255], "{} sobre {case} em ({x},{y})", k.name);
                }
            }
            checked += 1;
        }
    }
    assert!(
        checked >= KERNELS.len() * 4,
        "cada kernel conferido em ≥ 4 entradas ({checked})"
    );
}

/// A lista segue a ordem do Trial e só omite os medidos-e-recusados (`law_measured_round2`).
#[test]
fn the_kernel_list_is_the_trials_order_minus_the_refused() {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_convolution.json"))
        .expect("fixtura");
    let trial: Vec<&str> = d["kernels"]
        .as_array()
        .expect("lista")
        .iter()
        .map(|k| k["name"].as_str().expect("nome"))
        .collect();
    let ours: Vec<&str> = KERNELS.iter().map(|k| k.name).collect();
    let kept: Vec<&str> = trial.iter().copied().filter(|n| ours.contains(n)).collect();
    assert_eq!(ours, kept, "a ordem da lista é a do Trial");
    let refused = [
        "blur-17x17",
        "blur-17x3-left",
        "blur-3x17-top",
        "edges-find",
        "edges-find-horizontal",
        "edges-find-vertical",
        "misc-texturize",
        "misc-emboss",
        "misc-rock",
        "drunk-17x17_x",
        "drunk-17x17_+",
        "drunk-17x17_o",
        "outline-transparent-layer-(1)",
        "outline-transparent-layer-(2)",
    ];
    let missing: Vec<&str> = trial
        .iter()
        .copied()
        .filter(|n| !ours.contains(n))
        .collect();
    assert_eq!(missing, refused, "só faltam os recusados");
    for k in KERNELS {
        assert_eq!(k.weights.len(), (k.w * k.h) as usize, "{}", k.name);
        assert!(k.cx < k.w && k.cy < k.h, "{}", k.name);
    }
}

/// *Tiled*: a janela dá a volta à imagem em vez de replicar a borda.
#[test]
fn tiled_wraps_where_clamp_replicates() {
    let k = ph2d_pixel_core::filter::convolution::kernel("blur-3x3").expect("blur-3x3");
    let src = input("edge");
    let clamp = Convolution {
        kernel: k,
        edge: Edge::Clamp,
    }
    .at(&src, 0, 7);
    let wrap = Convolution {
        kernel: k,
        edge: Edge::Wrap,
    }
    .at(&src, 0, 7);
    assert_eq!(clamp[0], 255, "replicada: só branco à esquerda");
    assert_eq!(wrap[0], 191, "a volta traz a coluna 14, preta: ⌊255·12/16⌋");
}
