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
    // Os 17-largos não cabem na imagem de 15: o Trial faz ali uma borda própria (coluna 0 intacta),
    // que não se reproduz — conferem-se em 40×40 (`round3_kernels_on_the_wide_and_low_contrast_inputs`).
    for k in KERNELS.iter().filter(|k| k.w <= 15 && k.h <= 15) {
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
                    // As rodadas 1–2 leram o ecrã (só RGB); o alfa é `the_color_under_alpha_…`.
                    assert_eq!(got[..3], [v; 3], "{} sobre {case} em ({x},{y})", k.name);
                }
            }
            checked += 1;
        }
    }
    assert!(
        checked >= 31 * 4,
        "cada kernel conferido em ≥ 4 entradas ({checked})"
    );
}

fn gray_grid(v: &Value) -> Vec<Vec<u8>> {
    v.as_array()
        .expect("linhas")
        .iter()
        .map(|r| {
            r.as_array()
                .expect("linha")
                .iter()
                .map(|n| n.as_u64().expect("u8") as u8)
                .collect()
        })
        .collect()
}

/// Rodada 3 (`oracle_trial_round3.json`): os 17-largos em 40×40 (A1), os que saturavam agora em
/// baixo contraste (A2) e o Tiled (A3), célula a célula sobre a entrada gravada na própria fixtura.
#[test]
fn round3_kernels_on_the_wide_and_low_contrast_inputs() {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_round3.json"))
        .expect("fixtura");
    let mut checked = 0;
    for sec in ["A1_conv_17wide", "A2_conv_low_contrast", "A3_conv_tiled"] {
        let cases = d[sec]["cases"].as_object().expect("casos");
        for (key, out) in cases {
            let mut parts = key.split(':');
            let (input, k) = (parts.next().expect("entrada"), parts.next().expect("k"));
            if k == "input" {
                continue;
            }
            let edge = if parts.next() == Some("tiled") {
                Edge::Wrap
            } else {
                Edge::Clamp
            };
            let index: usize = k.parse().expect("índice do kernel");
            let name = d_name(index);
            let kernel = KERNELS.iter().find(|q| q.name == name).expect(name);
            let src_rows = gray_grid(&cases[&format!("{input}:input")]);
            let (w, h) = (src_rows[0].len(), src_rows.len());
            let src = Source::from_rgba(
                w as u32,
                h as u32,
                src_rows.iter().flatten().map(|&v| [v, v, v, 255]).collect(),
            );
            let f = Convolution { kernel, edge };
            for (y, row) in gray_grid(out).iter().enumerate() {
                for (x, &v) in row.iter().enumerate() {
                    let got = f.at(&src, x as i32, y as i32);
                    assert_eq!(got[..3], [v; 3], "{name} sobre {key} em ({x},{y})");
                }
            }
            checked += 1;
        }
    }
    assert!(
        checked >= 6 * 4 + 6 * 3 + 2,
        "casos da rodada 3 ({checked})"
    );
}

/// O nome do kernel `i` na lista do Trial (a ordem da rodada 1).
fn d_name(i: usize) -> &'static str {
    const TRIAL: &str = include_str!("../fixtures/oracle_trial_convolution.json");
    let d: Value = serde_json::from_str(TRIAL).expect("fixtura");
    let n = d["kernels"][i]["name"].as_str().expect("nome").to_owned();
    KERNELS.iter().find(|k| k.name == n).map_or("?", |k| k.name)
}

