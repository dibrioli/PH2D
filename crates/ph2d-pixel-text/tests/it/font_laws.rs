//! Os vetores de `docs/Pixel Art/app/tests/font.test.js` (W27), portados um a um.

use std::collections::BTreeMap;

use ph2d_pixel_text::{
    Align, FontChoice, Glyph, LayoutOpts, PixelFont, Rendered, TextStyle, builtin, caret_at,
    charset_to_unicode, decode_text, layout_text, parse_bdf, parse_fnt, parse_hex, rasterize,
    read_font_file, render_text, text_lines,
};

fn g(rows: &[&str], left: i32, top: i32, adv: i32) -> Glyph {
    let h = rows.len() as u32;
    let w = rows.first().map_or(0, |r| r.len()) as u32;
    let bits = rows
        .iter()
        .flat_map(|r| r.bytes().map(|c| u8::from(c == b'#')))
        .collect();
    Glyph::new(w, h, left, top, adv, bits)
}

fn mini_with(default_cp: Option<u32>) -> PixelFont {
    let glyphs = BTreeMap::from([
        (65, g(&[".#.", "#.#", "###", "#.#", "#.#"], 0, 5, 4)),
        (
            106,
            g(&["..#", "...", "..#", "..#", "#.#", ".#."], -1, 5, 3),
        ),
        (32, g(&[], 0, 0, 3)),
        (63, g(&["##.", "..#", ".#.", "...", ".#."], 0, 5, 4)),
        (0xe3, g(&["##", "..", "##", "##"], 0, 6, 3)),
    ]);
    PixelFont::new(
        "Mini".into(),
        String::new(),
        "teste",
        5,
        1,
        0,
        glyphs,
        default_cp,
        Vec::new(),
    )
}

fn mini() -> PixelFont {
    mini_with(Some(63))
}

fn opts(align: Align, tracking: i32, leading: i32) -> LayoutOpts {
    LayoutOpts {
        align,
        tracking,
        leading,
    }
}

