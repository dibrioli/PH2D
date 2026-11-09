//! A fixture `heroi_p14` do [plano P14 §7](../../../docs/PixelArt/P14_plano.md), em código, para os
//! gates desta onda: quadros e tags (P14a), a slice `body` (P14b), o disco de cor e a camada Normal
//! Map com o hemisfério no mesmo recorte (P14c).

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{
    Direction, FrameSet, NewSprite, Op, Patch, Rect, SliceId, SliceKey, SliceShape, Sprite, Tag,
    UserData, apply,
};
use ph2d_pixel_core::{layers, slice};

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
    com_slice_body(&mut s);
    com_disco_e_normal(&mut s);
    s
}

/// O raio do herói (um disco centrado em (8, 8), em píxeis).
pub const RAIO: f64 = 7.5;
/// A cor do herói na camada de cor.
pub const COR: [u8; 4] = [70, 120, 200, 255];

/// A normal do hemisfério no píxel `(x, y)` (centro do píxel), `None` fora do disco: `n·0,5+0,5`
/// com `y` para cima — a codificação da roda Normal Map (`seletor.rs`).
#[must_use]
pub fn hemisferio(x: u32, y: u32) -> Option<[u8; 4]> {
    let c = 8.0;
    let nx = (f64::from(x) + 0.5 - c) / RAIO;
    let ny = -(f64::from(y) + 0.5 - c) / RAIO;
    let d = nx * nx + ny * ny;
    if d > 1.0 {
        return None;
    }
    let nz = (1.0 - d).sqrt();
    let b = |n: f64| ((n * 0.5 + 0.5) * 255.0).round() as u8;
    Some([b(nx), b(ny), b(nz), 255])
}

/// P14c: a camada de cor (o disco) e, por cima, a camada Normal Map com o hemisfério no MESMO
/// recorte, nos 6 quadros.
fn com_disco_e_normal(s: &mut Sprite) {
    let cor = s.layers()[0].id;
    let (op, normal) = layers::new_layer(s, cor).expect("camada normal");
    apply(s, op).expect("camada normal");
    let op = layers::edit(s, normal, |l| l.name = "Normal Map".into()).expect("nome");
    apply(s, op).expect("nome");
    let op = layers::set_normal(s, normal, true).expect("Normal Map");
    apply(s, op).expect("Normal Map");
    let r = s.bounds();
    let px = |f: &dyn Fn(u32, u32) -> Option<[u8; 4]>| -> Vec<u8> {
        (0..r.h)
            .flat_map(|y| (0..r.w).map(move |x| (x, y)))
            .flat_map(|(x, y)| f(x, y).unwrap_or([0; 4]))
            .collect()
    };
    let (pc, pn) = (px(&|x, y| hemisferio(x, y).map(|_| COR)), px(&hemisferio));
    let frames: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    for f in frames {
        for (layer, pixels) in [(cor, pc.clone()), (normal, pn.clone())] {
            let op = Op::Patch(Patch {
                layer,
                frame: f,
                bbox: Some(r),
                rect: r,
                pixels,
            });
            apply(s, op).expect("pintura");
        }
    }
}

/// A caixa da slice `body` nos quadros 0–2: 8×14, pivô nos pés (8, 15).
pub const BODY_0: Rect = Rect::new(4, 2, 8, 14);
/// A key MENOR do quadro 3 (o herói agacha): 6×10, pivô nos pés (8, 15).
pub const BODY_3: Rect = Rect::new(5, 6, 6, 10);

/// A slice `body` (P14b): key no quadro 0, key menor no 3, o pivô muda no 4 (de (8,15) para
/// (6,15)); é a `pivot_slice` do sprite.
fn com_slice_body(s: &mut Sprite) {
    let (op, id) = slice::create(s, 0, BODY_0).expect("slice");
    apply(s, op).expect("slice");
    let mut lista = s.slices().to_vec();
    let body = lista.iter_mut().find(|c| c.id == id).expect("body");
    body.name = "body".into();
    let key = |frame, bounds: Rect, pivot| SliceKey {
        frame,
        shape: SliceShape {
            bounds,
            center: None,
            pivot: Some(pivot),
        },
    };
    body.keys = vec![
        key(0, BODY_0, (4, 13)),
        key(3, BODY_3, (3, 9)),
        key(4, BODY_3, (1, 9)),
    ];
    apply(s, Op::Slices(lista)).expect("keys");
    let op = slice::set_pivot(s, Some(id)).expect("pivô");
    apply(s, op).expect("pivô");
}

/// O id da slice `body`.
#[must_use]
pub fn body(s: &Sprite) -> SliceId {
    s.slices()
        .iter()
        .find(|c| c.name == "body")
        .expect("body")
        .id
}
