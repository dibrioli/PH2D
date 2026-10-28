//! **Transformar contra o Aseprite Trial observado** — a mesma fixtura `oracle_trial_selection.json`
//! (as imagens lidas da tela: ids da ENTRADA, `-2` transparente, `-1` cor nova com `rgb`). Cada caso
//! refaz-se pelo núcleo e compara-se célula a célula; o censo no fim prova que os 367 casos têm
//! dono (gate, medição ou recusa listada).
//!
//! Fixados aqui: Edit ▸ Flip/Rotate/Shift, Rotate/Flip Canvas, a escala e o 90° da transformação,
//! Sprite Size nearest e bilinear (exacta: a coordenada é o passo SOMADO em `f64`; `d·passo`,
//! pontos fixos 2⁴–2²⁰ e `f32` ficavam em 8 392/8 463) (+ RotSprite a 2×), Canvas Size, Crop, Trim
//! e Paste. ⛔ Recusas MEDIDAS: a rotação não-90° e a inclinação não têm fórmula
//! do Trial (só tabelas): medem-se em `mede_rotacao.rs`, nunca por tabela.

use ph2d_pixel_core::canvas::{self, Anchor, Borders, SizeMethod, anchor_borders};
use ph2d_pixel_core::edit::{self, Rotation};
use ph2d_pixel_core::selection::{self, Mask, SelectMode};
use ph2d_pixel_core::transform::lift;
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, NewSprite, Op, Patch, Rect, Sprite, apply, compose_rgba, layers,
};
use serde_json::Value;

use crate::selection_oracle::{cases, cell, fx, rgba_sprite, unhex};

pub(crate) fn run(s: &mut Sprite, op: Option<Op>) {
    if let Some(op) = op {
        apply(s, op).expect("op");
    }
}

fn key(s: &Sprite) -> (ph2d_pixel_core::LayerId, FrameId) {
    (s.layers()[0].id, s.frames()[0].id)
}

fn composed(s: &Sprite) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    compose_rgba(s, s.frames()[0].id, None, &mut out);
    out.as_chunks::<4>().0.to_vec()
}

fn ids(c: &Value) -> Vec<Vec<i64>> {
    c["ids"]
        .as_array()
        .expect("ids")
        .iter()
        .map(|r| {
            r.as_array()
                .expect("r")
                .iter()
                .map(|v| v.as_i64().expect("i"))
                .collect()
        })
        .collect()
}

/// O sprite `xf.ase`: a forma de 19 ids sobre transparente.
pub(crate) fn xf_sprite(f: &Value) -> Sprite {
    let inp = &f["inputs"];
    let rows: Vec<Value> = inp["xf_ids"]
        .as_array()
        .expect("xf")
        .iter()
        .map(|r| {
            Value::String(
                r.as_array()
                    .expect("r")
                    .iter()
                    .map(|id| match id.as_i64().expect("id") {
                        0 => "00000000".to_owned(),
                        n => inp["xf_colors_rgba"][n.to_string()]
                            .as_str()
                            .expect("cor")
                            .to_owned(),
                    })
                    .collect(),
            )
        })
        .collect();
    rgba_sprite(&rows)
}

/// A tela do Trial (ids de `xf.ase`) contra o nosso composto.
pub(crate) fn xf_ids(f: &Value, s: &Sprite) -> Vec<Vec<i64>> {
    let px = composed(s);
    let w = s.width() as usize;
    let colour = |id: i64| {
        unhex(
            f["inputs"]["xf_colors_rgba"][id.to_string()]
                .as_str()
                .expect("c"),
        )
    };
    (0..s.height() as usize)
        .map(|y| {
            (0..w)
                .map(|x| {
                    let p = px[y * w + x];
                    if p[3] == 0 {
                        return 0;
                    }
                    (1..=19).find(|&i| colour(i) == p).unwrap_or(-9)
                })
                .collect()
        })
        .collect()
}

/// O composto contra a leitura de um caso `sz`/`szb`/`pm`/`trim*`: id da entrada `sz`, `-2`
/// transparente, `-1` a cor da tela em `rgb`. Devolve as células que diferem.
pub(crate) fn sz_diff(f: &Value, s: &Sprite, c: &Value) -> usize {
    let sz: Vec<u8> = f["inputs"]["sz_rows_rgba"]
        .as_array()
        .expect("sz")
        .iter()
        .flat_map(|r| unhex(r.as_str().expect("r")))
        .collect();
    let want = ids(c);
    assert_eq!(
        (s.width() as usize, s.height() as usize),
        (want[0].len(), want.len()),
        "tamanho"
    );
    let rgb: Vec<Vec<u8>> = c["rgb"]
        .as_array()
        .map(|a| a.iter().map(|r| unhex(r.as_str().expect("r"))).collect())
        .unwrap_or_default();
    let px = composed(s);
    let w = s.width() as usize;
    let mut bad = 0;
    for (y, row) in want.iter().enumerate() {
        for (x, &id) in row.iter().enumerate() {
            let p = px[y * w + x];
            let ok = match id {
                -2 => p[3] == 0,
                -1 => p[3] == 255 && p[..3] == rgb[y][3 * x..3 * x + 3],
                n => p[..] == sz[4 * n as usize..4 * n as usize + 4],
            };
            bad += usize::from(!ok);
        }
    }
    bad
}

