//! O que a facilitação guarda no documento (W7, `docs/MiroClone/05_facilitacao.md` §2 V4 e S1–S3).

use crate::{
    BoardOp, BoardSet, Element, Rgba, RichText, Shape, ShapeType, Stamp, StampCount, Style,
};

fn note(set: &mut BoardSet) -> (crate::BoardId, Element) {
    let a = set.create("Retro".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([30, 30, 30, 255]);
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        Shape {
            kind: ShapeType::Sticky,
            style: Style::new(Some(Rgba([255, 249, 177, 255])), None, ink),
            text: RichText::plain("ideia"),
        },
        [0.0, 0.0, 199.0, 199.0],
    );
    (a, el)
}

/// ⭐ S1/S3: um carimbo nasce no fim com a contagem, soma, e a 0 SAI (a ficha desaparece).
#[test]
fn a_stamp_counts_and_leaves_at_zero() {
    let (_, mut el) = note(&mut BoardSet::default());
    assert!(el.add_stamp(Stamp::Heart, 1));
    assert!(el.add_stamp(Stamp::Star, 1));
    assert!(el.add_stamp(Stamp::Heart, 2));
    assert_eq!(
        el.stamps(),
        [
            StampCount {
                stamp: Stamp::Heart,
                count: 3
            },
            StampCount {
                stamp: Stamp::Star,
                count: 1
            }
        ]
    );
    assert!(el.add_stamp(Stamp::Star, -1));
    assert_eq!(el.stamps().len(), 1, "a 0 a ficha sai");
    assert!(
        !el.add_stamp(Stamp::Flame, -1),
        "tirar um que não há não mexe"
    );
    assert!(el.org.as_ref().is_some_and(|o| o.stamps.len() == 1));
}

/// ⭐ S1: os carimbos gravam-se e lêem-se (os votos são da equipa desde a W13: `poll_tests.rs`).
#[test]
fn stamps_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let (a, mut el) = note(&mut set);
    el.add_stamp(Stamp::ThumbsUp, 2);
    el.add_stamp(Stamp::Party, 1);
    BoardOp::Put(el).apply(&mut set.get_mut(a).unwrap().doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

/// ⭐ Um ficheiro do formato 6 (W6) continua a abrir — gravado pelo build de `6a9b47240` (o último do
/// formato 6, um teste temporário que escreveu `to_bytes`): um frame, uma nota dentro dele com
/// etiqueta, e um mapa (a raiz e um filho recolhido). Nada tem votos nem carimbos.
#[test]
fn a_format_6_file_still_opens_with_no_votes_or_stamps() {
    let set = BoardSet::from_bytes(V6_BYTES).expect("o formato 6 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Mind6");
    assert_eq!(b.camera.zoom, 0.5);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 4);
    assert!(els[0].as_frame().is_some());
    let note = els[1];
    assert_eq!(note.frame, Some(els[0].id));
    assert_eq!(note.tags()[0].name, "urgente");
    assert_eq!(els[2].shape().unwrap().text.as_str(), "Plan");
    assert!(els[2].mind().is_some_and(|m| m.parent.is_none()));
    let child = els[3].mind().expect("o filho é nó");
    assert!(child.collapsed && child.parent == Some(els[2].id));
    assert!(els.iter().all(|e| e.stamps().is_empty()));
    assert!(b.team.polls.is_empty(), "sem votos, sem sessão");
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 6 (`6a9b47240`).
const V6_BYTES: &[u8] = include_bytes!("../fixtures/format_v6.bin");

/// ⭐ V2/V5: o `+` e o selo são os medidos do Miro — o `+` 0,21 do lado com o centro NO canto; o selo
/// de 0,22 (o menor total) a 0,34 (o maior), a 0,03 do canto para fora; um só total = o maior.
#[test]
fn the_badge_grows_with_the_count() {
    let (_, el) = note(&mut BoardSet::default());
    let ([cx, cy], d) = crate::vote_plus(&el);
    assert_eq!([cx, cy], [199.0, 199.0]);
    assert!(
        (d - 49.0 / 236.0 * 199.0).abs() < 1.0,
        "Ø {d}: o `vote_03` dá 49 em 236"
    );
    let small = crate::vote_badge(&el, 1, [1, 2]).1;
    let big = crate::vote_badge(&el, 2, [1, 2]).1;
    assert!(
        (small - 52.0 / 236.0 * 199.0).abs() < 1.0,
        "1 voto: {small}"
    );
    assert!((big - 80.0 / 236.0 * 199.0).abs() < 1.0, "2 votos: {big}");
    assert!(
        (crate::vote_badge(&el, 7, [7, 7]).1 - big).abs() < 1e-9,
        "um só total = o maior"
    );
    let ([bx, by], _) = crate::vote_badge(&el, 2, [1, 2]);
    assert!((bx - 199.0 - 6.0 / 236.0 * 199.0).abs() < 1.0 && (by - bx).abs() < 1e-9);
}

/// S3: as fichas nascem por baixo da nota, da esquerda, e partem em filas da largura dela; a ficha
/// alarga com os algarismos.
#[test]
fn stamp_chips_wrap_under_the_note() {
    let (_, mut el) = note(&mut BoardSet::default());
    for s in Stamp::ALL {
        el.add_stamp(s, 1);
    }
    el.add_stamp(Stamp::Heart, 99);
    let chips = crate::stamp_chips(&el);
    assert_eq!(chips.len(), 8);
    assert!(
        chips
            .iter()
            .all(|(_, r)| r[1] > el.h && r[0] >= 0.0 && r[2] <= el.w)
    );
    assert!(
        chips[0].1[0] == 0.0 && chips.iter().any(|(_, r)| r[1] > chips[0].1[3]),
        "parte em filas"
    );
    let heart = chips
        .iter()
        .find(|(c, _)| c.stamp == Stamp::Heart)
        .unwrap()
        .1;
    let one = chips[0].1;
    assert!(
        heart[2] - heart[0] > one[2] - one[0],
        "100 é mais largo que 1"
    );
}