/// A lista segue a ordem do Trial e só omite os medidos-e-recusados.
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
    // Só o alfa delas não é linear (dilata o alfa e deixa o RGB intacto, rodada 3 A4).
    let refused = [
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

/// O Trial 3b (`A4b`, estado dos botões lido por pixel) e 3 (`A4`: o «A_off» é o A clicado DEPOIS do
/// kernel, logo o inverso do que o kernel traz): RGBA célula a célula, o alfa sempre e o RGB onde o
/// alfa medido > 0 (o clipboard não determina o RGB sob alfa 0). O A do caso entra por `merge`.
#[test]
fn the_color_under_alpha_and_the_alpha_channel_match_the_trial_cell_by_cell() {
    use ph2d_pixel_core::filter::Channels;
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_round3.json"))
        .expect("fixtura");
    let trial = |k: &str| match k {
        "2" => "negative",
        "3" => "blur-3x3",
        "14" => "sharpen-3x3",
        "20" => "misc-contour",
        "23" => "misc-marmolize",
        "25" => "misc-rock-edges",
        _ => "",
    };
    let grid = |v: &Value| -> Vec<Vec<[u8; 4]>> {
        v.as_array()
            .expect("linhas")
            .iter()
            .map(|r| {
                r.as_array()
                    .expect("linha")
                    .iter()
                    .map(|c| std::array::from_fn(|i| c[i].as_u64().expect("u8") as u8))
                    .collect()
            })
            .collect()
    };
    let mut cases: Vec<(String, String, bool, Value, Value)> = Vec::new();
    let b = &d["A4b_conv_alpha"];
    for (key, out) in b["cases_rgba"].as_object().expect("casos") {
        let p: Vec<&str> = key.split(':').collect();
        if p[1] == "input" {
            continue;
        }
        let a = b["button_state"]["per_case"][key.as_str()]["at_ok"]["A"]
            .as_bool()
            .expect("A");
        let input = b["cases_rgba"][format!("{}:input", p[0])].clone();
        cases.push((key.clone(), trial(p[1]).into(), a, input, out.clone()));
    }
    let a4 = &d["A4_conv_alpha"]["cases"];
    for (key, out) in a4.as_object().expect("casos") {
        let p: Vec<&str> = key.split(':').collect();
        let name = trial(p[1]);
        if name.is_empty() {
            continue; // input e os outline (37, 38), fora da nossa lista
        }
        let a = (name == "blur-3x3") != (p.len() == 3);
        let input = a4[format!("{}:input", p[0])].clone();
        cases.push((format!("A4 {key}"), name.into(), a, input, out.clone()));
    }
    assert_eq!(cases.len(), 20, "16 de A4b + 4 de A4");
    let mut fora = Vec::new();
    for (key, name, a, input, out) in &cases {
        let inp = grid(input);
        let (h, w) = (inp.len(), inp[0].len());
        let src = Source::from_rgba(w as u32, h as u32, inp.concat());
        let f = Convolution {
            kernel: KERNELS.iter().find(|k| k.name == name).expect("kernel"),
            edge: Edge::Clamp,
        };
        let ch = Channels {
            r: true,
            g: true,
            b: true,
            a: *a,
        };
        for (y, row) in grid(out).iter().enumerate() {
            for (x, want) in row.iter().enumerate() {
                let got = ch.merge(inp[y][x], f.at(&src, x as i32, y as i32));
                let vis = if want[3] == 0 { 3..4 } else { 0..4 };
                if vis.into_iter().any(|c| got[c] != want[c]) {
                    fora.push(format!("{key} ({x},{y}) Trial {want:?} nós {got:?}"));
                }
            }
        }
    }
    assert!(
        fora.is_empty(),
        "{} células fora: {:#?}",
        fora.len(),
        &fora[..fora.len().min(12)]
    );
}

/// Os botões que escolher cada kernel deixa (`A5_kernel_channels`), um por um.
#[test]
fn every_kernel_carries_the_trials_channel_buttons() {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_round3.json"))
        .expect("fixtura");
    let per = d["A5_kernel_channels"]["per_kernel"]
        .as_object()
        .expect("A5");
    for k in KERNELS {
        let s = per
            .iter()
            .find(|(n, _)| n.split_once(' ').map(|p| p.1) == Some(k.name))
            .and_then(|(_, v)| v.as_str())
            .unwrap_or_else(|| panic!("{} medido", k.name));
        let want: [bool; 4] = std::array::from_fn(|i| s.contains(['R', 'G', 'B', 'A'][i]));
        assert_eq!(k.channels, want, "{}", k.name);
    }
}

/// Só o centro opaco e o divisor = a soma dos pesos fora dele (todos transparentes): `div − Σ w`
/// dá 0 e a cor divide por 1. Fixa a ESCOLHA do «1» — o Trial não a mediu (handoff P8b §3).
#[test]
fn a_color_divisor_that_drops_to_zero_divides_by_one() {
    static K: ph2d_pixel_core::filter::convolution::Kernel =
        ph2d_pixel_core::filter::convolution::Kernel {
            name: "teste-centro-so",
            w: 3,
            h: 3,
            cx: 1,
            cy: 1,
            div: 4,
            bias: 0,
            channels: [true; 4],
            weights: &[0, 1, 0, 1, 4, 1, 0, 1, 0],
        };
    let rgba = (0..9)
        .map(|i| if i == 4 { [10, 20, 30, 255] } else { [0; 4] })
        .collect();
    let src = Source::from_rgba(3, 3, rgba);
    let f = Convolution {
        kernel: &K,
        edge: Edge::Clamp,
    };
    assert_eq!(f.at(&src, 1, 1), [40, 80, 120, 255]);
}
