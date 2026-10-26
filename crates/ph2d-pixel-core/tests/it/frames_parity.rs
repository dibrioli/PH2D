//! **P4 contra o MVP web** — a fixtura `mvp_frames.json` (`docs/PixelArt/ferramentas/extrai_quadros_mvp.mjs`).
//!
//! A estrutura dos frames (ordem por id, duração, os pixels seguem o id, o duplicado copia, o
//! desfazer e o refazer), o tocar sem tags, a ordem de um ciclo de cada direcção e as tags depois de
//! inserir/apagar/mover. O MVP regista um passo mesmo quando nada muda (mover para o mesmo sítio,
//! a mesma duração); aqui isso não é passo — por isso os estados comparam-se sem as repetições.

use std::collections::BTreeMap;

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::play::{Play, PlayOptions};
use ph2d_pixel_core::{
    Background, ColorMode, Direction, FrameId, History, NewSprite, Op, Patch, Rect, Sprite, apply,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_frames.json")).expect("json")
}

fn n(v: &Value) -> u64 {
    v.as_u64().expect("int")
}

fn indexed(w: u32, h: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn dedup<T: PartialEq>(mut v: Vec<T>) -> Vec<T> {
    v.dedup();
    v
}

/// `(id do MVP, ms, célula (0,0))` de cada frame, na ordem.
type Snap = Vec<(u64, u64, u64)>;

fn snap(s: &Sprite, back: &BTreeMap<FrameId, u64>) -> Snap {
    let l = s.layers()[0].id;
    s.frames()
        .iter()
        .map(|f| (back[&f.id], u64::from(f.duration_ms), u64::from(s.value_at((l, f.id), 0, 0))))
        .collect()
}

fn mvp_snap(v: &Value) -> Snap {
    v.as_array()
        .expect("snap")
        .iter()
        .map(|f| (n(&f["id"]), n(&f["ms"]), n(&f["cell00"])))
        .collect()
}

#[test]
fn frame_ops_undo_and_redo_reproduce_the_mvp_timeline_step_by_step() {
    let f = fx();
    for (k, case) in f["frame_ops"].as_array().expect("ops").iter().enumerate() {
        let mut s = indexed(2, 2);
        let l = s.layers()[0].id;
        let r = Rect::new(0, 0, 1, 1);
        let frame = s.frames()[0].id;
        apply(&mut s, Op::Patch(Patch { layer: l, frame, bbox: Some(r), rect: r, pixels: vec![1] })).expect("marca");
        let mut to: BTreeMap<u64, FrameId> = BTreeMap::from([(1, s.frames()[0].id)]);
        let mut back: BTreeMap<FrameId, u64> = BTreeMap::from([(s.frames()[0].id, 1)]);
        let mut h = History::default();
        let mut states = vec![snap(&s, &back)];
        for st in case["steps"].as_array().expect("steps") {
            let op = match st["op"].as_str().expect("op") {
                "insert" => {
                    let src = st["copy_of"].as_u64().map_or(s.frames()[0].id, |c| to[&c]);
                    let kind = if st["copy_of"].is_null() { NewFrame::Empty } else { NewFrame::Copy };
                    let (op, id) =
                        frames::insert(&mut s, n(&st["index"]) as usize, src, kind, n(&st["ms"]) as u32).expect("insere");
                    to.insert(n(&st["id"]), id);
                    back.insert(id, n(&st["id"]));
                    Some(op)
                }
                "delete" => frames::delete(&s, &[to[&n(&st["id"])]]),
                "move" => {
                    let id = s.frames()[n(&st["from"]) as usize].id;
                    frames::move_to(&s, id, n(&st["to"]) as usize)
                }
                "duration" => frames::set_duration(&s, &[to[&n(&st["id"])]], n(&st["ms"]) as u32),
                o => panic!("op {o}"),
            };
            if let Some(op) = op {
                h.commit(&mut s, op).expect("commit");
            }
            assert_eq!(snap(&s, &back), mvp_snap(&st["after"]), "caso {k}, passo {st}");
            states.push(snap(&s, &back));
        }
        let mut undos = Vec::new();
        while h.undo(&mut s) {
            undos.push(snap(&s, &back));
        }
        let mvp_undos: Vec<Snap> = case["undos"].as_array().expect("u").iter().map(mvp_snap).collect();
        let mut want = vec![states.last().cloned().expect("fim")];
        want.extend(mvp_undos);
        let mut got = vec![states.last().cloned().expect("fim")];
        got.extend(undos);
        assert_eq!(dedup(got), dedup(want), "caso {k}: desfazer");
        let mut redos = vec![snap(&s, &back)];
        while h.redo(&mut s) {
            redos.push(snap(&s, &back));
        }
        let mut want = vec![states[0].clone()];
        want.extend(case["redos"].as_array().expect("r").iter().map(mvp_snap));
        assert_eq!(dedup(redos), dedup(want), "caso {k}: refazer");
    }
    assert!(f["frame_limits"]["delete_last"].is_string());
    let s = indexed(1, 1);
    assert!(frames::delete(&s, &[s.frames()[0].id]).is_none(), "o último frame não se apaga");
    assert_eq!(u64::from(s.frames()[0].duration_ms), n(&f["frame_limits"]["default_ms"]));
}

fn timeline(ms: &[u32]) -> Sprite {
    let mut s = indexed(1, 1);
    for i in 1..ms.len() {
        let after = s.frames()[i - 1].id;
        let (op, _) = frames::new_frame(&mut s, after, NewFrame::Empty).expect("frame");
        apply(&mut s, op).expect("frame");
    }
    let ids: Vec<FrameId> = s.frames().iter().map(|f| f.id).collect();
    for (id, &m) in ids.iter().zip(ms) {
        if let Some(op) = frames::set_duration(&s, &[*id], m) {
            apply(&mut s, op).expect("ms");
        }
    }
    s
}

#[test]
fn playback_without_tags_shows_the_frame_the_mvp_shows_at_each_instant() {
    let f = fx();
    let mut vistos = 0;
    for c in f["playback"].as_array().expect("pb") {
        let ms: Vec<u32> = c["ms"].as_array().expect("ms").iter().map(|v| n(v) as u32).collect();
        let s = timeline(&ms);
        for (key, once) in [("loop", false), ("once", true)] {
            for tv in c[key].as_array().expect("tv") {
                let t = tv[0].as_i64().expect("t");
                if t < 0 {
                    continue; // o relógio do host nunca anda para trás
                }
                let mut p = Play::start(&s, 0, PlayOptions { once, all: false });
                p.advance(&s, t as u32);
                assert_eq!(p.index as u64, n(&tv[1]), "ms {ms:?}, t {t}, {key}");
                vistos += 1;
            }
        }
    }
    assert!(vistos > 1000, "{vistos}");
}

#[test]
fn one_cycle_of_each_tag_direction_is_the_mvp_play_order() {
    let f = fx();
    for c in f["play_order"].as_array().expect("po") {
        let (from, to) = (n(&c["from"]) as u32, n(&c["to"]) as u32);
        let mut s = timeline(&vec![100; to as usize + 2]);
        { let op = frames::new_tag(&s, "t", from, to).expect("tag"); apply(&mut s, op) }.expect("tag");
        let d = match c["direction"].as_str().expect("d") {
            "forward" => Direction::Forward,
            "reverse" => Direction::Reverse,
            "pingpong" => Direction::PingPong,
            _ => Direction::PingPongReverse,
        };
        if let Some(op) = frames::edit_tag(&s, 0, |t| t.direction = d) {
            apply(&mut s, op).expect("dir");
        }
        let want: Vec<usize> = c["order"].as_array().expect("o").iter().map(|v| n(v) as usize).collect();
        let start = if matches!(d, Direction::Forward | Direction::PingPong) { from } else { to };
        let mut p = Play::start(&s, start as usize, PlayOptions::default());
        let mut got = vec![p.index];
        while got.len() < want.len() * 2 {
            p.step(&s);
            got.push(p.index);
        }
        let ciclo: Vec<usize> = want.iter().chain(&want).copied().collect();
        assert_eq!(got, ciclo, "{c}");
    }
}

#[test]
fn tags_follow_frame_insert_delete_move_and_undo_as_in_the_mvp() {
    let f = fx();
    let tags = |s: &Sprite| -> Vec<(String, u64, u64)> {
        s.tags().iter().map(|t| (t.name.clone(), u64::from(t.from), u64::from(t.to))).collect()
    };
    let mvp = |v: &Value| -> Vec<(String, u64, u64)> {
        v.as_array().expect("t").iter().map(|t| (n(&t["id"]).to_string(), n(&t["from"]), n(&t["to"]))).collect()
    };
    for (k, c) in f["tag_ops"].as_array().expect("to").iter().enumerate() {
        let mut s = timeline(&vec![100; n(&c["frames"]) as usize]);
        for t in c["tags"].as_array().expect("tags") {
            let op = frames::new_tag(&s, &n(&t["id"]).to_string(), n(&t["from"]) as u32, n(&t["to"]) as u32);
            apply(&mut s, op.expect("tag")).expect("tag");
        }
        let mut h = History::default();
        let mut states = vec![tags(&s)];
        for st in c["steps"].as_array().expect("steps") {
            let op = match st["op"].as_str().expect("op") {
                "insert" => {
                    let src = s.frames()[0].id;
                    frames::insert(&mut s, n(&st["index"]) as usize, src, NewFrame::Empty, 100).map(|(o, _)| o)
                }
                "delete" => frames::delete(&s, &[s.frames()[n(&st["index"]) as usize].id]),
                _ => frames::move_to(&s, s.frames()[n(&st["from"]) as usize].id, n(&st["to"]) as usize),
            };
            if let Some(op) = op {
                h.commit(&mut s, op).expect("commit");
            }
            assert_eq!(tags(&s), mvp(&st["after"]), "caso {k}, {st}");
            states.push(tags(&s));
        }
        let fin = tags(&s);
        let mut got = vec![fin.clone()];
        while h.undo(&mut s) {
            got.push(tags(&s));
        }
        let mut want = vec![fin];
        want.extend(c["undos"].as_array().expect("u").iter().map(mvp));
        assert_eq!(dedup(got), dedup(want), "caso {k}: desfazer");
        assert_eq!(tags(&s), states[0]);
    }
}
