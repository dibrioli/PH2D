//! ⭐ O RASCUNHO contra o EXCALIDRAW, número a número: o Excalidraw 0.18.1 corrido sobre entradas
//! nossas com SEMENTES fixas (`docs/MiroClone/ferramentas/excalidraw_oracle/saidas/formas_*` e
//! `rascunho_*`, 07/10); o `d` de cada traço do SVG dele (duas casas) tem de sair das nossas opções
//! ([`hand_options`], [`hand_roughness`]) e primitivas ([`hand_shape`], [`hand_line`]).
//! O ajuste que achou estas leis está em `ferramentas/excalidraw_oracle/ajuste/`.

use super::*;
use ph2d_board_model::{Rgba, Shape};
use ph2d_board_rough::rough::Op;
use serde_json::Value;

macro_rules! saida {
    ($n:literal) => {
        (
            $n,
            include_str!(concat!(
                "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/",
                $n,
                ".restored.json"
            )),
            include_str!(concat!(
                "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/",
                $n,
                ".svg"
            )),
        )
    };
}

/// Os números de cada `<path d>` de cada `<g stroke-linecap="round">` do SVG, por elemento.
fn svg_groups(svg: &str) -> Vec<Vec<Vec<f64>>> {
    svg.split("<g stroke-linecap=\"round\"")
        .skip(1)
        .map(|g| {
            let g = &g[..g.find("</g>").unwrap_or(g.len())];
            g.split("d=\"")
                .skip(1)
                .map(|d| {
                    d[..d.find('"').expect("d fecha")]
                        .replace(['M', 'C', 'L', ','], " ")
                        .split_whitespace()
                        .map(|t| t.parse::<f64>().expect("número"))
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn numbers(set: &OpSet) -> Vec<f64> {
    set.ops
        .iter()
        .flat_map(|op| match *op {
            Op::Move(p) | Op::Line(p) => p.to_vec(),
            Op::Cubic(a, b, c) => vec![a[0], a[1], b[0], b[1], c[0], c[1]],
        })
        .collect()
}

fn same(name: &str, got: &[OpSet], want: &[Vec<f64>]) {
    assert_eq!(got.len(), want.len(), "{name}: número de traços");
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        let g = numbers(g);
        assert_eq!(g.len(), w.len(), "{name}: traço {k}, número de valores");
        for (i, (a, b)) in g.iter().zip(w).enumerate() {
            // O SVG tem duas casas: o nosso valor arredondado tem de ser o dele.
            assert!(
                (a - b).abs() <= 0.005 + 1e-9,
                "{name}: traço {k} valor {i}: {a} ≠ Excalidraw {b}"
            );
        }
    }
}

fn style(e: &Value) -> Style {
    let ink = Rgba([30, 30, 30, 255]);
    let fill = (e["backgroundColor"] != "transparent").then_some(Rgba([165, 216, 255, 255]));
    let mut s = Style::new(fill, Some(ink), ink);
    s.stroke_width = e["strokeWidth"].as_f64().expect("espessura");
    s
}

const SHAPES: [(&str, &str, &str); 10] = [
    saida!("formas_retangulo"),
    saida!("formas_elipse"),
    saida!("formas_losango"),
    saida!("rascunho_tamanhos"),
    saida!("rascunho_limiar"),
    saida!("rascunho_limiar2"),
    saida!("rascunho_limiar3"),
    saida!("rascunho_espessuras"),
    saida!("rascunho_cantos"),
    saida!("rascunho_ramos"),
];

/// ⭐ As formas: o tremor do tamanho, os vértices presos, a elipse, o rectângulo de cantos redondos
/// — o MESMO traço que o Excalidraw desenha com a mesma semente.
#[test]
fn every_sketched_shape_is_the_one_excalidraw_draws_with_the_same_seed() {
    let mut checked = 0;
    for (name, restored, svg) in SHAPES {
        let els: Vec<Value> = serde_json::from_str(restored).expect("elementos");
        let groups = svg_groups(svg);
        for (e, want) in els.iter().zip(&groups) {
            let (w, h) = (e["width"].as_f64().unwrap(), e["height"].as_f64().unwrap());
            let round = !e["roundness"].is_null();
            let kind = match e["type"].as_str().unwrap() {
                "rectangle" => ShapeType::Rectangle,
                "ellipse" => ShapeType::Ellipse,
                "diamond" if !round => ShapeType::Diamond,
                _ => continue,
            };
            // O produto só desenha o «artista» cheio.
            if e["roughness"] != 1 || e["fillStyle"] != "solid" {
                continue;
            }
            let dash = match e["strokeStyle"].as_str() {
                Some("dashed") => ph2d_board_model::Dash::Dashed,
                Some("dotted") => ph2d_board_model::Dash::Dotted,
                _ => ph2d_board_model::Dash::Solid,
            };
            let st = Style {
                round,
                dash,
                ..style(e)
            };
            let seed = e["seed"].as_u64().unwrap() as u32;
            // As DUAS passagens do Excalidraw (a terceira do quadro é nossa).
            let opts = rough::Options {
                passes: 2,
                ..hand_options(seed, hand_roughness(w, h, round, false), &st, true)
            };
            let got = if kind == ShapeType::Diamond {
                // O losango do Excalidraw tem os vértices a `⌊w/2⌋+1` (o dele, não lei: o nosso é ao
                // meio) — aqui confere-se a LEI das opções sobre os vértices dele.
                let (tx, ry) = ((w / 2.0).floor() + 1.0, (h / 2.0).floor() + 1.0);
                rough::polygon(&[[tx, 0.0], [w, ry], [tx, h], [0.0, ry]], &opts)
            } else {
                let shape = Shape {
                    kind,
                    style: st.clone(),
                    text: Default::default(),
                };
                let o = ph2d_board_geom::outline(&shape, w, h);
                hand_shape(kind, round, w, h, &o, &opts)
            };
            same(&format!("{name} {kind:?} {w}×{h}"), &got, want);
            checked += 1;
        }
    }
    assert!(
        checked >= 50,
        "a cobertura do oráculo não encolhe: {checked}"
    );
}

/// ⭐ As setas: a de cantos vivos é segmentos com os vértices presos, a arredondada é a curva pelos
/// pontos; o tremor é o de uma LINHA (inteiro a partir de 50, metade numa seta de 40).
#[test]
fn sketched_arrows_are_the_lines_excalidraw_draws_with_the_same_seed() {
    let mut checked = 0;
    for (_, restored, svg) in [saida!("rascunho_setas"), saida!("rascunho_ramos")] {
        let els: Vec<Value> = serde_json::from_str(restored).expect("elementos");
        let groups = svg_groups(svg);
        for (e, want) in els.iter().zip(&groups) {
            if !matches!(e["type"].as_str(), Some("arrow" | "line")) {
                continue;
            }
            let pts: Vec<[f64; 2]> = e["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                .collect();
            let seed = e["seed"].as_u64().unwrap() as u32;
            let (w, h) = (e["width"].as_f64().unwrap(), e["height"].as_f64().unwrap());
            let opts = rough::Options {
                passes: 2,
                ..hand_options(seed, hand_roughness(w, h, false, true), &style(e), false)
            };
            let got = if e["roundness"].is_null() {
                let mut p = BezPath::new();
                p.move_to((pts[0][0], pts[0][1]));
                for q in &pts[1..] {
                    p.line_to((q[0], q[1]));
                }
                hand_line(&p, &opts)
            } else {
                rough::curve(&pts, &opts)
            };
            same(
                &format!("seta {} pontos {w}×{h}", pts.len()),
                &got,
                &want[..1],
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "as setas do oráculo: {checked}");
}

/// O quadro desenha cada traço com TRÊS passagens (ordem do dono, 07/10) — nunca as duas do
/// Excalidraw sozinhas.
#[test]
fn the_board_draws_every_line_three_times() {
    let ink = Rgba([30, 30, 30, 255]);
    let st = Style::new(None, Some(ink), ink);
    assert_eq!(hand_options(1, ROUGHNESS, &st, false).passes, 3);
    let sets = hand_shape(
        ShapeType::Rectangle,
        false,
        160.0,
        120.0,
        &ph2d_board_geom::outline(
            &Shape {
                kind: ShapeType::Rectangle,
                style: st.clone(),
                text: Default::default(),
            },
            160.0,
            120.0,
        ),
        &hand_options(1, ROUGHNESS, &st, false),
    );
    let moves = sets[0]
        .ops
        .iter()
        .filter(|o| matches!(o, Op::Move(_)))
        .count();
    assert_eq!(moves, 4 * 3, "quatro arestas, três passagens cada");
}
