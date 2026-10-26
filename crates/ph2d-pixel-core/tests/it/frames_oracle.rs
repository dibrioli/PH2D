//! **Quadros contra o Aseprite Trial observado** — a fixtura `oracle_trial_frames.json`
//! (`docs/PixelArt/ferramentas/trial_ase_quadros.py` escreve os `.ase`, `trial_quadros.py` lê a tela e
//! grava o vídeo do tocar, `trial_fixture_quadros.py` grava). Cada caso compõe-se (ou toca-se) pelo
//! NOSSO documento e tem de dar a tela célula a célula / a sequência frame a frame.

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::layers;
use ph2d_pixel_core::play::{Play, PlayOptions};
use ph2d_pixel_core::{
    Background, ColorMode, Direction, FrameId, LayerId, NewSprite, Onion, Op, Patch, Rect, Sprite,
    apply, compose_rgba, compose_with_onion,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_frames.json")).expect("json")
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect()
}

fn sprite(w: u32, h: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn camada(s: &mut Sprite, above: Option<LayerId>) -> LayerId {
    match above {
        None => s.layers()[0].id,
        Some(a) => {
            let (op, id) = layers::new_layer(s, a).expect("nova");
            apply(s, op).expect("nova");
            id
        }
    }
}

fn pinta(s: &mut Sprite, layer: LayerId, frame: FrameId, bytes: Vec<u8>) {
    let r = Rect::new(0, 0, s.width(), s.height());
    apply(s, Op::Patch(Patch { layer, frame, bbox: Some(r), rect: r, pixels: bytes })).expect("patch");
}

fn rgb(s: &Sprite, frame: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, frame, None, &mut out);
    out.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect()
}

fn dims(f: &Value) -> (u32, u32) {
    let w = f["w"].as_u64().expect("w") as u32;
    (w, f["n"].as_u64().expect("n") as u32 / w)
}

#[test]
fn the_cel_opacity_multiplies_the_layer_opacity_as_on_the_trial_screen() {
    let f = fx();
    let (w, h) = dims(&f);
    let (back, src) = (unhex(f["back"].as_str().expect("b")), unhex(f["src"]["A"].as_str().expect("a")));
    let casos = f["cel"].as_array().expect("cel");
    assert_eq!(casos.len(), 7);
    for c in casos {
        let (lo, co) = (c["layer"].as_u64().expect("l") as u8, c["cel"].as_u64().expect("c") as u8);
        let mut s = sprite(w, h);
        let fr = s.frames()[0].id;
        let b = camada(&mut s, None);
        pinta(&mut s, b, fr, back.clone());
        let l = camada(&mut s, Some(b));
        pinta(&mut s, l, fr, src.clone());
        if let Some(op) = layers::edit(&s, l, |x| x.opacity = lo) {
            apply(&mut s, op).expect("camada");
        }
        if let Some(op) = frames::set_cel(&s, (l, fr), Some(co), None) {
            apply(&mut s, op).expect("cel");
        }
        let want = unhex(c["screen_rgb"].as_str().expect("rgb"));
        let bad = rgb(&s, fr).chunks(3).zip(want.chunks(3)).filter(|(a, b)| a != b).count();
        assert_eq!(bad, 0, "camada {lo} × cel {co}: {bad} células diferentes");
    }
}

#[test]
fn cels_draw_by_layer_index_plus_z_then_z_as_on_the_trial_screen() {
    let f = fx();
    let (w, h) = dims(&f);
    let back = unhex(f["back"].as_str().expect("b"));
    let casos = f["z"].as_array().expect("z");
    assert_eq!(casos.len(), 10);
    for c in casos {
        let mut s = sprite(w, h);
        let fr = s.frames()[0].id;
        let mut top = camada(&mut s, None);
        pinta(&mut s, top, fr, back.clone());
        let zs = c["z"].as_array().expect("zs");
        let mut ids = Vec::new();
        for (k, name) in ["A", "B", "C"].iter().enumerate() {
            top = camada(&mut s, Some(top));
            pinta(&mut s, top, fr, unhex(f["src"][name].as_str().expect("src")));
            let z = zs[k].as_i64().expect("z") as i16;
            if let Some(op) = frames::set_cel(&s, (top, fr), None, Some(z)) {
                apply(&mut s, op).expect("z");
            }
            ids.push(top);
        }
        let want = unhex(c["screen_rgb"].as_str().expect("rgb"));
        let bad = rgb(&s, fr).chunks(3).zip(want.chunks(3)).filter(|(a, b)| a != b).count();
        assert_eq!(bad, 0, "{}: {bad} células diferentes", c["name"]);
    }
}

