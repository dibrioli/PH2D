//! A saída (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 E1–E7).

use super::*;
use ph2d_board_model::{
    Anchor, BoardSet, Connector, Element, Frame, Rgba, RichText, Route, Shape, ShapeType, Style,
};
use ph2d_vector::DrawOp;

const INK: Rgba = Rgba([30, 30, 30, 255]);

fn shape(b: &mut Board, kind: ShapeType, text: &str, bx: [f64; 4], frame: Option<ElementId>) -> ElementId {
    let fill = if kind.is_note() { Some(Rgba([255, 249, 177, 255])) } else { None };
    let mut el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Shape {
            kind,
            style: Style::new(fill, Some(INK).filter(|_| !kind.is_note()), INK),
            text: RichText::plain(text),
        },
        bx,
    );
    el.frame = frame;
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

fn frame(b: &mut Board, name: &str, bx: [f64; 4]) -> ElementId {
    let f = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Frame {
            name: name.into(),
            style: Style::new(Some(Rgba([255; 4])), None, INK),
        },
        bx,
    );
    let id = f.id;
    BoardOp::Put(f).apply(&mut b.doc);
    id
}

/// Dois frames («Problem» com uma nota, «Plan» com uma caixa), uma caixa solta fora, e uma seta da nota
/// para a caixa de fora.
fn board() -> (BoardSet, [ElementId; 5]) {
    let mut set = BoardSet::default();
    let a = set.create("Launch".into());
    let b = set.get_mut(a).unwrap();
    let f1 = frame(b, "Problem", [0.0, 0.0, 800.0, 450.0]);
    let note = shape(b, ShapeType::Sticky, "Slow builds", [40.0, 40.0, 199.0, 199.0], Some(f1));
    let f2 = frame(b, "Plan", [1000.0, 0.0, 800.0, 450.0]);
    shape(b, ShapeType::Rectangle, "Cache", [1100.0, 100.0, 160.0, 80.0], Some(f2));
    let out = shape(b, ShapeType::Ellipse, "Outside", [0.0, 600.0, 160.0, 80.0], None);
    let arrow = Element::new_connector(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Connector::new(
            ph2d_board_model::End::Bound {
                target: note,
                anchor: Anchor::Center,
            },
            ph2d_board_model::End::Bound {
                target: out,
                anchor: Anchor::Center,
            },
            Route::Straight,
            Style::new(None, Some(INK), INK),
        ),
    );
    let arrow_id = arrow.id;
    BoardOp::Put(arrow).apply(&mut b.doc);
    (set, [f1, note, f2, out, arrow_id])
}

fn texts(ops: &[DrawOp]) -> Vec<String> {
    ops.iter()
        .filter_map(|o| match o {
            DrawOp::Glyphs(r) => Some(r.text.clone()),
            _ => None,
        })
        .collect()
}

fn routes(b: &Board) -> RouteCache {
    let mut r = RouteCache::default();
    r.sync(&b.doc);
    r
}

/// ⭐ E2: o quadro inteiro é a caixa dos vivos (as setas pela rota) + 10; a selecção leva o que vive
/// dentro dos frames dela.
#[test]
fn the_board_page_is_the_extent_plus_the_excalidraw_padding() {
    let (set, [f1, ..]) = board();
    let b = &set.boards()[0];
    let r = routes(b);
    let p = board_page(b, &r).unwrap();
    assert_eq!(p.rect, [-PAD, -PAD, 1800.0 + PAD, 680.0 + PAD]);
    assert_eq!(p.frame, None);
    let s = selection_page(b, &r, &[f1]).unwrap();
    assert_eq!(s.rect, [-PAD, -PAD, 800.0 + PAD, 450.0 + PAD]);
    let mut empty = BoardSet::default();
    let e = empty.create("e".into());
    assert!(board_page(empty.get(e).unwrap(), &RouteCache::default()).is_none(), "vazio: nada");
}

/// ⭐ E4: o PDF tem um frame por página na ordem do painel, sem os escondidos; a selecção restringe aos
/// frames dela; sem frames, uma página com o quadro.
#[test]
fn pdf_pages_are_the_slides_or_the_selected_frames() {
    let (mut set, [f1, note, f2, ..]) = board();
    let id = set.boards()[0].id;
    let r = routes(set.get(id).unwrap());
    let pages = pdf_pages(set.get(id).unwrap(), &r, &[]);
    assert_eq!(
        pages.iter().map(|p| p.frame).collect::<Vec<_>>(),
        [Some(f1), Some(f2)]
    );
    assert_eq!(pages[1].rect, [1000.0, 0.0, 1800.0, 450.0]);
    assert_eq!(pdf_pages(set.get(id).unwrap(), &r, &[f2, note])[0].frame, Some(f2));
    set.get_mut(id).unwrap().team.set_slide_hidden(f1, true);
    assert_eq!(pdf_pages(set.get(id).unwrap(), &r, &[]).len(), 1);
    let mut bare = BoardSet::default();
    let a = bare.create("x".into());
    shape(bare.get_mut(a).unwrap(), ShapeType::Rectangle, "", [0.0, 0.0, 10.0, 10.0], None);
    let p = pdf_pages(bare.get(a).unwrap(), &RouteCache::default(), &[]);
    assert_eq!((p.len(), p[0].frame), (1, None));
}