fn sz_sprite(f: &Value) -> Sprite {
    rgba_sprite(f["inputs"]["sz_rows_rgba"].as_array().expect("sz"))
}

/// O mesmo conteúdo numa camada Background.
fn background_sprite(rows: &[Value]) -> Sprite {
    let src = rgba_sprite(rows);
    let mut s = Sprite::new(NewSprite {
        width: src.width(),
        height: src.height(),
        mode: ColorMode::Rgba,
        background: Background::Black,
    })
    .expect("sprite");
    let r = s.bounds();
    let k = key(&s);
    let pixels = src.read_rect(key(&src), r);
    run(
        &mut s,
        Some(Op::Patch(Patch {
            layer: k.0,
            frame: k.1,
            bbox: Some(r),
            rect: r,
            pixels,
        })),
    );
    s
}

fn select_rect(s: &mut Sprite, sel: &Value) {
    let m = if sel.is_string() {
        Mask::full(s.width(), s.height())
    } else {
        Mask::marquee(s.width(), s.height(), cell(&sel[0]), cell(&sel[1]))
    };
    let op = selection::select(s, &m, SelectMode::Replace);
    run(s, op);
}

#[test]
fn edit_flip_rotate_shift_and_rotate_canvas_match_the_trial() {
    let f = fx();
    let cs = cases(&f, "edit_");
    for (k, c) in &cs {
        let mut s = xf_sprite(&f);
        if !c["selection"].is_null() {
            select_rect(&mut s, &c["selection"]);
        }
        let kk = key(&s);
        let op = match c["op"].as_str().expect("op") {
            "flip_h" => edit::flip(&s, kk, true, 0),
            "flip_v" => edit::flip(&s, kk, false, 0),
            "edit_rotate_180" => edit::rotate(&s, kk, Rotation::R180, 0),
            "edit_rotate_90cw" => edit::rotate(&s, kk, Rotation::Cw90, 0),
            "edit_rotate_90ccw" => edit::rotate(&s, kk, Rotation::Ccw90, 0),
            "edit_shift_left" => edit::shift(&s, kk, -1, 0, 0),
            "edit_shift_right" => edit::shift(&s, kk, 1, 0, 0),
            "edit_shift_up" => edit::shift(&s, kk, 0, -1, 0),
            "edit_shift_down" => edit::shift(&s, kk, 0, 1, 0),
            "rotate_canvas_180" => canvas::rotate_canvas(&s, Rotation::R180),
            "rotate_canvas_90cw" => canvas::rotate_canvas(&s, Rotation::Cw90),
            "rotate_canvas_90ccw" => canvas::rotate_canvas(&s, Rotation::Ccw90),
            "rotate_canvas_flip_h" => canvas::flip_canvas(&s, true),
            "rotate_canvas_flip_v" => canvas::flip_canvas(&s, false),
            other => panic!("op {other}"),
        };
        run(&mut s, op);
        assert_eq!(xf_ids(&f, &s), ids(c), "{k}");
    }
    assert_eq!(cs.len(), 23);
}

/// A caixa que o Ctrl+T do Trial levanta: os limites do cel (a forma 7×5 em (9,5)).
pub(crate) fn lifted_box(s: &Sprite) -> Mask {
    Mask::marquee(s.width(), s.height(), (9, 5), (15, 9))
}

#[test]
fn transform_scale_fields_and_ninety_degrees_match_the_trial() {
    use ph2d_pixel_core::affine::{Affine, Linear, Resample};
    let f = fx();
    let mut n = 0;
    for (k, c) in cases(&f, "xform_") {
        let fields = &c["fields"];
        let method = if c["algorithm"] == "rotsprite" {
            Resample::RotSprite
        } else {
            Resample::Fast
        };
        let mut s = xf_sprite(&f);
        let kk = key(&s);
        let original = lifted_box(&s);
        let fl = lift(&s, kk, Some(&original)).expect("lift");
        let moved = if let (Some(w), Some(h)) = (fields["w"].as_u64(), fields["h"].as_u64()) {
            edit::scale_fields(&fl, w as u32, h as u32)
        } else if fields["angle"].as_i64() == Some(90) {
            let t = Affine::around_center(Linear::from_screen_ccw(90.0, 1.0, 1.0, 0.0), fl.rect());
            edit::transform(&fl, &t, method, 0).expect("90")
        } else {
            continue; // sem fórmula do Trial: `mede_rotacao.rs`
        };
        let op = edit::commit(&s, kk, &original, &moved, 0);
        run(&mut s, op);
        assert_eq!(xf_ids(&f, &s), ids(c), "{k}");
        n += 1;
    }
    assert_eq!(n, 8, "3 escalas + 90° por algoritmo");
}

