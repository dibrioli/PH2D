//! **A cópia de um quadro leva a slice DELE** (`P11_plano.md` §4.4): cada acção que cria quadros a
//! partir de outros (New Frame Copy/Linked, `insert`, o Ctrl+largar de `copy_block`) dá ao quadro
//! novo o valor da slice do quadro cuja arte ele mostra; os que ficam guardam o seu. Exaustivo sobre
//! cada origem e cada fronteira de um sprite de 6 quadros com 3 keys.

use std::collections::BTreeMap;

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::slice;
use ph2d_pixel_core::{
    ColorMode, FrameId, NewSprite, Op, Rect, SliceId, SliceShape, Sprite, apply,
};

fn sprite() -> (Sprite, SliceId) {
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        mode: ColorMode::Rgba,
        ..NewSprite::default()
    })
    .expect("nasce");
    for _ in 1..6 {
        let last = s.frames().last().expect("quadro").id;
        let (op, _) = frames::new_frame(&mut s, last, NewFrame::Empty).expect("novo");
        apply(&mut s, op).expect("aplica");
    }
    let (op, id) = slice::create(&mut s, 0, Rect::new(8, 8, 16, 16)).expect("cria");
    apply(&mut s, op).expect("aplica");
    for (f, x, y) in [(2, 24, 8), (4, 24, 24)] {
        let k = SliceShape {
            bounds: Rect::new(x, y, 16, 16),
            ..SliceShape::default()
        };
        let op = slice::set_shape(&s, id, f, k).expect("key");
        apply(&mut s, op).expect("aplica");
    }
    (s, id)
}

fn por_quadro(s: &Sprite, id: SliceId) -> BTreeMap<FrameId, SliceShape> {
    let c = s.slice(id).expect("a slice");
    s.frames()
        .iter()
        .enumerate()
        .map(|(i, f)| (f.id, c.shape_at(i as u32).expect("valor")))
        .collect()
}

/// Aplica `op` e confere: cada quadro que fica guarda o seu valor, cada novo leva o da origem.
fn confere(s: &Sprite, id: SliceId, op: Op, origem: &[(FrameId, FrameId)], caso: &str) {
    let antes = por_quadro(s, id);
    let mut d = s.clone();
    apply(&mut d, op).expect("aplica");
    let depois = por_quadro(&d, id);
    for (f, v) in &depois {
        let fonte = origem.iter().find(|o| o.0 == *f).map_or(*f, |o| o.1);
        assert_eq!(
            Some(v),
            antes.get(&fonte),
            "{caso}: o quadro {f:?} (arte de {fonte:?})"
        );
    }
}

#[test]
fn a_copied_frame_carries_the_slice_of_the_frame_it_copies() {
    let (s, id) = sprite();
    let ids: Vec<FrameId> = s.frames().iter().map(|f| f.id).collect();
    for (i, &src) in ids.iter().enumerate() {
        for at in 0..=ids.len() {
            for content in [NewFrame::Copy, NewFrame::Linked] {
                let mut c = s.clone();
                let (op, novo) = frames::insert(&mut c, at, src, content, 100).expect("insere");
                confere(
                    &c,
                    id,
                    op,
                    &[(novo, src)],
                    &format!("insert {content:?} {i}→{at}"),
                );
            }
            for j in i..ids.len().min(i + 3) {
                let bloco = &ids[i..=j];
                let mut c = s.clone();
                let (op, novos) = frames::copy_block(&mut c, bloco, at).expect("copia");
                let origem: Vec<_> = novos.iter().copied().zip(bloco.iter().copied()).collect();
                confere(&c, id, op, &origem, &format!("copy_block {i}..={j} → {at}"));
            }
        }
    }
    // O New Frame do menu copia o quadro de onde nasce.
    for &src in &ids {
        let mut c = s.clone();
        let (op, novo) = frames::new_frame(&mut c, src, NewFrame::Copy).expect("novo");
        confere(&c, id, op, &[(novo, src)], "new_frame Copy");
    }
}