/// ⭐ E3: o PNG mede-se por ÁREA — Small 4, Medium 16, Large 49 MP — com o rácio da área.
#[test]
fn png_size_is_an_area_in_megapixels() {
    let p = Page {
        rect: [0.0, 0.0, 3000.0, 1000.0],
        frame: None,
    };
    for mp in PNG_MEGAPIXELS {
        let s = png_scale(&p, mp);
        let (w, h) = (3000.0 * s, 1000.0 * s);
        assert!((w * h - mp * 1e6).abs() < 1.0, "{mp}: {w}×{h}");
        assert!((w / h - 3.0).abs() < 1e-9);
    }
}

/// ⭐ E6 e «only widgets inside of frames»: a página de um frame desenha o frame e o que vive nele — não
/// a caixa de fora, nem a seta que sai dele, nem o título; a do quadro desenha tudo menos os títulos.
#[test]
fn a_frame_page_draws_only_its_frame_and_never_titles() {
    let (set, [f1, ..]) = board();
    let b = &set.boards()[0];
    let mut ts = TextSystem::without_system_fonts();
    let page = frame_page(b, f1).unwrap();
    let (mut scene, size) = paint_page(b, &page, 1.0, Some(ph2d_board_render::EXPORT_PAPER), true, &mut ts);
    assert_eq!(size, [800.0, 450.0]);
    let ops = scene.take_record();
    assert_eq!(texts(&ops).concat(), "Slow builds");
    let strokes = ops
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { .. }))
        .count();
    let r = routes(b);
    let (mut all, _) = paint_page(b, &board_page(b, &r).unwrap(), 1.0, None, true, &mut ts);
    let all_ops = all.take_record();
    let mut t = texts(&all_ops);
    t.sort();
    assert_eq!(t, ["Cache", "Outside", "Slow builds"], "sem «Problem» nem «Plan»");
    let all_strokes = all_ops
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { .. }))
        .count();
    assert!(all_strokes > strokes, "a seta e os contornos de fora só no quadro");
}

/// E7: papel branco por omissão; transparente = nenhum preenchimento de fundo.
#[test]
fn the_paper_is_white_or_nothing() {
    let (set, [f1, ..]) = board();
    let b = &set.boards()[0];
    let mut ts = TextSystem::without_system_fonts();
    let page = frame_page(b, f1).unwrap();
    let mut fills = |paper| {
        let (mut s, _) = paint_page(b, &page, 1.0, paper, true, &mut ts);
        let ops = s.take_record();
        let first = ops.iter().find_map(|o| match o {
            DrawOp::Fill { brush, .. } => Some(brush.clone()),
            _ => None,
        });
        let n = ops.iter().filter(|o| matches!(o, DrawOp::Fill { .. })).count();
        (first, n)
    };
    let (white, with) = fills(Some(ph2d_board_render::EXPORT_PAPER));
    assert_eq!(white, Some(ph2d_vector::Brush::Solid(ph2d_board_render::EXPORT_PAPER)));
    let (_, without) = fills(None);
    assert_eq!(with, without + 1, "transparente: o papel não se pinta");
}

/// ⭐ E5: o SVG é um documento com o texto (em `aria-label`) e as letras como caminhos; o PDF tem uma
/// página por frame e o texto real (o `ActualText`).
#[test]
fn svg_and_pdf_carry_the_text() {
    let (set, _) = board();
    let b = &set.boards()[0];
    let r = routes(b);
    let mut ts = TextSystem::without_system_fonts();
    let (mut s, size) = paint_page(b, &board_page(b, &r).unwrap(), 2.0, None, true, &mut ts);
    let ops = s.take_record();
    let svg = svg::write(&ops, size);
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.ends_with("</svg>\n"));
    assert!(svg.contains("aria-label=\"Slow builds\""));
    assert!(svg.contains("filter=\"url(#b"), "a sombra da nota");
    let pages = pdf_pages(b, &r, &[]);
    let recs: Vec<_> = pages
        .iter()
        .map(|p| {
            let (mut sc, sz) = paint_page(b, p, 1.0, Some(ph2d_board_render::EXPORT_PAPER), true, &mut ts);
            (sc.take_record(), sz)
        })
        .collect();
    let pdf_pages: Vec<pdf::PdfPage<'_>> = recs
        .iter()
        .map(|(ops, size)| pdf::PdfPage {
            ops,
            size: *size,
            links: Vec::new(),
        })
        .collect();
    let bytes = pdf::write(&pdf_pages);
    assert!(bytes.starts_with(b"%PDF-"));
    let text = String::from_utf8_lossy(&bytes);
    let pages = text.matches("/Type /Page").count() - text.matches("/Type /Pages").count();
    assert_eq!(pages, 2, "uma página por frame");
}
