//! **A selecção contra o Aseprite Trial observado** — a fixtura `oracle_trial_selection.json`
//! (`docs/PixelArt/ferramentas/trial_ase_selecao.py` escreve os `.ase`, `trial_selecao.py` arrasta e
//! lê a máscara pelo Edit ▸ Fill magenta, `trial_fixture_selecao.py` grava). Cada caso refaz-se pelo
//! NOSSO núcleo e tem de dar a máscara célula a célula; cada família conta os seus casos (um salto
//! mudo reprova).
//!
//! ⛔ Recusas MEDIDAS: a marquee elíptica como Zingl dá 106/112 — `4x6 5x6 6x5 10x6 10x8 24x16` são
//! mais apertadas no Trial; nenhuma das variantes medidas (raio/centro em meia célula, limiar,
//! arredondamento por linha, a caixa ±1, transposta, linhas ∩ colunas) passa das 106. As seis
//! ficam listadas abaixo: uma lei melhor aparece como «menos falhas» e reprova até a lista encolher.
//! `mode_btn_add` é uma leitura partida do driver (máscara vazia); mede-se pelo `mode_shift`.

use ph2d_pixel_core::fill::BucketSample;
use ph2d_pixel_core::selection::{
    self, Mask, Modify, ModifyBrush, SelectMode, Wand, color_range, wand,
};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, Sprite, apply};
use serde_json::Value;

pub(crate) fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_selection.json")).expect("json")
}

pub(crate) fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect()
}

pub(crate) fn cell(v: &Value) -> (i32, i32) {
    (
        v[0].as_i64().expect("x") as i32,
        v[1].as_i64().expect("y") as i32,
    )
}

/// Um sprite RGBA com as linhas `rows` (hex RGBA por célula).
pub(crate) fn rgba_sprite(rows: &[Value]) -> Sprite {
    let bytes: Vec<u8> = rows
        .iter()
        .flat_map(|r| unhex(r.as_str().expect("row")))
        .collect();
    let h = rows.len() as u32;
    let w = bytes.len() as u32 / 4 / h;
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = Rect::new(0, 0, w, h);
    apply(
        &mut s,
        Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(r),
            rect: r,
            pixels: bytes,
        }),
    )
    .expect("patch");
    s
}

fn rows(m: &Mask) -> Vec<String> {
    (0..m.height() as i32)
        .map(|y| {
            (0..m.width() as i32)
                .map(|x| if m.get(x, y) { '1' } else { '0' })
                .collect()
        })
        .collect()
}

fn want(c: &Value) -> Vec<String> {
    c["mask"]
        .as_array()
        .expect("mask")
        .iter()
        .map(|r| r.as_str().expect("r").to_owned())
        .collect()
}

/// Os casos cujo nome começa por `prefix`, em ordem.
pub(crate) fn cases<'a>(f: &'a Value, prefix: &str) -> Vec<(&'a String, &'a Value)> {
    f["cases"]
        .as_object()
        .expect("cases")
        .iter()
        .filter(|(k, _)| k.starts_with(prefix))
        .collect()
}

const W: u32 = 24;
const H: u32 = 16;

fn shape(desc: &Value) -> Mask {
    let mut m = Mask::empty(W, H);
    for part in desc.as_array().expect("shape") {
        let (a, b) = (cell(&part[1]), cell(&part[2]));
        let s = match part[0].as_str().expect("kind") {
            "ellipse" => Mask::ellipse(W, H, a, b),
            _ => Mask::marquee(W, H, a, b),
        };
        m = m.combine(&s, SelectMode::Add);
    }
    m
}

fn mode_of(mods: &Value) -> SelectMode {
    let has = |k: &str| {
        mods.as_array()
            .is_some_and(|a| a.iter().any(|m| m.as_str() == Some(k)))
    };
    match (has("shift"), has("alt"), has("ctrl")) {
        (true, false, true) => SelectMode::Intersect,
        (_, true, _) => SelectMode::Subtract,
        (true, false, false) => SelectMode::Add,
        _ => SelectMode::Replace,
    }
}

#[test]
fn the_rect_marquee_is_inclusive_press_to_release_like_the_trial() {
    let f = fx();
    let cs = cases(&f, "rect_");
    for (k, c) in &cs {
        let m = Mask::marquee(W, H, cell(&c["press"]), cell(&c["release"]));
        assert_eq!(rows(&m), want(c), "{k}");
    }
    assert_eq!(cs.len(), 10);
}

