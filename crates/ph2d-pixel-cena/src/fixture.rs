//! A fixture `heroi_p14` do [plano P14 §7](../../../docs/PixelArt/P14_plano.md), em código, para os
//! gates desta onda. P14a-1 monta quadros e tags; a slice `body` e a camada Normal Map entram com
//! as partes que as leem (P14b, P14c).

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{Direction, FrameSet, NewSprite, Op, Sprite, Tag, UserData, apply};

/// As durações dos 6 quadros, em ms.
pub const DURACOES_MS: [u32; 6] = [80, 80, 120, 120, 60, 200];

fn tag(name: &str, from: u32, to: u32, direction: Direction, repeat: u16) -> Tag {
    Tag {
        name: name.into(),
        from,
        to,
        direction,
        repeat,
        color: [0, 0, 0, 255],
        user: UserData::default(),
    }
}

/// 16×16, 6 quadros (`80 80 120 120 60 200`); `idle` 0–1 Forward, `run` 2–4 PingPong repeat 2,
/// `hit` 5 Reverse.
#[must_use]
pub fn heroi_p14() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        ..NewSprite::default()
    })
    .expect("16x16");
    for _ in 1..DURACOES_MS.len() {
        let last = s.frames().last().expect("um quadro").id;
        let (op, _) = frames::new_frame(&mut s, last, NewFrame::Empty).expect("novo quadro");
        apply(&mut s, op).expect("novo quadro");
    }
    let mut fs = s.frames().to_vec();
    for (f, ms) in fs.iter_mut().zip(DURACOES_MS) {
        f.duration_ms = ms;
    }
    let tags = vec![
        tag("idle", 0, 1, Direction::Forward, 0),
        tag("run", 2, 4, Direction::PingPong, 2),
        tag("hit", 5, 5, Direction::Reverse, 0),
    ];
    let slices = s.slices().to_vec();
    apply(
        &mut s,
        Op::Frames(Box::new(FrameSet {
            frames: fs,
            cels: Vec::new(),
            tags,
            slices,
        })),
    )
    .expect("quadros e tags");
    s
}