#[test]
fn sprite_size_nearest_bilinear_and_rotsprite_at_2x_match_the_trial() {
    let f = fx();
    let sz: Vec<u8> = f["inputs"]["sz_rows_rgba"]
        .as_array()
        .expect("sz")
        .iter()
        .flat_map(|r| unhex(r.as_str().expect("r")))
        .collect();
    let (mut gated, mut exact, mut channels) = (0, 0, 0);
    for (k, c) in cases(&f, "size_") {
        let (w, h) = (
            c["w"].as_u64().expect("w") as u32,
            c["h"].as_u64().expect("h") as u32,
        );
        let method = match c["method"].as_str().expect("m") {
            "nearest" => SizeMethod::Nearest,
            "bilinear" => SizeMethod::Bilinear,
            _ => SizeMethod::RotSprite,
        };
        if method == SizeMethod::RotSprite && (w % 24 != 0 || h % 16 != 0) {
            continue; // padrão próprio do Trial: `mede_rotacao.rs`
        }
        let mut s = sz_sprite(&f);
        let op = canvas::sprite_size(&s, w, h, method);
        run(&mut s, op);
        if method == SizeMethod::Bilinear {
            let rgb: Vec<Vec<u8>> = c["rgb"]
                .as_array()
                .expect("rgb")
                .iter()
                .map(|r| unhex(r.as_str().expect("r")))
                .collect();
            let px = composed(&s);
            for y in 0..h as usize {
                for x in 0..w as usize {
                    for ch in 0..3 {
                        let (a, b) = (px[y * w as usize + x][ch], rgb[y][3 * x + ch]);
                        assert_eq!(a, b, "{k} ({x},{y}) canal {ch}");
                        exact += usize::from(a == b);
                        channels += 1;
                    }
                }
            }
        } else {
            assert_eq!(sz_diff(&f, &s, c), 0, "{k}");
            let _ = &sz;
        }
        gated += 1;
    }
    assert_eq!(gated, 9, "4 nearest + 4 bilinear + RotSprite a 2×");
    assert_eq!((exact, channels), (8463, 8463), "a bilinear é exacta");
}

#[test]
fn canvas_size_anchors_borders_and_the_background_colour_match_the_trial() {
    let f = fx();
    let cs = cases(&f, "canvas_");
    for (k, c) in &cs {
        let rows = f["inputs"]["sz_rows_rgba"].as_array().expect("sz");
        let mut s = if c["file"] == "szb.ase" {
            background_sprite(rows)
        } else {
            sz_sprite(&f)
        };
        let b = match c["borders"].as_array() {
            Some(b) => {
                let v: Vec<i32> = b.iter().map(|v| v.as_i64().expect("b") as i32).collect();
                Borders {
                    left: v[0],
                    top: v[1],
                    right: v[2],
                    bottom: v[3],
                }
            }
            None => {
                let a = match c["anchor"].as_str() {
                    Some("tl") => Anchor { col: 0, row: 0 },
                    Some("br") => Anchor { col: 2, row: 2 },
                    _ => Anchor::CENTER,
                };
                let (w, h) = (c["w"].as_u64().expect("w"), c["h"].as_u64().expect("h"));
                anchor_borders(24, 16, w as u32, h as u32, a)
            }
        };
        let black = s.value_of_rgba([0, 0, 0, 255]);
        let op = canvas::canvas_size(&s, b, black);
        run(&mut s, op);
        assert_eq!(sz_diff(&f, &s, c), 0, "{k}");
    }
    assert_eq!(cs.len(), 8);
}

