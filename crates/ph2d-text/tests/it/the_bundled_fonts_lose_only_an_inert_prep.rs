//! ⭐⭐ **O registo tira o `prep` inerte das fontes embutidas, e SÓ ele** (CodeEditor 15/10).
//!
//! Com o `prep` (7 bytes de *dropout control*, zero instruções nos glifos) o skrifa escolhia o
//! interpretador TrueType e o `hint(true)` era um no-op: as duas barras do `=` saíam desiguais
//! (`docs/CodeEditor/ferramentas/medicoes/igual_perfis.txt`). Sem ele corre o autohinter.

use parley::fontique::{Blob, Collection, CollectionOptions, FontInfoOverride, SourceCache};
use parley::{
    FontContext, FontFamily, FontWeight, LayoutContext, PositionedLayoutItem, StyleProperty,
};
use ph2d_text::{TextSystem, sfnt_checksum, without_prep_table};
use read_fonts::types::Tag;
use read_fonts::{FontRef, TableProvider};
use std::sync::Arc;

const NOTO: &[u8] = include_bytes!("../../fonts/NotoSansVariable.ttf");
const ATKINSON: &[u8] = include_bytes!("../../fonts/AtkinsonHyperlegibleNextVariable.ttf");
const MONO: &[u8] = include_bytes!("../../fonts/JetBrainsMonoVariable.ttf");
const INTER: &[u8] = include_bytes!("../../fonts/InterVariable.ttf");
const VIRGIL: &[u8] = include_bytes!("../../fonts/Virgil-Regular.ttf");
const COM_PREP: [(&str, &[u8]); 3] = [("noto", NOTO), ("atkinson", ATKINSON), ("mono", MONO)];
const AMOSTRA: &str =
    "Script  Browse  amplitude  speed  line 3 · Expected identifier, got '=' == ação";

fn tabela<'a>(f: &FontRef<'a>, tag: Tag) -> &'a [u8] {
    f.table_data(tag).map(|d| d.as_bytes()).unwrap_or_default()
}

#[test]
fn only_the_prep_table_leaves_and_the_font_stays_valid() {
    for (nome, raw) in COM_PREP {
        let novo = without_prep_table(raw).unwrap_or_else(|| panic!("{nome}: não tirou o prep"));
        let (a, b) = (FontRef::new(raw).unwrap(), FontRef::new(&novo).unwrap());
        let prep = Tag::new(b"prep");
        assert!(
            a.table_data(prep).is_some(),
            "{nome}: controlo — o original tem prep"
        );
        assert!(b.table_data(prep).is_none(), "{nome}: o prep ficou");
        let tags_a: Vec<Tag> = a
            .table_directory
            .table_records()
            .iter()
            .map(|r| r.tag())
            .filter(|&t| t != prep)
            .collect();
        let tags_b: Vec<Tag> = b
            .table_directory
            .table_records()
            .iter()
            .map(|r| r.tag())
            .collect();
        assert_eq!(tags_a, tags_b, "{nome}: as outras tabelas mudaram");
        for r in b.table_directory.table_records() {
            let (ta, tb) = (tabela(&a, r.tag()), tabela(&b, r.tag()));
            if r.tag() == Tag::new(b"head") {
                // Só o `checkSumAdjustment` (bytes 8..12) muda.
                assert_eq!((&ta[..8], &ta[12..]), (&tb[..8], &tb[12..]), "{nome}: head");
                let mut zero = tb.to_vec();
                zero[8..12].fill(0);
                assert_eq!(
                    sfnt_checksum(&zero),
                    r.checksum(),
                    "{nome}: checksum do head"
                );
            } else {
                assert_eq!(ta, tb, "{nome}: a tabela {} mudou", r.tag());
                assert_eq!(
                    sfnt_checksum(tb),
                    r.checksum(),
                    "{nome}: checksum de {}",
                    r.tag()
                );
            }
            assert_eq!(r.offset() % 4, 0, "{nome}: {} desalinhada", r.tag());
        }
        assert_eq!(
            sfnt_checksum(&novo),
            0xB1B0_AFBA,
            "{nome}: checksum do ficheiro"
        );
        assert_eq!(novo.len() % 4, 0);
    }
}

/// O que não tem `prep`, ou tem hinting de verdade, fica intocado.
#[test]
fn fonts_without_an_inert_prep_are_left_alone() {
    assert!(without_prep_table(INTER).is_none(), "a Inter não tem prep");
    assert!(
        without_prep_table(VIRGIL).is_none(),
        "a Virgil não tem prep"
    );
    // Controlo: a Noto com uma tabela renomeada `fpgm` (hinting de verdade) é recusada.
    let mut com_fpgm = NOTO.to_vec();
    let n = usize::from(u16::from_be_bytes([NOTO[4], NOTO[5]]));
    let i = (0..n)
        .find(|&j| &NOTO[12 + 16 * j..16 + 16 * j] == b"gasp")
        .unwrap();
    com_fpgm[12 + 16 * i..16 + 16 * i].copy_from_slice(b"fpgm");
    assert!(
        without_prep_table(&com_fpgm).is_none(),
        "tirou o prep de uma fonte com fpgm"
    );
    assert!(
        without_prep_table(NOTO).is_some(),
        "controlo: a Noto sem a troca perde o prep"
    );
}