fn arte(r: &Rendered) -> String {
    (0..r.mh)
        .map(|y| {
            (0..r.mw)
                .map(|x| {
                    if r.mask[(y * r.mw + x) as usize] != 0 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("|")
}

#[test]
fn layout_pen_walks_each_advance() {
    let f = mini();
    let l = layout_text(&f, "AA", LayoutOpts::default());
    assert_eq!(
        l.glyphs.iter().map(|g| (g.x, g.y)).collect::<Vec<_>>(),
        vec![(0, 0), (4, 0)]
    );
    assert_eq!((l.width, l.height), (8, 6));
    assert_eq!(l.ink, Some((0, 0, 7, 5)));
}

#[test]
fn layout_tracking_never_walks_back() {
    let f = mini();
    assert_eq!(
        layout_text(&f, "AA", opts(Align::Left, 2, 0)).glyphs[1].x,
        6
    );
    let junto = layout_text(&f, "AA", opts(Align::Left, -10, 0));
    assert_eq!(
        junto.glyphs.iter().map(|g| g.x).collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(junto.width, 4);
    assert_eq!(f.spacing_floor(), (-4, -5));
}

#[test]
fn layout_align_inside_widest_row() {
    let f = mini();
    let off = |a, t: &str| {
        layout_text(&f, t, opts(a, 0, 0))
            .rows
            .iter()
            .map(|r| r.offset)
            .collect::<Vec<_>>()
    };
    assert_eq!(off(Align::Left, "A\nAAA"), vec![0, 0]);
    assert_eq!(off(Align::Center, "A\nAAA"), vec![4, 0]);
    assert_eq!(off(Align::Right, "A\nAAA"), vec![8, 0]);
    assert_eq!(
        off(Align::Center, "A\nAA "),
        vec![3, 0],
        "sobra ímpar cai para a esquerda"
    );
}

#[test]
fn layout_leading_and_minimum_step() {
    let f = mini();
    let l = layout_text(&f, "A\nA", opts(Align::Left, 0, 3));
    assert_eq!((l.rows[1].top, l.height), (9, 15));
    assert_eq!(layout_text(&f, "A\nA", opts(Align::Left, 0, -100)).step, 1);
    assert_eq!(text_lines("A\r\nA\rA").len(), 3);
}

#[test]
fn render_ink_outside_box_enters_mask() {
    let f = mini();
    let r = render_text(&f, "jA", LayoutOpts::default());
    assert_eq!((r.mx, r.mw, r.mh), (-1, 7, 6));
    assert_eq!(arte(&r).split('|').next(), Some("..#..#."));
    assert_eq!(render_text(&f, "ã", LayoutOpts::default()).my, -1);
}

#[test]
fn layout_tab_and_control_chars() {
    let f = mini();
    assert_eq!(
        layout_text(&f, "A\tA", LayoutOpts::default()).glyphs[1].x,
        16
    );
    assert_eq!(
        layout_text(&f, "\u{7}A", LayoutOpts::default()).glyphs[0].x,
        0
    );
}

#[test]
fn missing_letter_uses_font_default_and_is_listed() {
    let f = mini();
    let l = layout_text(&f, "ZZY", LayoutOpts::default());
    assert_eq!(l.missing, vec![89, 90]);
    assert_eq!(l.width, 12);
    assert_eq!(
        render_text(&f, "Z", LayoutOpts::default()).mask,
        render_text(&f, "?", LayoutOpts::default()).mask
    );
}

#[test]
fn missing_letter_without_default_is_hollow_box() {
    let f = mini_with(None);
    assert_eq!(
        arte(&render_text(&f, "Z", LayoutOpts::default())),
        "###|#.#|#.#|#.#|###"
    );
    assert_eq!(f.fallback_glyph().adv, 4);
}

#[test]
fn nfc_and_astral_codepoints() {
    let f = mini();
    assert!(
        layout_text(&f, "a\u{303}", LayoutOpts::default())
            .missing
            .is_empty()
    );
    assert_eq!(
        render_text(&f, "a\u{303}", LayoutOpts::default()).mask,
        render_text(&f, "ã", LayoutOpts::default()).mask
    );
    assert_eq!(
        layout_text(&f, "\u{1F600}", LayoutOpts::default()).missing,
        vec![0x1f600]
    );
}

#[test]
fn caret_where_next_letter_enters() {
    let f = mini();
    let d = LayoutOpts::default();
    assert_eq!(caret_at(&f, "AA\nA", 2, d), (8, 0, 6));
    assert_eq!(caret_at(&f, "AA\nA", 3, d), (0, 6, 6));
    assert_eq!(caret_at(&f, "AA\nA", 4, d).0, 4);
    assert_eq!(caret_at(&f, "AA\nA", 4, opts(Align::Center, 0, 0)).0, 6);
    assert_eq!(caret_at(&f, "AA", 2, opts(Align::Left, 2, 0)).0, 12);
    assert_eq!(caret_at(&f, "", 0, d).0, 0);
}

#[test]
fn label_family_style_height() {
    let mk = |style: &str| {
        PixelFont::new(
            "Cozette".into(),
            style.into(),
            "t",
            10,
            3,
            0,
            BTreeMap::new(),
            None,
            Vec::new(),
        )
    };
    assert_eq!(mk("Medium").label(), "Cozette · 13px");
    assert_eq!(mk("Bold").label(), "Cozette Bold · 13px");
}

const BDF: &str = "STARTFONT 2.1
COMMENT teste
FONT -teste-Mini-Medium-R-Normal--10-100-75-75-P-50-ISO10646-1
SIZE 10 75 75
FONTBOUNDINGBOX 5 7 0 -2
STARTPROPERTIES 3
FAMILY_NAME \"Mini \"\"aspas\"\"\"
DEFAULT_CHAR 65
CHARSET_REGISTRY \"ISO10646\"
ENDPROPERTIES
CHARS 5
STARTCHAR A
ENCODING 65
SWIDTH 500 0
DWIDTH 6 0
BBX 3 2 1 0
BITMAP
A0000
40
ENDCHAR
STARTCHAR B_so_swidth
ENCODING 66
SWIDTH 480 0
BBX 2 1 0 0
BITMAP
C
ENDCHAR
STARTCHAR C_sem_largura
ENCODING 67
BBX 2 1 1 0
BITMAP
8
ENDCHAR
STARTCHAR sem_codigo
ENCODING -1
BBX 1 1 0 0
BITMAP
80
ENDCHAR
STARTCHAR D_caixa_da_fonte
ENCODING 68
DWIDTH 4 0
BITMAP
F8
F8
F8
F8
F8
F8
F8
ENDCHAR
ENDFONT
";

#[test]
fn bdf_box_position_advance_and_padding() {
    let f = parse_bdf(BDF, "").unwrap();
    let a = &f.glyphs[&65];
    assert_eq!((a.w, a.h, a.left, a.top, a.adv), (3, 2, 1, 2, 6));
    assert_eq!(a.bits, vec![1, 0, 1, 0, 1, 0]);
    assert_eq!(f.family, "Mini \"aspas\"");
    assert_eq!((f.ascent, f.descent), (5, 2));
    assert_eq!(f.default_cp, Some(65));
}

#[test]
fn bdf_no_dwidth_is_box_width() {
    let f = parse_bdf(BDF, "").unwrap();
    assert_eq!(f.glyphs[&66].adv, 2);
    assert_eq!(f.glyphs[&67].adv, 2);
    let d = &f.glyphs[&68];
    assert_eq!((d.w, d.h, d.top), (5, 7, 5));
    assert_eq!(f.glyphs.len(), 4);
    assert!(
        f.notes.iter().any(|n| n.contains("sem código")),
        "{:?}",
        f.notes
    );
}

#[test]
fn bdf_font_level_dwidth() {
    let f = parse_bdf(&BDF.replace("CHARS 5", "DWIDTH 9 0\nCHARS 5"), "").unwrap();
    assert_eq!((f.glyphs[&67].adv, f.glyphs[&65].adv), (9, 6));
}

#[test]
fn bdf_refusals_name_line_and_reason() {
    let erro = |t: &str| parse_bdf(t, "").err().unwrap_or_default();
    let e = erro(&BDF.replace("A0000", "G0"));
    assert!(
        e.contains("linha 18: \"G\" não é um dígito hexadecimal"),
        "{e}"
    );
    assert!(erro(&BDF.replace("40\nENDCHAR", "40")).contains("glifo sem ENDCHAR"));
    assert!(erro("olá\n").contains("começa com STARTFONT"));
    assert!(erro(&BDF.replace("BBX 3 2 1 0", "BBX -3 2 1 0")).contains("caixa negativa"));
    assert!(erro(&BDF.replace("FONTBOUNDINGBOX 5 7 0 -2\n", "")).contains("sem FONTBOUNDINGBOX"));
}

#[test]
fn hex_unifont_widths_and_baseline() {
    let t = format!(
        "# um comentário\n0041:0000000018242442427E424242420000\n\n00E9:00000C3000003C42427E4040423C0000\n3041:{}0400020013800E00048007E00D10161026102A2010C00000",
        "0".repeat(16)
    );
    let f = parse_hex(&t, "uni.hex").unwrap();
    let (a, hira) = (&f.glyphs[&0x41], &f.glyphs[&0x3041]);
    assert_eq!((a.w, a.h, hira.w, hira.h), (8, 16, 16, 16));
    assert_eq!((f.ascent, f.descent), (14, 2));
    assert_eq!(a.bits[32..40], [0, 0, 0, 1, 1, 0, 0, 0]);
    assert_eq!(f.family, "uni");
}

#[test]
fn hex_unscii_8_and_wide_exception() {
    let f = parse_hex("00041:183C66667E666600\n00042:7C66667C66667C00\n00043:3C66606060663C00\n1F600:0000000000000000FFFFFFFFFFFFFFFF", "").unwrap();
    assert_eq!((f.glyphs[&0x41].w, f.glyphs[&0x41].h), (8, 8));
    assert_eq!((f.glyphs[&0x1f600].w, f.glyphs[&0x1f600].h), (16, 8));
    assert_eq!((f.ascent, f.descent), (8, 0));
}

#[test]
fn hex_refusals_name_line() {
    let erro = |t: &str| parse_hex(t, "").err().unwrap_or_default();
    let oito = "0041:0000000018242442427E424242420000\n0042:0000000018242442427E424242420000\n";
    assert!(erro(&format!("{oito}olá")).contains("linha 3: esperado CÓDIGO:BITS"));
    assert!(
        erro(&format!("{oito}0043:0123456789"))
            .contains("linha 3: 10 dígitos não fecham uma altura de 16")
    );
    assert!(erro("# só comentário\n").contains("nenhuma letra"));
    let f = parse_hex(&format!("{oito}0041:0000000018242442427E424242420000"), "").unwrap();
    assert!(f.notes.iter().any(|n| n.contains("repetido")));
}

struct FntG {
    w: usize,
    rows: [&'static str; 2],
}

const DOIS: [FntG; 2] = [
    FntG {
        w: 2,
        rows: ["#.", ".#"],
    },
    FntG {
        w: 3,
        rows: ["###", "..."],
    },
];

/// O `.fnt` mínimo do teste do MVP (formato público "Windows 3.0 font file").
fn fnt(glyphs: &[FntG], ty: u16, version: u16, flags: u32) -> Vec<u8> {
    let (first, height, ascent, charset, ext_leading, default_rel) =
        (0x80u8, 2usize, 2u16, 0u8, 1u16, 1u8);
    let v3 = version == 0x300;
    let (hdr, entry) = if v3 { (148, 6) } else { (118, 4) };
    let last = first + glyphs.len() as u8 - 1;
    let table_end = hdr + (glyphs.len() + 1) * entry;
    let mut off = table_end;
    let mut blocos = Vec::new();
    for gl in glyphs {
        let row = (gl.w + 7) >> 3;
        let mut bytes = vec![0u8; row * height];
        for (y, r) in gl.rows.iter().enumerate() {
            for (x, c) in r.bytes().enumerate().take(gl.w) {
                if c == b'#' {
                    bytes[y * row + (x >> 3)] |= 0x80 >> (x & 7);
                }
            }
        }
        blocos.push((gl.w, off, bytes.clone()));
        off += bytes.len();
    }
    let face = b"Mini\0";
    let mut b = vec![0u8; off + face.len()];
    let len = b.len() as u32;
    let put16 = |b: &mut Vec<u8>, o: usize, v: u16| b[o..o + 2].copy_from_slice(&v.to_le_bytes());
    let put32 = |b: &mut Vec<u8>, o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    put16(&mut b, 0, version);
    put32(&mut b, 2, len);
    put16(&mut b, 66, ty);
    put16(&mut b, 74, ascent);
    put16(&mut b, 78, ext_leading);
    put16(&mut b, 83, 400);
    b[85] = charset;
    put16(&mut b, 88, height as u16);
    b[95] = first;
    b[96] = last;
    b[97] = default_rel;
    put32(&mut b, 105, off as u32);
    put32(&mut b, 113, table_end as u32);
    if v3 {
        put32(&mut b, 118, flags);
    }
    for (k, (w, o, bytes)) in blocos.iter().enumerate() {
        put16(&mut b, hdr + k * entry, *w as u16);
        if v3 {
            put32(&mut b, hdr + k * entry + 2, *o as u32);
        } else {
            put16(&mut b, hdr + k * entry + 2, *o as u16);
        }
        b[*o..*o + bytes.len()].copy_from_slice(bytes);
    }
    b[off..].copy_from_slice(face);
    b
}

#[test]
fn fnt_charset_translation_and_default() {
    let bytes = fnt(&DOIS, 0, 0x200, 0);
    let f = parse_fnt(&bytes, "", 0, bytes.len()).unwrap();
    assert_eq!(f.glyphs[&0x20ac].bits, vec![1, 0, 0, 1]);
    assert_eq!(f.glyphs[&0x81].w, 3);
    assert_eq!(f.default_cp, Some(0x81));
    assert_eq!((f.ascent, f.descent, f.line_gap), (2, 0, 1));
    assert_eq!(f.family, "Mini");
    assert_eq!(f.codes, vec![(0x80, 0x20ac), (0x81, 0x81)]);
    let b3 = fnt(&DOIS, 0, 0x300, 0);
    let f3 = parse_fnt(&b3, "", 0, b3.len()).unwrap();
    assert_eq!(
        (f3.glyphs[&0x20ac].bits.clone(), f3.format),
        (vec![1, 0, 0, 1], "fnt3")
    );
}

#[test]
fn fnt_refusals_with_reason() {
    let erro = |b: &[u8]| read_font_file("x.fnt", b).err().unwrap_or_default();
    assert!(erro(&fnt(&DOIS, 1, 0x200, 0)).contains("VETORIAL"));
    assert!(erro(&fnt(&DOIS, 0, 0x300, 0x10)).contains("COLORIDA"));
    let mut velha = fnt(&DOIS, 0, 0x200, 0);
    velha[1] = 1;
    let e = erro(&velha);
    assert!(
        e.contains("só a 2.0 e a 3.0") || e.contains("formato não reconhecido"),
        "{e}"
    );
    let e = erro(&fnt(&DOIS, 0, 0x200, 0)[..128]);
    assert!(e.contains("fora do recurso"), "{e}");
}

#[test]
fn fon_neither_ne_nor_pe_and_empty_ne() {
    let erro = |b: &[u8]| read_font_file("x.fon", b).err().unwrap_or_default();
    let mut mz = vec![0u8; 0x100];
    mz[0] = 0x4d;
    mz[1] = 0x5a;
    mz[0x3c] = 0x40;
    mz[0x40] = 0x58;
    assert!(erro(&mz).contains("nem NE nem PE"));
    let mut ne = vec![0u8; 0x100];
    ne[0] = 0x4d;
    ne[1] = 0x5a;
    ne[0x3c] = 0x40;
    ne[0x40] = 0x4e;
    ne[0x41] = 0x45;
    ne[0x64] = 0x40;
    assert!(erro(&ne).contains("nenhuma fonte"), "{}", erro(&ne));
}

#[test]
fn content_decides_format() {
    assert_eq!(
        read_font_file("qualquer.txt", BDF.as_bytes()).unwrap()[0].format,
        "bdf"
    );
    let erro = |b: &[u8]| read_font_file("x.fnt", b).err().unwrap_or_default();
    assert!(erro(b"info face=\"Arial\" size=32\ncommon lineHeight=32\n").contains("BMFont"));
    assert!(erro("olá, mundo".as_bytes()).contains(".bdf, .hex, .fnt e .fon"));
    assert!(erro(b"").contains("vazio"));
}

#[test]
fn decode_utf8_else_latin1() {
    assert_eq!(decode_text("Ælla é".as_bytes()), "Ælla é");
    assert_eq!(decode_text(&[0x63, 0xe9]), "cé");
}

#[test]
fn charsets_of_dfcharset() {
    assert_eq!(charset_to_unicode(0, 0x80), 0x20ac);
    assert_eq!(charset_to_unicode(0, 0x41), 0x41);
    assert_eq!(charset_to_unicode(255, 0x01), 0x263a);
    assert_eq!(charset_to_unicode(255, 0x7f), 0x2302);
    assert_eq!(charset_to_unicode(255, 0xe9), 0x398);
    assert_eq!(charset_to_unicode(204, 0xc0), 0x410);
    assert_eq!(charset_to_unicode(2, 0xc0), 0xc0);
}

/// As quatro de fábrica abrem e escrevem; a conferência glifo a glifo contra a FreeType é a do MVP
/// (`docs/Pixel Art/app/tests/font_oracle.py`, sobre estes MESMOS bytes).
#[test]
fn builtin_fonts_open_and_write() {
    for (name, lh) in [
        ("cozette", 13),
        ("tom-thumb", 6),
        ("unscii-8", 8),
        ("unscii-16", 16),
    ] {
        let f = builtin(name).unwrap();
        assert_eq!(f.line_height(), lh, "{name}");
        let style = TextStyle {
            font: FontChoice::Builtin(name.into()),
            px: 0,
            bold: false,
            italic: false,
            antialias: false,
        };
        let m = rasterize("Ag1", &style).unwrap();
        assert!(
            m.cover.iter().any(|&c| c == 255) && m.missing.is_empty(),
            "{name}"
        );
        let dobro = rasterize(
            "Ag1",
            &TextStyle {
                px: 2 * lh as u32,
                ..style
            },
        )
        .unwrap();
        assert_eq!(
            (dobro.w, dobro.h),
            (2 * m.w, 2 * m.h),
            "{name}: 2× é inteiro"
        );
    }
}
