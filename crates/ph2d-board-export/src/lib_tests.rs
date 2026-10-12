//! A saída (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 E1–E7).

use super::*;
use ph2d_board_model::{
    Anchor, BoardSet, Connector, Element, Frame, Rgba, RichText, Route, Shape, ShapeType, Style,
};
use ph2d_vector::DrawOp;

const INK: Rgba = Rgba([30, 30, 30, 255]);

fn shape(
    b: &mut Board,
    kind: ShapeType,
    text: &str,
    bx: [f64; 4],
    frame: Option<ElementId>,
) -> ElementId {
    let fill = if kind.is_note() {
        Some(Rgba([255, 249, 177, 255]))
    } else {
        None
    };
    let mut el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Shape {
            kind,
            style: Style::new(fill, (!kind.is_note()).then_some(INK), INK),
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
    let note = shape(
        b,
        ShapeType::Sticky,
        "Slow builds",
        [40.0, 40.0, 199.0, 199.0],
        Some(f1),
    );
    let f2 = frame(b, "Plan", [1000.0, 0.0, 800.0, 450.0]);
    shape(
        b,
        ShapeType::Rectangle,
        "Cache",
        [1100.0, 100.0, 160.0, 80.0],
        Some(f2),
    );
    let out = shape(
        b,
        ShapeType::Ellipse,
        "Outside",
        [0.0, 600.0, 160.0, 80.0],
        None,
    );
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
    assert!(
        board_page(empty.get(e).unwrap(), &RouteCache::default()).is_none(),
        "vazio: nada"
    );
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
    assert_eq!(
        pdf_pages(set.get(id).unwrap(), &r, &[f2, note])[0].frame,
        Some(f2)
    );
    // A selecção SEM frames de topo é a caixa dela numa página — nunca o deck (auditoria W8).
    let only_note = pdf_pages(set.get(id).unwrap(), &r, &[note]);
    assert_eq!((only_note.len(), only_note[0].frame), (1, None));
    assert_eq!(
        only_note[0].rect,
        [40.0 - PAD, 40.0 - PAD, 239.0 + PAD, 239.0 + PAD]
    );
    set.get_mut(id).unwrap().team.set_slide_hidden(f1, true);
    assert_eq!(pdf_pages(set.get(id).unwrap(), &r, &[]).len(), 1);
    let mut bare = BoardSet::default();
    let a = bare.create("x".into());
    shape(
        bare.get_mut(a).unwrap(),
        ShapeType::Rectangle,
        "",
        [0.0, 0.0, 10.0, 10.0],
        None,
    );
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
    let (mut scene, size) = paint_page(
        b,
        &page,
        1.0,
        Some(ph2d_board_render::EXPORT_PAPER),
        ExportTarget::Svg,
        &mut ExportCache::default(),
        &mut ts,
    );
    assert_eq!(size, [800.0, 450.0]);
    let ops = scene.take_record();
    assert_eq!(texts(&ops).concat(), "Slow builds");
    let strokes = ops
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { .. }))
        .count();
    let r = routes(b);
    let (mut all, _) = paint_page(
        b,
        &board_page(b, &r).unwrap(),
        1.0,
        None,
        ExportTarget::Svg,
        &mut ExportCache::default(),
        &mut ts,
    );
    let all_ops = all.take_record();
    let mut t = texts(&all_ops);
    t.sort();
    assert_eq!(
        t,
        ["Cache", "Outside", "Slow builds"],
        "sem «Problem» nem «Plan»"
    );
    let all_strokes = all_ops
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { .. }))
        .count();
    assert!(
        all_strokes > strokes,
        "a seta e os contornos de fora só no quadro"
    );
}

