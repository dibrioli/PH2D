//! **A lista de pincéis no computador** (`brushes.txt`): ida e volta exactas, texto canónico,
//! recusa com a linha, e o padrão em RGBA que vale em sprites de qualquer modo de cor.

use ph2d_pixel_core::brush::BrushShape;
use ph2d_pixel_core::ink::Ink;
use ph2d_pixel_core::pattern::PatternBrush;
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, Sprite, apply};
use ph2d_pixel_io::brush_file::{BrushKind, BrushParams, BrushSlot, read, write};

fn sprite(mode: ColorMode) -> Sprite {
    Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn padrao() -> BrushSlot {
    BrushSlot {
        locked: false,
        params: BrushParams::NEW_BRUSH,
        kind: BrushKind::Pattern {
            size: (3, 2),
            cells: vec![
                Some(Some([255, 0, 0, 255])),
                Some(None),
                None,
                Some(Some([0, 0, 255, 128])),
                Some(Some([0, 160, 0, 255])),
                Some(None),
            ],
            origin: (8, -2),
        },
        size: 4,
        angle: -30,
        image_color: true,
        ink: Ink::LockAlpha,
        fg: [1, 2, 3, 255],
        bg: [0, 0, 0, 0],
        pixel_perfect: true,
    }
}

fn forma() -> BrushSlot {
    BrushSlot {
        locked: true,
        params: BrushParams::SAVE_HERE,
        kind: BrushKind::Shape(BrushShape::Square),
        size: 5,
        angle: 30,
        image_color: false,
        ink: Ink::Shading,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 255],
        pixel_perfect: false,
    }
}

#[test]
fn a_list_round_trips_exactly_and_the_text_is_canonical() {
    let l = vec![
        padrao(),
        forma(),
        BrushSlot {
            params: BrushParams::default(),
            ..forma()
        },
    ];
    let t = write(&l);
    assert_eq!(read(&t).expect("lê"), l);
    assert_eq!(
        write(&read(&t).expect("lê")),
        t,
        "escrever o lido dá o mesmo texto"
    );
    assert!(t.starts_with("ph2d-pixel-brushes 1\n"));
    assert!(t.contains("row ff0000ff - .\n"), "{t}");
    assert!(t.contains("\nparams\n"), "params vazios: a linha fica");
    assert_eq!(read("ph2d-pixel-brushes 1\n").expect("vazia"), vec![]);
    let crlf = t.replace('\n', "\r\n");
    assert_eq!(read(&crlf).expect("CRLF"), l);
}

#[test]
fn a_malformed_file_is_refused_with_its_line() {
    let t = write(&[padrao()]);
    let linhas: Vec<&str> = t.lines().collect();
    let troca = |i: usize, por: &str| {
        let mut v = linhas.clone();
        v[i] = por;
        v.join("\n")
    };
    let linha_de = |k: &str| linhas.iter().position(|l| l.starts_with(k)).expect(k);
    for (i, por) in [
        (0, "outra coisa 1"),
        (linha_de("row"), "row ff0000ff -"),
        (linha_de("ink"), "ink acrylic"),
        (linha_de("size"), "size -3"),
        (linha_de("locked"), "locked 2"),
        (linha_de("params"), "params colour"),
        (linha_de("fg"), "fg nothex"),
    ] {
        let e = read(&troca(i, por)).expect_err(por);
        assert_eq!(e.line, i + 1, "{por}: {e:?}");
    }
    assert!(read("").is_err());
    assert!(read(&t.replace("end\n", "")).is_err(), "sem end");
    let menos = t.replacen("row 0000ff80 00a000ff -\n", "", 1);
    assert!(read(&menos).is_err(), "linhas a menos");
}

/// O padrão guardado em RGBA volta no modo do sprite que o recebe: RGBA tal e qual, Indexed pela
/// cor mais próxima da paleta; a máscara e a origem viajam.
#[test]
fn a_saved_pattern_comes_back_in_any_colour_mode() {
    let mut s = sprite(ColorMode::Rgba);
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = Rect::new(2, 2, 2, 1);
    apply(
        &mut s,
        Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(r),
            rect: r,
            pixels: [[255, 0, 0, 255], [0, 0, 255, 255]].concat(),
        }),
    )
    .expect("patch");
    let p =
        PatternBrush::from_selection(&s, (l, f), &Mask::ellipse(8, 8, (1, 1), (4, 3))).expect("p");
    let cells = p.to_rgba(&s);
    let volta = PatternBrush::from_rgba(&s, p.size(), &cells, p.origin).expect("volta");
    assert_eq!(volta, p, "RGBA: o mesmo pincel");
    let ix = sprite(ColorMode::Indexed);
    let q = PatternBrush::from_rgba(&ix, p.size(), &cells, p.origin).expect("indexed");
    assert_eq!(q.offsets(), p.offsets(), "a máscara viaja");
    let cor = |b: &PatternBrush, s: &Sprite| -> Vec<Option<[u8; 4]>> {
        b.to_rgba(s).into_iter().flatten().collect()
    };
    let perto: Vec<_> = cor(&p, &s)
        .into_iter()
        .map(|c| c.map(|c| ix.rgba_of_value(ix.value_of_rgba(c))))
        .collect();
    assert_eq!(cor(&q, &ix), perto, "Indexed: a cor mais próxima da paleta");
    assert!(
        PatternBrush::from_rgba(&s, (3, 3), &cells, (0, 0)).is_none(),
        "medidas"
    );
    assert!(
        PatternBrush::from_rgba(&s, (1, 1), &[None], (0, 0)).is_none(),
        "máscara vazia"
    );
}
