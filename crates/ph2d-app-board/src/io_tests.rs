//! Os ficheiros do quadro (W8) sem o diálogo: o que se escreve num caminho e o que se abre de um.

use super::*;
use ph2d_board_model::{
    BoardOp, BoardSet, Element, Frame, Rgba, RichText, Shape, ShapeType, Style,
};

const INK: Rgba = Rgba([30, 30, 30, 255]);

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_w8_io_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Dois frames com uma forma cada; a do 1.º leva (link do W5) à do 2.º.
fn board() -> Board {
    let mut set = BoardSet::default();
    let a = set.create("Launch".into());
    let b = set.get_mut(a).unwrap();
    let mut ids = Vec::new();
    for (i, name) in ["Problem", "Plan"].into_iter().enumerate() {
        let x = i as f64 * 1000.0;
        let f = Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Frame {
                name: name.into(),
                style: Style::new(Some(Rgba([255; 4])), None, INK),
            },
            [x, 0.0, 800.0, 450.0],
        );
        let fid = f.id;
        BoardOp::Put(f).apply(&mut b.doc);
        let mut s = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Shape {
                kind: ShapeType::Rectangle,
                style: Style::new(Some(Rgba([200, 230, 255, 255])), Some(INK), INK),
                text: RichText::plain(name),
            },
            [x + 100.0, 100.0, 200.0, 100.0],
        );
        s.frame = Some(fid);
        ids.push(s.id);
        BoardOp::Put(s).apply(&mut b.doc);
    }
    let mut first = b.doc.get(ids[0]).unwrap().clone();
    first.set_link(Some(ids[1]));
    BoardOp::Put(first).apply(&mut b.doc);
    b.clone()
}

fn choice(format: Format, scope: Scope) -> ExportChoice {
    ExportChoice {
        format,
        scope,
        ..ExportChoice::default()
    }
}

/// ⭐ E4/E5 — o PDF do quadro: uma página por frame, e o link entre elas (uma anotação GoTo).
#[test]
fn the_pdf_has_a_page_per_frame_and_the_link_between_them() {
    let d = dir("pdf");
    let mut ts = TextSystem::without_system_fonts();
    let path = d.join("b.pdf");
    let files = write_export(
        &board(),
        choice(Format::Pdf, Scope::Board),
        &[],
        None,
        &mut ts,
        &path,
    )
    .unwrap();
    assert_eq!(files, [path.clone()]);
    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(bytes.starts_with(b"%PDF-"));
    assert_eq!(
        text.matches("/Type /Page").count() - text.matches("/Type /Pages").count(),
        2
    );
    assert_eq!(
        text.matches("/Subtype /Link").count(),
        1,
        "o link do 1.º frame para o 2.º"
    );
}

/// E2 — o SVG do quadro; vários frames em SVG = um ficheiro por frame, com o nome do frame.
#[test]
fn svg_writes_one_file_or_one_per_frame() {
    let d = dir("svg");
    let mut ts = TextSystem::without_system_fonts();
    let b = board();
    let one = d.join("b.svg");
    let files = write_export(
        &b,
        choice(Format::Svg, Scope::Board),
        &[],
        None,
        &mut ts,
        &one,
    )
    .unwrap();
    assert_eq!(files, [one.clone()]);
    assert!(std::fs::read_to_string(&one).unwrap().starts_with("<svg"));
    let many = write_export(
        &b,
        choice(Format::Svg, Scope::Frames),
        &[],
        None,
        &mut ts,
        &one,
    )
    .unwrap();
    let names: Vec<String> = many.iter().map(|p| display(p)).collect();
    assert_eq!(names, ["b-Problem.svg", "b-Plan.svg"]);
}

/// ⭐ X1/X3 — o `.excalidraw` escrito no disco abre de volta com os mesmos elementos (e o nome).
#[test]
fn an_excalidraw_file_written_opens_back() {
    let d = dir("exc");
    let mut ts = TextSystem::without_system_fonts();
    let b = board();
    let path = d.join("Launch.excalidraw");
    write_export(
        &b,
        choice(Format::Excalidraw, Scope::Board),
        &[],
        None,
        &mut ts,
        &path,
    )
    .unwrap();
    let (name, imp) = import_file(&path).unwrap();
    assert_eq!(name, "Launch");
    assert_eq!(imp.doc.live_len(), b.doc.live_len());
    assert!(imp.skipped.is_empty());
}