/// E7: papel branco por omissão; transparente = nenhum preenchimento de fundo.
#[test]
fn the_paper_is_white_or_nothing() {
    let (set, [f1, ..]) = board();
    let b = &set.boards()[0];
    let mut ts = TextSystem::without_system_fonts();
    let page = frame_page(b, f1).unwrap();
    let mut fills = |paper| {
        let (mut s, _) = paint_page(
            b,
            &page,
            1.0,
            paper,
            ExportTarget::Svg,
            &mut ExportCache::default(),
            &mut ts,
        );
        let ops = s.take_record();
        let first = ops.iter().find_map(|o| match o {
            DrawOp::Fill { brush, .. } => Some(brush.clone()),
            _ => None,
        });
        let n = ops
            .iter()
            .filter(|o| matches!(o, DrawOp::Fill { .. }))
            .count();
        (first, n)
    };
    let (white, with) = fills(Some(ph2d_board_render::EXPORT_PAPER));
    assert_eq!(
        white,
        Some(ph2d_vector::Brush::Solid(ph2d_board_render::EXPORT_PAPER))
    );
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
    let (mut s, size) = paint_page(
        b,
        &board_page(b, &r).unwrap(),
        2.0,
        None,
        ExportTarget::Svg,
        &mut ExportCache::default(),
        &mut ts,
    );
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
            let (mut sc, sz) = paint_page(
                b,
                p,
                1.0,
                Some(ph2d_board_render::EXPORT_PAPER),
                ExportTarget::Pdf,
                &mut ExportCache::default(),
                &mut ts,
            );
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

/// ⭐ E5 — o SVG é XML mesmo com caracteres de controlo no texto (um `.excalidraw` importado pode
/// trazê-los), e uma ligação vira `<a>` para uma `<view>` com a caixa do alvo.
#[test]
fn svg_stays_xml_and_carries_links() {
    assert_eq!(svg::escape("a\u{1}b<&\""), "a b&lt;&amp;&quot;");
    let s = svg::write_linked(
        &[],
        [100.0, 50.0],
        &[([1.0, 2.0, 11.0, 12.0], [50.0, 0.0, 90.0, 40.0])],
    );
    assert!(s.contains("<view id=\"link0\" viewBox=\"50 0 40 40\"/>"));
    assert!(
        s.contains("<a xlink:href=\"#link0\"><rect x=\"1\" y=\"2\" width=\"10\" height=\"10\"")
    );
    assert!(s.contains("xmlns:xlink"));
}

/// ⭐ W18 — um vídeo exportado leva o LINK dele: no PDF uma anotação `/URI`, no SVG um `<a href>` (o `&`
/// do endereço escapado) sobre a caixa dele.
#[test]
fn a_video_carries_its_link_out_in_pdf_and_svg() {
    let url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ&t=9s".to_owned();
    let bx = [10.0, 20.0, 570.0, 335.0];
    let s = svg::write_with_uris(&[], [600.0, 400.0], &[], &[(bx, url.clone())]);
    assert!(s.contains("<a xlink:href=\"https://www.youtube.com/watch?v=aqz-KE-bpKQ&amp;t=9s\""), "{s}");
    assert!(s.contains("<rect x=\"10\" y=\"20\" width=\"560\" height=\"315\""), "{s}");
    let page = pdf::PdfPage { ops: &[], size: [600.0, 400.0], links: vec![] };
    let bytes = pdf::write_with_uris(&[page], &[vec![(bx, url)]]);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/S /URI"), "a acção /URI");
    assert!(text.contains("/URI (https://www.youtube.com/watch?v=aqz-KE-bpKQ&t=9s)"), "o endereço");
    assert!(text.contains("/Rect [10 65 570 380]"), "a caixa em pontos do PDF (y para cima)");
}

/// ⭐ O PDF cresce com o NÚMERO de objectos translúcidos, nunca com o quadrado dele: cada grupo de
/// transparência declara só os recursos que usa (auditoria W8: com o dicionário inteiro em cada grupo,
/// 500 → 5,2 MB e 2 000 → 85 MB).
#[test]
fn the_pdf_grows_linearly_with_faded_objects() {
    // `n` objectos esbatidos com `k` opacidades diferentes (cada uma o seu estado de alfa `A*`).
    let size_for = |n: usize, k: usize| {
        let mut set = BoardSet::default();
        let a = set.create("Faded".into());
        let b = set.get_mut(a).unwrap();
        for i in 0..n {
            let id = shape(
                b,
                ShapeType::Rectangle,
                "abc",
                [(i % 40) as f64 * 60.0, (i / 40) as f64 * 60.0, 50.0, 40.0],
                None,
            );
            let mut el = b.doc.get(id).unwrap().clone();
            el.style_mut().opacity = u8::try_from(1 + i % k).unwrap();
            BoardOp::Put(el).apply(&mut b.doc);
        }
        let b = set.get(a).unwrap().clone();
        let mut r = RouteCache::default();
        r.sync(&b.doc);
        let mut ts = TextSystem::without_system_fonts();
        let (mut s, size) = paint_page(
            &b,
            &board_page(&b, &r).unwrap(),
            1.0,
            None,
            ExportTarget::Pdf,
            &mut ExportCache::default(),
            &mut ts,
        );
        let ops = s.take_record();
        pdf::write(&[pdf::PdfPage {
            ops: &ops,
            size,
            links: Vec::new(),
        }])
        .len() as f64
    };
    let (small, big) = (size_for(200, 1), size_for(800, 1));
    assert!(
        big / small < 6.0,
        "4× os objectos deu {:.1}× os bytes",
        big / small
    );
    // Cada conteúdo declara só os estados de alfa que USA (prova de mutação W8: com todos em cada
    // grupo, 9× as opacidades multiplicava os bytes; só os usados, quase nada).
    let (few, many) = (size_for(180, 10), size_for(180, 90));
    assert!(
        many / few < 1.5,
        "9× as opacidades deu {:.2}× os bytes",
        many / few
    );
}

/// ⭐ W13 (prova de mutação, M18; pontas da W12): a nota que quem exporta não lê sai como no ecrã — sem o
/// texto (que é um texto que já não existe). ⛔ Controlo: sem ela, o texto sai.
#[test]
fn an_unread_note_is_exported_without_its_text() {
    let (set, [_, note, ..]) = board();
    let b = &set.boards()[0];
    let r = routes(b);
    let mut ts = TextSystem::without_system_fonts();
    let mut out = |unread: std::collections::BTreeSet<ElementId>| {
        let page = board_page(b, &r).unwrap();
        let (mut s, _) = paint_page(
            b,
            &page,
            1.0,
            None,
            ExportTarget::Svg,
            &mut ExportCache::new(unread),
            &mut ts,
        );
        texts(&s.take_record())
    };
    assert!(
        out(std::collections::BTreeSet::new())
            .iter()
            .any(|t| t == "Slow builds")
    );
    let hidden = out(std::collections::BTreeSet::from([note]));
    assert!(!hidden.iter().any(|t| t.contains("Slow")), "{hidden:?}");
}