#[test]
fn crop_to_the_selection_bounds_and_trim_match_the_trial() {
    let f = fx();
    let mut n = 0;
    for (k, c) in cases(&f, "crop_") {
        let mut s = sz_sprite(&f);
        let sel = &c["selection"];
        let (a, b) = (cell(&sel[1]), cell(&sel[2]));
        let m = if sel[0] == "ellipse" {
            Mask::ellipse(24, 16, a, b)
        } else {
            Mask::marquee(24, 16, a, b)
        };
        let op = selection::select(&s, &m, SelectMode::Replace);
        run(&mut s, op);
        let op = canvas::crop_to_selection(&s);
        run(&mut s, op);
        assert_eq!(sz_diff(&f, &s, c), 0, "{k}");
        assert!(s.mask().is_none(), "{k}: o crop tira a selecção");
        n += 1;
    }
    for (k, c) in cases(&f, "trim_") {
        let trim = f["inputs"]["trim_rows_rgba"].as_array().expect("t");
        let blue: Vec<Value> = trim
            .iter()
            .map(|r| {
                let b = unhex(r.as_str().expect("r"));
                Value::String(
                    b.as_chunks::<4>()
                        .0
                        .iter()
                        .map(|p| {
                            if p[3] == 0 {
                                "0000ffff".to_owned()
                            } else {
                                p.iter().map(|v| format!("{v:02x}")).collect()
                            }
                        })
                        .collect(),
                )
            })
            .collect();
        let mut s = match c["file"].as_str().expect("file") {
            "trim.ase" => rgba_sprite(trim),
            "trimc.ase" => rgba_sprite(&blue),
            _ => background_sprite(&blue),
        };
        let op = canvas::trim(&s);
        run(&mut s, op);
        assert_eq!(sz_diff(&f, &s, c), 0, "{k}");
        n += 1;
    }
    assert_eq!(n, 5);
}

/// `pm.ase`: o `sz` em baixo (a camada ACTIVA ao abrir) e azul na metade esquerda por cima.
fn pm_sprite(f: &Value) -> Sprite {
    let mut s = sz_sprite(f);
    let bottom = s.layers()[0].id;
    let (op, top) = layers::new_layer(&mut s, bottom).expect("camada");
    run(&mut s, Some(op));
    let r = Rect::new(0, 0, 12, 16);
    let frame = s.frames()[0].id;
    run(
        &mut s,
        Some(Op::Patch(Patch {
            layer: top,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: [0, 0, 255, 255].repeat(r.area()),
        })),
    );
    s
}

#[test]
fn copy_copy_merged_and_paste_land_where_the_trial_puts_them() {
    let f = fx();
    let cs = cases(&f, "paste_");
    for (k, c) in &cs {
        let pm = c["file"] == "pm.ase";
        let mut s = if pm { pm_sprite(&f) } else { sz_sprite(&f) };
        let kk = key(&s);
        let (a, b) = (cell(&c["copy"][0]), cell(&c["copy"][1]));
        let op = selection::select(&s, &Mask::marquee(24, 16, a, b), SelectMode::Replace);
        run(&mut s, op);
        let mut fl = if c["op"] == "copy_merged_paste" {
            edit::copy_merged(&s, kk.1)
        } else {
            edit::copy(&s, kk)
        }
        .expect("copy");
        let then = c["then"].as_str().expect("then");
        if then.starts_with("Esc") {
            let op = selection::deselect(&s);
            run(&mut s, op);
            fl.x += if then.contains("x10") { 10 } else { 8 };
        } else {
            let m = Mask::marquee(24, 16, (14, 8), (20, 13));
            let op = selection::select(&s, &m, SelectMode::Replace);
            run(&mut s, op);
        }
        let op = edit::paste(&s, kk, &fl, 0);
        run(&mut s, op);
        assert_eq!(sz_diff(&f, &s, c), 0, "{k}");
    }
    assert_eq!(cs.len(), 4);
}

/// Cada caso da fixtura tem dono: uma família por prefixo, e a soma é a fixtura inteira.
#[test]
fn every_trial_selection_case_belongs_to_a_counted_family() {
    let f = fx();
    const FAMILIES: [(&str, usize); 17] = [
        ("rect_", 10),
        ("ell_", 112),
        ("lasso_", 6),
        ("plasso_", 6),
        ("mode_", 8),
        ("select_", 6),
        ("morph_", 90),
        ("wand_", 35),
        ("wandc_", 10),
        ("range_", 7),
        ("edit_", 23),
        ("xform_", 25),
        ("size_", 12),
        ("canvas_", 8),
        ("crop_", 2),
        ("trim_", 3),
        ("paste_", 4),
    ];
    let all = f["cases"].as_object().expect("cases");
    for k in all.keys() {
        let owners = FAMILIES.iter().filter(|(p, _)| k.starts_with(p)).count();
        assert_eq!(owners, 1, "{k} sem família (ou com duas)");
    }
    for (p, n) in FAMILIES {
        assert_eq!(cases(&f, p).len(), n, "{p}");
    }
    assert_eq!(all.len(), FAMILIES.iter().map(|(_, n)| n).sum::<usize>());
    assert_eq!(all.len(), 367);
}
