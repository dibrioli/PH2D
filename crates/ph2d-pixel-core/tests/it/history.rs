//! O tecto do histórico é em bytes: o mais velho cai, o mais novo fica sempre.

use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Paint, Sprite, Stroke};

#[test]
fn the_byte_cap_drops_the_oldest_step_and_keeps_the_newest() {
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut h = History::with_cap(2_000);
    for y in 0..20 {
        let mut st = Stroke::begin(&s, l, f, 0xff00_00ff + y as u32, Paint::default(), (0, y));
        st.to(&s, 63, y);
        {
            let op = st.op(&s).expect("op");
            h.commit(&mut s, op)
        }
        .expect("commit");
        assert!(
            h.bytes() <= 2_000 || h.undo_len() == 1,
            "passo {y}: {} bytes",
            h.bytes()
        );
    }
    assert!(h.undo_len() < 20, "o tecto não cortou nada");
    assert!(h.undo_len() >= 1);
}

// ---- P12 H: a lista com nomes (Aseprite O11–O14, R2–R3) ----

use ph2d_pixel_core::layers;
use ph2d_pixel_core::{Label, PrettySize, StepState, Touched};

const PENCIL: Label = Label::new("pixel.history.pencil");
const LINE: Label = Label::new("pixel.history.line");
const NEW_LAYER: Label = Label::new("pixel.history.new_layer");

