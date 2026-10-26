//! **As leis da P4**: cada acção do menu Frame é UM passo com inverso byte-igual; o cel ligado
//! partilha a imagem (pintar um muda todos, desligar separa, apagar o dono passa a imagem ao
//! seguinte, apagar tudo deixa a imagem vazia sem quebrar a ligação); o formato 1.2 leva tags,
//! ligações, z, opacidade do cel e a camada *continuous*; o documento recusa o que viola as leis.

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::layers;
use ph2d_pixel_core::{
    Background, Cel, ColorMode, Direction, FrameId, FrameSet, LayerId, NewSprite, Op, OpError,
    Patch, Rect, Sprite, Tag, apply, codec, compose_rgba,
};

fn sprite() -> Sprite {
    Sprite::new(NewSprite {
        width: 4,
        height: 4,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn dot(s: &mut Sprite, l: LayerId, f: FrameId, x: i32, c: [u8; 4]) {
    let r = Rect::new(x, 0, 1, 1);
    let bbox = s.cel((l, f)).map_or(r, |k| k.bbox.union(r));
    apply(
        s,
        Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(bbox),
            rect: r,
            pixels: c.to_vec(),
        }),
    )
    .expect("dot");
}

fn img(s: &Sprite, f: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, f, None, &mut out);
    out
}

/// Aplica, desfaz (tem de voltar ao byte) e refaz (tem de dar o mesmo de antes).
fn one_step(s: &mut Sprite, op: Op) {
    let before = s.clone();
    let inv = apply(s, op).expect("aplica");
    let after = s.clone();
    let redo = apply(s, inv).expect("desfaz");
    assert_eq!(*s, before, "o inverso não devolveu o sprite ao byte");
    apply(s, redo).expect("refaz");
    assert_eq!(*s, after, "o refazer não reproduziu");
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

#[test]
fn every_frame_action_is_one_step_with_a_byte_exact_inverse() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Copy).expect("novo");
    one_step(&mut s, op);
    let (op, f2) = frames::new_frame(&mut s, f1, NewFrame::Linked).expect("ligado");
    one_step(&mut s, op);
    let (op, _) = frames::new_frame(&mut s, f2, NewFrame::Empty).expect("vazio");
    one_step(&mut s, op);
    {
        let op = frames::new_tag(&s, "walk", 1, 3).expect("tag");
        one_step(&mut s, op)
    };
    {
        let op = frames::edit_tag(&s, 0, |t| {
            (t.direction, t.repeat) = (Direction::PingPong, 2)
        })
        .expect("props");
        one_step(&mut s, op)
    };
    {
        let op = frames::set_duration(&s, &[f1, f2], 250).expect("ms");
        one_step(&mut s, op)
    };
    {
        let op = frames::set_cel(&s, (l, f1), Some(128), Some(-1)).expect("cel");
        one_step(&mut s, op)
    };
    {
        let op = frames::move_to(&s, f2, 0).expect("move");
        one_step(&mut s, op)
    };
    {
        let op = frames::reverse(&s, 0, 3).expect("reverse");
        one_step(&mut s, op)
    };
    {
        let op = frames::link(&s, l, &[f0, f1]).expect("liga");
        one_step(&mut s, op)
    };
    {
        let op = frames::unlink(&s, (l, f1)).expect("desliga");
        one_step(&mut s, op)
    };
    {
        let op = frames::delete(&s, &[f0]).expect("apaga");
        one_step(&mut s, op)
    };
    {
        let op = frames::delete_tag(&s, 0).expect("apaga tag");
        one_step(&mut s, op)
    };
}

#[test]
fn a_linked_cel_shares_the_image_until_it_is_unlinked() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Linked).expect("ligado");
    apply(&mut s, op).expect("ligado");
    dot(&mut s, l, f1, 1, BLUE);
    assert_eq!(img(&s, f0), img(&s, f1), "pintar o ligado mudou só ele");
    assert_eq!(s.linked_frames((l, f0)), vec![f0, f1]);
    // a opacidade é da imagem (todos), o z é do cel (só ele)
    {
        let op = frames::set_cel(&s, (l, f1), Some(100), Some(2)).expect("cel");
        apply(&mut s, op)
    }
    .expect("cel");
    assert_eq!(s.cel((l, f0)).map(|c| c.opacity), Some(100));
    assert_eq!(
        (s.entry((l, f0)).map(|c| c.z), s.entry((l, f1)).map(|c| c.z)),
        (Some(0), Some(2))
    );
    {
        let op = frames::unlink(&s, (l, f1)).expect("desliga");
        apply(&mut s, op)
    }
    .expect("desliga");
    dot(&mut s, l, f1, 2, BLUE);
    assert_ne!(
        img(&s, f0),
        img(&s, f1),
        "depois de desligar, pintar um não muda o outro"
    );
}