fn molda(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<()>,
    family: &str,
    wght: f32,
) -> Vec<(u32, u32, u32)> {
    let mut b = lcx.ranged_builder(fcx, AMOSTRA, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(family.into())));
    b.push_default(StyleProperty::FontSize(11.08));
    b.push_default(StyleProperty::FontWeight(FontWeight::new(wght)));
    let mut layout = b.build(AMOSTRA);
    layout.break_all_lines(None);
    let mut out = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(gr) = item {
                out.extend(
                    gr.positioned_glyphs()
                        .map(|g| (g.id, g.x.to_bits(), g.advance.to_bits())),
                );
            }
        }
    }
    out
}

#[test]
fn the_advances_do_not_move_a_bit() {
    let mut fcx = FontContext {
        collection: Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        }),
        source_cache: SourceCache::default(),
    };
    let mut lcx = LayoutContext::new();
    for (nome, raw) in COM_PREP {
        let novo = without_prep_table(raw).unwrap();
        for (fam, bytes) in [
            (format!("A {nome}"), raw.to_vec()),
            (format!("B {nome}"), novo),
        ] {
            let reg = fcx.collection.register_fonts(
                Blob::new(Arc::new(bytes)),
                Some(FontInfoOverride {
                    family_name: Some(&fam),
                    ..Default::default()
                }),
            );
            assert!(!reg.is_empty(), "{fam} não registou");
        }
        for wght in [300.0, 500.0, 800.0] {
            let a = molda(&mut fcx, &mut lcx, &format!("A {nome}"), wght);
            let b = molda(&mut fcx, &mut lcx, &format!("B {nome}"), wght);
            assert!(a.len() > 40, "{nome}: a amostra moldou {} glifos", a.len());
            assert_eq!(a, b, "{nome} wght {wght}: os avanços mudaram");
        }
    }
}

/// ⭐⭐ **Toda fonte da interface e a mono, COMO REGISTADAS, deixam o skrifa escolher o
/// autohinter** — sem `fpgm`, sem `prep`, sem instruções (a regra do `Engine::AutoFallback`).
#[test]
fn every_registered_font_reaches_the_autohinter() {
    let mut ts = TextSystem::without_system_fonts();
    let mut vistas = Vec::new();
    for font in ph2d_tokens::UiFont::ALL {
        ph2d_text::set_active_text_style(ph2d_tokens::UiTextStyle {
            font,
            ..Default::default()
        });
        let layout = ts.layout(AMOSTRA, 11.0, 4000.0);
        vistas.push((format!("{font:?}"), fonte(&layout)));
    }
    ph2d_text::set_active_text_style(ph2d_tokens::UiTextStyle::default());
    let (fcx, lcx, stack) = ts.mono_contexts();
    let mut b = lcx.ranged_builder(fcx, AMOSTRA, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(stack.into())));
    let mut layout = b.build(AMOSTRA);
    layout.break_all_lines(None);
    vistas.push(("mono".into(), fonte(&layout)));
    for (nome, bytes) in &vistas {
        let f = FontRef::new(bytes).unwrap();
        for t in [b"fpgm", b"prep"] {
            assert!(
                f.table_data(Tag::new(t)).is_none(),
                "{nome}: registada com {}",
                Tag::new(t)
            );
        }
        let (loca, glyf) = (f.loca(None).unwrap(), f.glyf().unwrap());
        let n = f.maxp().unwrap().num_glyphs();
        let com = (0..u32::from(n))
            .filter(|&g| match loca.get_glyf(g.into(), &glyf).unwrap() {
                Some(read_fonts::tables::glyf::Glyph::Simple(s)) => !s.instructions().is_empty(),
                Some(read_fonts::tables::glyf::Glyph::Composite(c)) => {
                    c.instructions().is_some_and(|i| !i.is_empty())
                }
                None => false,
            })
            .count();
        assert_eq!(com, 0, "{nome}: {com} glifos com instruções");
    }
    // Controlo: as quatro são fontes diferentes (a pilha não caiu numa só).
    let mut unicas: Vec<&Vec<u8>> = vistas.iter().map(|v| &v.1).collect();
    unicas.sort();
    unicas.dedup();
    assert_eq!(
        unicas.len(),
        vistas.len(),
        "duas pilhas deram a mesma fonte"
    );
}

fn fonte(layout: &parley::Layout<()>) -> Vec<u8> {
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(gr) = item {
                return gr.run().font().data.data().to_vec();
            }
        }
    }
    panic!("nada moldado");
}