#[test]
fn the_ellipse_marquee_is_zingl_except_the_six_measured_boxes() {
    let f = fx();
    let cs = cases(&f, "ell_");
    let mut off = Vec::new();
    for (k, c) in &cs {
        let m = Mask::ellipse(W, H, cell(&c["press"]), cell(&c["release"]));
        if rows(&m) != want(c) {
            off.push(k.as_str());
        }
    }
    assert_eq!(cs.len(), 112);
    off.sort_unstable();
    assert_eq!(
        off,
        [
            "ell_10x6",
            "ell_10x8",
            "ell_4x6",
            "ell_5x6",
            "ell_6x5",
            "ell_from_0_0__23_15"
        ],
        "a lacuna medida da elipse mudou"
    );
}

/// As amostras do rato do `trial_selecao.py`: 16 px por célula, centro em +8, passos de ~4 px
/// (no mínimo 6) por segmento e os três acertos `-2, +1, 0` no último ponto.
fn mouse_samples(pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let sp: Vec<(i32, i32)> = pts.iter().map(|&(x, y)| (16 * x + 8, 16 * y + 8)).collect();
    let mut out = vec![sp[0]];
    for w in sp.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = f64::from(b.0 - a.0).hypot(f64::from(b.1 - a.1));
        let n = 6.max((len / 4.0) as i32);
        for s in 1..=n {
            out.push((
                a.0 + ((b.0 - a.0) * s).div_euclid(n),
                a.1 + ((b.1 - a.1) * s).div_euclid(n),
            ));
        }
    }
    let b = sp[sp.len() - 1];
    for d in [-2, 1, 0] {
        out.push((b.0 + d, b.1 + d));
    }
    out.into_iter()
        .map(|(x, y)| (x.div_euclid(16), y.div_euclid(16)))
        .collect()
}

#[test]
fn the_polygonal_and_freehand_lasso_are_even_odd_plus_the_directed_outline() {
    let f = fx();
    let mut n = 0;
    for prefix in ["plasso_", "lasso_"] {
        for (k, c) in cases(&f, prefix) {
            let pts: Vec<(i32, i32)> = c["vertices"]
                .as_array()
                .expect("v")
                .iter()
                .map(cell)
                .collect();
            let m = if prefix == "plasso_" {
                Mask::polygon(W, H, &pts)
            } else {
                Mask::lasso(W, H, &mouse_samples(&pts))
            };
            assert_eq!(rows(&m), want(c), "{k}");
            n += 1;
        }
    }
    assert_eq!(n, 12);
}

#[test]
fn the_selection_modes_and_the_move_tool_match_the_trial() {
    let f = fx();
    let cs = cases(&f, "mode_");
    for (k, c) in &cs {
        let first = Mask::marquee(W, H, cell(&c["first"][0]), cell(&c["first"][1]));
        let (a, b) = (cell(&c["second"][0]), cell(&c["second"][1]));
        let mods = &c["modifiers"];
        let only_ctrl = mods
            .as_array()
            .is_some_and(|m| m.len() == 1 && m[0] == "ctrl");
        let m = if only_ctrl {
            // Ctrl sozinho é a ferramenta Mover: a selecção anda o que o arrasto andou.
            first.offset(b.0 - a.0, b.1 - a.1)
        } else {
            let mode = match c["button"].as_str() {
                Some("add") => SelectMode::Add,
                Some("subtract") => SelectMode::Subtract,
                Some("intersect") => SelectMode::Intersect,
                _ => mode_of(mods),
            };
            let second = if c["op"] == "rect_then_ellipse" {
                Mask::ellipse(W, H, a, b)
            } else {
                Mask::marquee(W, H, a, b)
            };
            first.combine(&second, mode)
        };
        if k.as_str() == "mode_btn_add" {
            // ⛔ Leitura partida do driver: o Trial devolveu máscara VAZIA (somar nunca encolhe a
            // primeira). A lei «botão = modificador» mede-se pelo `mode_shift`.
            assert!(
                want(c).iter().all(|r| !r.contains('1')),
                "{k} deixou de ser vazio"
            );
            assert_eq!(rows(&m), want(&f["cases"]["mode_shift"]), "{k}");
            continue;
        }
        assert_eq!(rows(&m), want(c), "{k}");
    }
    assert_eq!(cs.len(), 8);
}