#[test]
fn deleting_the_owner_frame_hands_the_image_to_the_next_linked_cel() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Linked).expect("1");
    apply(&mut s, op).expect("1");
    let (op, f2) = frames::new_frame(&mut s, f1, NewFrame::Linked).expect("2");
    apply(&mut s, op).expect("2");
    let want = img(&s, f0);
    {
        let op = frames::delete(&s, &[f0]).expect("apaga");
        apply(&mut s, op)
    }
    .expect("apaga");
    assert_eq!((img(&s, f1), img(&s, f2)), (want.clone(), want));
    assert_eq!(
        s.entry((l, f1)).and_then(|c| c.link),
        None,
        "o 1.º que fica é o dono"
    );
    assert_eq!(s.entry((l, f2)).and_then(|c| c.link), Some(f1));
}

#[test]
fn erasing_every_cell_of_a_linked_image_keeps_the_link_with_an_empty_image() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Linked).expect("ligado");
    apply(&mut s, op).expect("ligado");
    let before = s.clone();
    let r = Rect::new(0, 0, 1, 1);
    let inv = apply(
        &mut s,
        Op::Patch(Patch {
            layer: l,
            frame: f1,
            bbox: None,
            rect: r,
            pixels: vec![0; 4],
        }),
    )
    .expect("apaga");
    assert!(
        s.cel((l, f0)).is_some_and(|c| c.bbox.is_empty()),
        "a imagem fica vazia"
    );
    assert_eq!(s.entry((l, f1)).and_then(|c| c.link), Some(f0));
    apply(&mut s, inv).expect("desfaz");
    assert_eq!(s, before);
}

#[test]
fn the_format_carries_tags_links_z_cel_opacity_and_continuous_layers() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    {
        let op = layers::edit(&s, l, |x| x.continuous = true).expect("cont");
        apply(&mut s, op)
    }
    .expect("cont");
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Copy).expect("novo");
    apply(&mut s, op).expect("novo");
    assert_eq!(
        s.entry((l, f1)).and_then(|c| c.link),
        Some(f0),
        "continuous: o New Frame liga"
    );
    {
        let op = frames::set_cel(&s, (l, f1), Some(77), Some(-3)).expect("cel");
        apply(&mut s, op)
    }
    .expect("cel");
    {
        let op = frames::new_tag(&s, "idle", 0, 1).expect("tag");
        apply(&mut s, op)
    }
    .expect("tag");
    {
        let op = frames::edit_tag(&s, 0, |t| (t.direction, t.repeat) = (Direction::Reverse, 3))
            .expect("p");
        apply(&mut s, op)
    }
    .expect("p");
    let lib = std::collections::BTreeMap::from([(7u64, s.clone())]);
    let bytes = codec::write_library(&lib);
    let back = codec::read_library(&bytes).expect("lê");
    assert_eq!(back[&7], s);
    assert_eq!(
        codec::write_library(&back),
        bytes,
        "gravar de novo dá os mesmos bytes"
    );
}

#[test]
fn the_document_refuses_broken_links_empty_timelines_and_tags_out_of_range() {
    let mut s = sprite();
    let l = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, l, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Empty).expect("novo");
    apply(&mut s, op).expect("novo");
    let set = |frames: Vec<_>, cels, tags| Op::Frames(Box::new(FrameSet { frames, cels, tags }));
    let fr = s.frames().to_vec();
    let before = s.clone();
    let bad = [
        set(
            fr.clone(),
            vec![((l, f1), Some(Cel::linked(FrameId(999), 0)))],
            Vec::new(),
        ),
        set(
            fr.clone(),
            vec![((l, f1), Some(Cel::linked(f1, 0)))],
            Vec::new(),
        ),
        set(
            fr.clone(),
            vec![((l, f1), Some(Cel::image(Rect::default(), Vec::new())))],
            Vec::new(),
        ),
        set(Vec::new(), Vec::new(), Vec::new()),
        set(
            fr.clone(),
            Vec::new(),
            vec![Tag {
                name: "x".into(),
                from: 0,
                to: 2,
                direction: Direction::Forward,
                repeat: 0,
                color: [0; 4],
            }],
        ),
        // o frame sai e o cel dele fica órfão
        set(vec![fr[1]], Vec::new(), Vec::new()),
    ];
    for op in bad {
        let e = apply(&mut s, op.clone()).expect_err(&format!("{op:?}"));
        assert!(
            matches!(e, OpError::BadCel(_) | OpError::BadFrames(_)),
            "{e:?}"
        );
        assert_eq!(s, before, "recusar não muda nada");
    }
}

#[test]
fn merge_down_keeps_the_image_of_a_bottom_link_in_frames_the_top_does_not_touch() {
    let mut s = sprite();
    let b = s.layers()[0].id;
    let f0 = s.frames()[0].id;
    dot(&mut s, b, f0, 0, RED);
    let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Linked).expect("ligado");
    apply(&mut s, op).expect("ligado");
    let (op, top) = layers::new_layer(&mut s, b).expect("cima");
    apply(&mut s, op).expect("cima");
    dot(&mut s, top, f0, 1, BLUE);
    let want_f1 = img(&s, f1);
    {
        let op = layers::merge_down(&s, top).expect("merge");
        apply(&mut s, op)
    }
    .expect("merge");
    assert_eq!(
        img(&s, f1),
        want_f1,
        "o frame sem cel de cima ficou com a imagem de antes"
    );
}