/// O sprite dos `on_*.ase`: 5 frames de uma camada; os 0, 1, 3, 4 pintam a sua quarta parte (em
/// cima alfa 255, em baixo 128); o activo (2) vazio ou com a faixa cinza opaca das 8 linhas de cima.
fn onion_sprite(f: &Value, w: u32, h: u32, faixa: bool) -> (Sprite, Vec<FrameId>) {
    let mut s = sprite(w, h);
    let l = s.layers()[0].id;
    let mut ids = vec![s.frames()[0].id];
    for _ in 1..5 {
        let after = *ids.last().expect("último");
        let (op, id) = frames::new_frame(&mut s, after, NewFrame::Empty).expect("frame");
        apply(&mut s, op).expect("frame");
        ids.push(id);
    }
    for (k, q) in [(0usize, 0u32), (1, 1), (3, 2), (4, 3)] {
        let c = f["onion_colors"][k.to_string()].as_array().expect("cor");
        let c: Vec<u8> = c.iter().map(|v| v.as_u64().expect("c") as u8).collect();
        let mut px = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let a = if x / (w / 4) != q { 0 } else if y < h / 2 { 255 } else { 128 };
                px.extend_from_slice(&[c[0], c[1], c[2], a]);
            }
        }
        pinta(&mut s, l, ids[k], px);
    }
    if faixa {
        let px = (0..w * h)
            .flat_map(|i| if i / w < 8 { [128, 128, 128, 255] } else { [0, 0, 0, 0] })
            .collect();
        pinta(&mut s, l, ids[2], px);
    }
    (s, ids)
}

#[test]
fn the_onion_skin_matches_the_trial_in_position_fade_and_red_blue_tint() {
    let f = fx();
    let (w, h) = (f["w"].as_u64().expect("w") as u32, f["h"].as_u64().expect("h") as u32);
    let mut vistos = 0;
    for (nome, v) in f["onion"].as_object().expect("onion") {
        let ini = &v["ini"];
        let get = |k: &str, d: u64| ini.get(k).and_then(Value::as_u64).unwrap_or(d);
        let onion = Onion {
            prev: get("prev_frames", 1) as u32,
            next: get("next_frames", 1) as u32,
            base: get("opacity_base", 68) as u8,
            step: get("opacity_step", 28) as u8,
            tint: get("type", 0) == 1,
            front: get("position", 0) == 1,
        };
        let faixa = v["file"].as_str() == Some("on_faixa.ase");
        let (s, ids) = onion_sprite(&f, w, h, faixa);
        let off = unhex(v["off_rgb"].as_str().expect("off"));
        // A leitura sem papel vegetal é o xadrez (e a faixa, que é opaca): o destino opaco.
        let under: Vec<u8> = off.chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect();
        let under = if faixa {
            // por baixo da faixa está o xadrez do Trial; ela é opaca e por isso não se lê — o
            // xadrez de 16 células cinza 128/192 (Trial: Checked size 16×16)
            under
                .chunks(4)
                .enumerate()
                .flat_map(|(i, p)| {
                    let (x, y) = (i as u32 % w, i as u32 / w);
                    if y < 8 {
                        let c = if ((x / 16) + (y / 16)) % 2 == 0 { 128 } else { 192 };
                        [c, c, c, 255]
                    } else {
                        [p[0], p[1], p[2], 255]
                    }
                })
                .collect()
        } else {
            under
        };
        let got = compose_with_onion(&s, ids[2], None, s.bounds(), &onion, Some(&under));
        let want = unhex(v["on_rgb"].as_str().expect("on"));
        let bad = got
            .chunks(4)
            .zip(want.chunks(3))
            .filter(|(a, b)| a[..3] != b[..])
            .count();
        assert_eq!(bad, 0, "{nome}: {bad} de {} células diferentes", w * h);
        vistos += 1;
    }
    assert_eq!(vistos, 4);
}

#[test]
fn playback_steps_through_tags_passes_and_exits_as_the_trial_played_them() {
    let f = fx();
    let casos = f["play"].as_array().expect("play");
    assert_eq!(casos.len(), 17);
    for c in casos {
        let durs: Vec<u32> = c["durations"].as_array().expect("d").iter().map(|v| v.as_u64().expect("ms") as u32).collect();
        let mut s = sprite(4, 4);
        let mut ids = vec![s.frames()[0].id];
        for _ in 1..durs.len() {
            let after = *ids.last().expect("último");
            let (op, id) = frames::new_frame(&mut s, after, NewFrame::Empty).expect("frame");
            apply(&mut s, op).expect("frame");
            ids.push(id);
        }
        for (id, &ms) in ids.iter().zip(&durs) {
            if let Some(op) = frames::set_duration(&s, &[*id], ms) {
                apply(&mut s, op).expect("ms");
            }
        }
        if let Some(t) = c["tag"].as_object() {
            let n = |k: &str| t[k].as_u64().expect("n");
            { let op = frames::new_tag(&s, "walk", n("from") as u32, n("to") as u32).expect("tag"); apply(&mut s, op) }.expect("tag");
            let op = frames::edit_tag(&s, 0, |x| {
                x.direction = Direction::from_code(n("direction") as u8).expect("dir");
                x.repeat = n("repeat") as u16;
            });
            if let Some(op) = op {
                apply(&mut s, op).expect("props");
            }
        }
        let want: Vec<usize> = c["frames"].as_array().expect("f").iter().map(|v| v.as_u64().expect("i") as usize).collect();
        let mut p = Play::start(&s, c["start"].as_u64().expect("s") as usize, PlayOptions::default());
        let mut got = vec![p.index];
        while got.len() < want.len() {
            p.step(&s);
            got.push(p.index);
        }
        assert_eq!(got, want, "{}", c["file"]);
        // o tempo de cada frame no vídeo (50 q/s: ±20 ms) é a duração dele
        for (k, ms) in c["ms"].as_array().expect("ms").iter().enumerate() {
            let d = durs[want[k + 1]] as i64;
            let m = ms.as_i64().expect("ms");
            assert!((m - d).abs() <= 20, "{}: frame {} durou {m} ms, devia {d}", c["file"], want[k + 1]);
        }
    }
}