fn sprite() -> Sprite {
    Sprite::new(NewSprite {
        width: 32,
        height: 32,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn traco(h: &mut History, s: &mut Sprite, y: i32, cor: u32, label: Label) {
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::begin(s, l, f, cor, Paint::default(), (0, y));
    st.to(s, 31, y);
    let op = st.op(s).expect("op");
    h.commit_labeled(s, op, label).expect("commit");
}

fn camada(h: &mut History, s: &mut Sprite) {
    let top = s.layers()[0].id;
    let (op, _) = layers::new_layer(s, top).expect("layer");
    h.commit_labeled(s, op, NEW_LAYER).expect("commit");
}

fn nomes(h: &History) -> Vec<(&'static str, StepState)> {
    h.entries().map(|(l, st)| (l.key(), st)).collect()
}

#[test]
fn history_labels_follow_undo_and_redo() {
    let mut s = sprite();
    let mut h = History::default();
    assert_eq!(nomes(&h), vec![(Label::INITIAL.key(), StepState::Done)]);
    traco(&mut h, &mut s, 1, 0xff00_00ff, PENCIL);
    traco(&mut h, &mut s, 2, 0xff00_ff00, LINE);
    camada(&mut h, &mut s);
    assert_eq!(h.current(), 3);
    assert!(h.undo(&mut s));
    assert!(h.undo(&mut s));
    use StepState::{Done, Undone};
    assert_eq!(
        nomes(&h),
        vec![
            (Label::INITIAL.key(), Done),
            (PENCIL.key(), Done),
            (LINE.key(), Undone),
            (NEW_LAYER.key(), Undone),
        ]
    );
    assert_eq!(h.current(), 1);
    assert!(h.redo(&mut s));
    assert_eq!(nomes(&h)[2], (LINE.key(), Done));
    assert_eq!(nomes(&h)[3], (NEW_LAYER.key(), Undone));
    // O nome por omissão do `commit` é a família do op.
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::begin(&s, l, f, 1, Paint::default(), (0, 9));
    st.to(&s, 3, 9);
    let op = st.op(&s).expect("op");
    h.commit(&mut s, op).expect("commit");
    assert_eq!(
        h.entries().last().map(|e| e.0.key()),
        Some("pixel.history.draw")
    );
}

#[test]
fn history_touched_names_the_cel_of_a_patch() {
    let mut s = sprite();
    let mut h = History::default();
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    traco(&mut h, &mut s, 1, 0xff00_00ff, PENCIL);
    camada(&mut h, &mut s);
    assert_eq!(h.touched(0), None);
    assert_eq!(h.touched(1), Some(Touched { layer: l, frame: f }));
    assert_eq!(h.touched(2), None);
    assert!(h.undo(&mut s) && h.undo(&mut s));
    // Desfeito, o passo continua a dizer o cel.
    assert_eq!(h.touched(1), Some(Touched { layer: l, frame: f }));
    assert_eq!(h.touched(3), None);
}

/// Propriedade sobre sequências pseudo-aleatórias (LCG, sem `rand`): `go_to(n)` = os mesmos passos
/// de `undo`/`redo` um a um — o mesmo sprite, a mesma lista, os mesmos bytes.
#[test]
fn history_go_to_equals_n_single_steps() {
    let mut seed: u64 = 0x5eed_1234;
    let mut next = |m: u64| {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) % m
    };
    for caso in 0..24 {
        let mut s = sprite();
        let mut h = History::default();
        for i in 0..(3 + next(6)) {
            if next(4) == 0 {
                camada(&mut h, &mut s);
            } else {
                traco(
                    &mut h,
                    &mut s,
                    (next(32)) as i32,
                    0xff00_0000 | next(0xff_ffff) as u32,
                    PENCIL,
                );
            }
            if next(3) == 0 {
                let _ = h.undo(&mut s);
            }
            let _ = i;
        }
        for _ in 0..6 {
            let n = next(h.len() as u64 + 1) as usize;
            let (mut s1, mut h1) = (s.clone(), h.clone());
            let ok = h.go_to(&mut s, n);
            assert_eq!(ok, n < h1.len(), "caso {caso}: go_to({n})");
            if ok {
                while h1.current() > n {
                    assert!(h1.undo(&mut s1));
                }
                while h1.current() < n {
                    assert!(h1.redo(&mut s1));
                }
                assert_eq!(h.current(), n);
            }
            assert_eq!(s, s1, "caso {caso}: go_to({n}) ≠ passos");
            assert_eq!(h, h1, "caso {caso}: a história diverge");
        }
    }
}

#[test]
fn history_new_commit_after_going_back_truncates() {
    let mut s = sprite();
    let mut h = History::default();
    for y in 0..4 {
        traco(&mut h, &mut s, y, 0xff00_00ff, PENCIL);
    }
    assert!(h.go_to(&mut s, 2));
    traco(&mut h, &mut s, 9, 0xff00_ff00, LINE);
    use StepState::Done;
    assert_eq!(
        nomes(&h),
        vec![
            (Label::INITIAL.key(), Done),
            (PENCIL.key(), Done),
            (PENCIL.key(), Done),
            (LINE.key(), Done),
        ]
    );
    assert!(!h.can_redo());
    assert_eq!(h.current(), 3);
}

#[test]
fn history_cap_drops_oldest_with_its_label_and_keeps_newest() {
    let mut s = sprite();
    let mut h = History::with_cap(600);
    traco(&mut h, &mut s, 0, 0xff00_00ff, PENCIL);
    for y in 1..12 {
        traco(&mut h, &mut s, y, 0xff00_ff00 + y as u32, LINE);
    }
    let n = nomes(&h);
    assert_eq!(n[0].0, Label::INITIAL.key());
    assert!(n.len() < 13, "o tecto não cortou");
    assert!(
        n.iter().all(|e| e.0 != PENCIL.key()),
        "o mais velho ficou: {n:?}"
    );
    assert_eq!(n.last().map(|e| e.0), Some(LINE.key()));
    // Um só passo maior que o tecto fica (R3: o mais novo fica sempre).
    let mut h = History::with_cap(1);
    traco(&mut h, &mut s, 3, 0xff12_3456, PENCIL);
    traco(&mut h, &mut s, 4, 0xff65_4321, LINE);
    assert_eq!(
        nomes(&h),
        vec![
            (Label::INITIAL.key(), StepState::Done),
            (LINE.key(), StepState::Done)
        ]
    );
    assert!(h.bytes() > 1);
}

#[test]
fn history_pretty_size_is_asepritelike() {
    assert_eq!(PrettySize::of(0), PrettySize::Bytes(0));
    assert_eq!(PrettySize::of(999), PrettySize::Bytes(999));
    assert_eq!(PrettySize::of(1000), PrettySize::K(1000.0 / 1024.0));
    assert_eq!(PrettySize::of(5530), PrettySize::K(5530.0 / 1024.0));
    assert_eq!(PrettySize::of(999_999), PrettySize::K(999_999.0 / 1024.0));
    assert_eq!(PrettySize::of(1_572_864), PrettySize::M(1.5));
}