fn commit(s: &mut Sprite, op: Option<Op>) {
    if let Some(op) = op {
        apply(s, op).expect("op");
    }
}

#[test]
fn the_select_menu_is_document_state_like_the_trial() {
    let f = fx();
    let cs = cases(&f, "select_");
    for (k, c) in &cs {
        let mut s = rgba_sprite(&vec![Value::String("ff0000ff".repeat(24)); 16]);
        let r = Mask::marquee(W, H, cell(&c["rect"][0]), cell(&c["rect"][1]));
        let what = c["what"].as_str().expect("what");
        if !matches!(what, "all" | "inverse_none") {
            let op = selection::select(&s, &r, SelectMode::Replace);
            commit(&mut s, op);
        }
        let op = match what {
            "all" => selection::select_all(&s),
            "inverse" | "inverse_none" => selection::invert(&s),
            "reselect" => {
                let d = selection::deselect(&s);
                commit(&mut s, d);
                selection::reselect(&s)
            }
            _ => selection::deselect(&s),
        };
        commit(&mut s, op);
        let got = s.mask().cloned().unwrap_or_else(|| Mask::empty(W, H));
        assert_eq!(rows(&got), want(c), "{k}");
    }
    assert_eq!(cs.len(), 6);
}

#[test]
fn modify_expand_contract_border_with_square_and_circle_match_the_trial() {
    let f = fx();
    let cs = cases(&f, "morph_");
    for (k, c) in &cs {
        let kind = match c["kind"].as_str().expect("kind") {
            "expand" => Modify::Expand,
            "contract" => Modify::Contract,
            _ => Modify::Border,
        };
        let brush = match c["brush"].as_str().expect("brush") {
            "circle" => ModifyBrush::Circle,
            _ => ModifyBrush::Square,
        };
        let n = c["n"].as_u64().expect("n") as u32;
        let m = shape(&c["shape"]).modify(kind, n, brush);
        assert_eq!(rows(&m), want(c), "{k}");
    }
    assert_eq!(cs.len(), 90);
}

#[test]
fn the_magic_wand_tolerance_transparency_and_connectivity_match_the_trial() {
    let f = fx();
    let inp = &f["inputs"];
    let wand_s = rgba_sprite(inp["wand_rows_rgba"].as_array().expect("w"));
    let conn_s = rgba_sprite(inp["conn_rows_rgba"].as_array().expect("c"));
    let mut n = 0;
    for prefix in ["wand_", "wandc_"] {
        for (k, c) in cases(&f, prefix) {
            let s = if c["file"] == "conn.ase" {
                &conn_s
            } else {
                &wand_s
            };
            let key = (s.layers()[0].id, s.frames()[0].id);
            let opt = Wand {
                tolerance: c["tolerance"].as_u64().expect("t") as u8,
                contiguous: c["contiguous"].as_bool().expect("c"),
                eight: c["connectivity"] == 8,
                sample: BucketSample::Layer,
            };
            let mut m = wand(s, key, cell(&c["at"]), opt);
            if !c["then"].is_null() {
                let then = &c["then"];
                let mode = match mode_of(&then["modifiers"]) {
                    SelectMode::Replace => SelectMode::Subtract, // Alt sozinho na varinha tira
                    other => other,
                };
                m = m.combine(&wand(s, key, cell(&then["at"]), opt), mode);
            }
            assert_eq!(rows(&m), want(c), "{k}");
            n += 1;
        }
    }
    assert_eq!(n, 45);
}

#[test]
fn color_range_is_the_global_wand_with_the_dialog_colour() {
    let f = fx();
    let s = rgba_sprite(f["inputs"]["wand_rows_rgba"].as_array().expect("w"));
    let key = (s.layers()[0].id, s.frames()[0].id);
    let cs = cases(&f, "range_");
    for (k, c) in &cs {
        let col = unhex(c["color"].as_str().expect("c"));
        let m = color_range(
            &s,
            key,
            [col[0], col[1], col[2], col[3]],
            c["tolerance"].as_u64().expect("t") as u8,
            BucketSample::Layer,
        );
        assert_eq!(rows(&m), want(c), "{k}");
    }
    assert_eq!(cs.len(), 7);
}