/// X3 — a cena EMBEBIDA num PNG do Excalidraw (a fixture do oráculo) abre num quadro.
#[test]
fn an_excalidraw_png_with_the_scene_opens() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_ph2d_quadro.embed.png");
    let d = dir("png_in");
    let path = d.join("drawing.excalidraw.png");
    std::fs::copy(src, &path).unwrap();
    let (name, imp) = import_file(&path).unwrap();
    assert_eq!(name, "drawing");
    assert!(imp.doc.live_len() > 5);
    let bad = d.join("plain.txt");
    std::fs::write(&bad, "hello").unwrap();
    assert!(
        import_file(&bad).is_err(),
        "um ficheiro qualquer recusa-se com a razão"
    );
}

/// O PNG sem placa recusa-se com a razão (a interface pinta-a), e nada a exportar também.
#[test]
fn png_without_a_gpu_and_an_empty_board_refuse_with_a_reason() {
    let d = dir("refuse");
    let mut ts = TextSystem::without_system_fonts();
    let e = write_export(
        &board(),
        choice(Format::Png, Scope::Board),
        &[],
        None,
        &mut ts,
        &d.join("b.png"),
    );
    assert_eq!(e, Err(tr("board.io.no_gpu").to_owned()));
    let mut set = BoardSet::default();
    let a = set.create("Empty".into());
    let empty = set.get(a).unwrap().clone();
    let e = write_export(
        &empty,
        choice(Format::Svg, Scope::Board),
        &[],
        None,
        &mut ts,
        &d.join("e.svg"),
    );
    assert_eq!(e, Err(tr("board.io.nothing").to_owned()));
}

/// O nome de um quadro num nome de ficheiro, sem barras.
#[test]
fn file_names_lose_the_slashes() {
    assert_eq!(file_stem("Q1/Q2: plan"), "Q1_Q2_ plan");
    assert_eq!(file_stem("   "), "board");
}

/// ⭐ E5 — sem frames, o link vale na MESMA página (PDF e SVG); e dois frames com o mesmo nome dão dois
/// ficheiros (auditoria W8).
#[test]
fn links_work_on_one_page_and_frame_names_never_collide() {
    let d = dir("links");
    let mut ts = TextSystem::without_system_fonts();
    let mut b = board();
    // Sem frames: as formas soltas, o link entre elas.
    let frames: Vec<_> = b
        .doc
        .live()
        .filter(|e| e.as_frame().is_some())
        .map(|e| e.id)
        .collect();
    for f in &frames {
        let kids: Vec<_> = b
            .doc
            .live()
            .filter(|e| e.frame == Some(*f))
            .cloned()
            .collect();
        for mut k in kids {
            k.frame = None;
            BoardOp::Put(k).apply(&mut b.doc);
        }
        BoardOp::Delete(*f).apply(&mut b.doc);
    }
    let pdf_path = d.join("one.pdf");
    write_export(
        &b,
        choice(Format::Pdf, Scope::Board),
        &[],
        None,
        &mut ts,
        &pdf_path,
    )
    .unwrap();
    let text = String::from_utf8_lossy(&std::fs::read(&pdf_path).unwrap()).into_owned();
    assert_eq!(
        text.matches("/Subtype /Link").count(),
        1,
        "o link na mesma página"
    );
    let svg_path = d.join("one.svg");
    write_export(
        &b,
        choice(Format::Svg, Scope::Board),
        &[],
        None,
        &mut ts,
        &svg_path,
    )
    .unwrap();
    assert!(
        std::fs::read_to_string(&svg_path)
            .unwrap()
            .contains("<a xlink:href=\"#link0\">")
    );
    // Dois frames com o mesmo nome.
    let mut twin = board();
    let ids: Vec<_> = twin
        .doc
        .live()
        .filter(|e| e.as_frame().is_some())
        .map(|e| e.id)
        .collect();
    for f in ids {
        let mut el = twin.doc.get(f).unwrap().clone();
        if let ph2d_board_model::ElementKind::Frame(fr) = &mut el.kind {
            fr.name = "Plan".into();
        }
        BoardOp::Put(el).apply(&mut twin.doc);
    }
    let files = write_export(
        &twin,
        choice(Format::Svg, Scope::Frames),
        &[],
        None,
        &mut ts,
        &d.join("t.svg"),
    )
    .unwrap();
    let names: Vec<String> = files.iter().map(|p| display(p)).collect();
    assert_eq!(names, ["t-Plan.svg", "t-Plan-2.svg"]);
}
